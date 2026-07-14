use crate::entities::protocol::{TransferDirection, TransferRole};
use crate::protocols::dsp::entities::auth::TransferDSPAuthn;
use crate::protocols::dsp::entities::context_common::{BuildAuthn, TransferContextRaw, header};
use crate::protocols::dsp::entities::context_common::{
    TransferContextConnectorRole, TransferContextProcessSlot,
};
use crate::protocols::dsp::entities::message_types::TransferDSPMessageType;
use crate::protocols::dsp::entities::rdf_extractor_dsp::DspTransferRdfExtractor;
use common::dsp_common::data_address::DataAddress;
use common::dsp_common::odrl::OdrlAgreement;
use common::dsp_common::well_known_types::DSPProtocolVersions;
use ymir::data::entities::shared::participant::Model as Mates;
use common::rdf::dsp::DspCanonicalizer;
use http::request::Parts;
use sha2::{Digest, Sha256};
use std::str::FromStr;
use urn::Urn;
use ymir::errors::{Errors, Outcome};

/// Context Created from DSP controllers
/// The context is used for validation at several steps, command for Manager creation

// TransferContextRaw --
impl BuildAuthn for TransferDSPAuthn {
    fn from_request_parts(parts: &Parts) -> Outcome<Self> {
        let associated_participant = parts.extensions.get::<Mates>().cloned().ok_or_else(|| {
            Errors::crazy(
                "auth middleware did not resolve participant (Mates missing)",
                None,
            )
        })?;
        let raw = header(&parts.headers, "authorization").unwrap_or_default();
        let (token_type, token_content) = raw
            .split_once(' ')
            .map(|(t, c)| (t.to_string(), c.trim().to_string()))
            .unwrap_or_else(|| (String::new(), raw.clone()));
        Ok(TransferDSPAuthn {
            raw,
            token_type,
            token_content,
            associated_participant,
        })
    }
}

// TransferDSPContextParsed --

#[derive(Debug)]
pub struct TransferDSPContextParsed {
    pub raw: TransferContextRaw<TransferDSPAuthn>,
    pub dsp_version: DSPProtocolVersions,
    pub dsp_message_type: TransferDSPMessageType,
    pub json_value: serde_json::Value,
}

impl TransferDSPContextParsed {
    /// Wrap the raw context with the parse-stage output (version, message type,
    /// JSON body). The parsing itself lives in the pipeline `json_schema` stage.
    pub fn from_raw(
        raw: TransferContextRaw<TransferDSPAuthn>,
        dsp_version: DSPProtocolVersions,
        dsp_message_type: TransferDSPMessageType,
        json_value: serde_json::Value,
    ) -> Outcome<Self> {
        Ok(Self {
            raw,
            dsp_version,
            dsp_message_type,
            json_value,
        })
    }
}

// TransferDSPContextRdf --

#[derive(Debug)]
pub struct TransferDSPContextRdf {
    pub parsed: TransferDSPContextParsed,
    /// URDNA2015 / RDFC-1.0 canonical n-quads of the expanded message.
    pub canonical_n_quads: String,
    pub canonical_hash: [u8; 32],
}

impl TransferDSPContextRdf {
    /// Expand the DSP JSON-LD message to RDF and canonicalize it (URDNA2015 /
    /// RDFC-1.0) so `canonical_hash` is a stable content identity for idempotency
    /// and message signing. See [`common::rdf::dsp::DspCanonicalizer`].
    pub async fn from_parsed(parsed: TransferDSPContextParsed) -> Outcome<Self> {
        let canonical_n_quads = DspCanonicalizer::new(parsed.json_value.clone())
            .canonicalize()
            .await?;
        let canonical_hash = Sha256::digest(canonical_n_quads.as_bytes()).into();
        Ok(Self {
            parsed,
            canonical_n_quads,
            canonical_hash,
        })
    }
}

// TransferDSPContextTyped --

#[derive(Debug)]
pub struct TransferDSPContextTyped {
    pub rdf: TransferDSPContextRdf,
    pub message: TransferDSPMessageType,
    pub consumer_pid: Option<String>,
    pub provider_pid: Option<String>,
    pub data_address: Option<DataAddress>,
}

impl TransferDSPContextTyped {
    /// Build the typed context from the RDF stage (the `Rdf to Typed` pipeline
    /// step). This is the only public entry; field extraction lives in the
    /// private [`DspTransferRdfExtractor`].
    pub fn from_rdf(rdf: TransferDSPContextRdf) -> Outcome<Self> {
        let extractor = DspTransferRdfExtractor::new(&rdf);
        let message = extractor.rdf.parsed.dsp_message_type.clone();
        let consumer_pid = extractor.consumer_pid();
        let provider_pid = extractor.provider_pid();
        let data_address = extractor.data_address()?;
        Ok(TransferDSPContextTyped {
            rdf,
            message,
            consumer_pid,
            provider_pid,
            data_address,
        })
    }
}

// TransferDSPContextDomain --

#[derive(Debug)]
pub struct TransferDSPContextDomain {
    pub typed: TransferDSPContextTyped,
    pub process: TransferContextProcessSlot,
    pub agreement: OdrlAgreement,
    pub role: TransferRole,
    pub transfer_direction: TransferDirection,
    pub connector_instance: TransferContextConnectorRole,
    pub is_restart: bool,
    pub is_idempotent_replay: bool,
    pub resolved_data_address: Option<DataAddress>,
}

impl TransferDSPContextDomain {
    /// Wrap the typed context with the domain facts resolved by the
    /// `domain_loader` stage: the process slot (loaded or newly minted),
    /// agreement, role, connector, and the restart / idempotent-replay flags.
    pub fn from_typed(
        typed: TransferDSPContextTyped,
        process: TransferContextProcessSlot,
        agreement: OdrlAgreement,
        role: TransferRole,
        transfer_direction: TransferDirection,
        connector_instance: TransferContextConnectorRole,
        is_restart: bool,
        is_idempotent_replay: bool,
    ) -> Outcome<Self> {
        Ok(Self {
            typed,
            process,
            agreement,
            role,
            transfer_direction,
            connector_instance,
            is_restart,
            is_idempotent_replay,
            resolved_data_address: None,
        })
    }

    // getters
    pub fn process_urn(&self, location: &str) -> Outcome<Urn> {
        match &self.process {
            TransferContextProcessSlot::Existing(p) => Ok(p.id().as_urn().clone()),
            TransferContextProcessSlot::New { consumer_pid } => Urn::from_str(consumer_pid)
                .map_err(|_| {
                    Errors::crazy(format!("invalid consumer_pid urn for {location}"), None)
                }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::message_envelope::Direction;
    use axum::extract::Request;
    use chrono::Utc;
    use serde_json::Value;

    fn mate() -> Mates {
        let t = Utc::now().naive_utc();
        Mates {
            participant_id: "did:example:provider".into(),
            participant_type: "agent".into(),
            participant_nick: "provider".to_string(),
            base_url: "http://127.0.0.1:122".to_string(),
            token: None,
            saved_at: t.into(),
            last_interaction: t.into(),
            extra_fields: Value::Null,
            is_me: false,
        }
    }

    fn request_with_mate(body: &'static str) -> Request {
        let mut req = Request::builder()
            .method("POST")
            .uri("/transfers/123/start?x=1")
            .header("x-request-id", "req-1")
            .header("authorization", "Bearer eyJabc")
            .body(axum::body::Body::from(body))
            .unwrap();
        // The auth middleware would have inserted this before we run.
        req.extensions_mut().insert(mate());
        req
    }

    #[tokio::test]
    async fn from_request_reads_wire_fields_and_resolved_participant() {
        let raw = TransferContextRaw::<TransferDSPAuthn>::from_request(request_with_mate("{}"))
            .await
            .unwrap();
        assert_eq!(raw.request_id.as_str(), "req-1");
        assert_eq!(raw.request_path, "/transfers/123/start");
        assert_eq!(raw.request_full_path, "/transfers/123/start?x=1");
        assert_eq!(raw.authn.token_type, "Bearer");
        assert_eq!(raw.authn.token_content, "eyJabc");
        assert_eq!(
            raw.authn.associated_participant.participant_nick,
            "provider"
        );
        assert!(matches!(raw.direction, Direction::Inbound));
    }

    #[tokio::test]
    async fn from_request_fails_without_auth_middleware() {
        // No Mates in extensions → wiring error, not a silent None.
        let req = Request::builder().body(axum::body::Body::empty()).unwrap();
        assert!(
            TransferContextRaw::<TransferDSPAuthn>::from_request(req)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn canonical_hash_is_deterministic_and_key_order_independent() {
        // Real JSON-LD expansion resolves everything to full IRIs, so the same
        // TransferStartMessage with fields in different source order canonicalizes
        // to identical n-quads.
        let a = rdf_from(
            r#"{"@context":"https://w3id.org/dspace/2025/1/context.jsonld","@type":"TransferStartMessage","providerPid":"urn:uuid:pp","consumerPid":"urn:uuid:cc"}"#,
        )
        .await;
        let b = rdf_from(
            r#"{"@context":"https://w3id.org/dspace/2025/1/context.jsonld","consumerPid":"urn:uuid:cc","@type":"TransferStartMessage","providerPid":"urn:uuid:pp"}"#,
        )
        .await;
        assert!(
            !a.canonical_n_quads.is_empty(),
            "expansion must yield quads"
        );
        assert_eq!(
            a.canonical_hash, b.canonical_hash,
            "canonicalization must be key-order independent"
        );

        // Different content (provider pid) → different canonical hash.
        let c = rdf_from(
            r#"{"@context":"https://w3id.org/dspace/2025/1/context.jsonld","@type":"TransferStartMessage","providerPid":"urn:uuid:XX","consumerPid":"urn:uuid:cc"}"#,
        )
        .await;
        assert_ne!(
            a.canonical_hash, c.canonical_hash,
            "different content must hash differently"
        );
    }

    #[tokio::test]
    async fn extractor_pulls_pids_message_and_data_address() {
        let rdf = rdf_from(
            r#"{"@context":"https://w3id.org/dspace/2025/1/context.jsonld","@type":"TransferStartMessage","providerPid":"urn:uuid:pp","consumerPid":"urn:uuid:cc","dataAddress":{"@type":"DataAddress","endpointType":"HttpData","endpoint":"http://example.com/data","endpointProperties":[]}}"#,
        )
        .await;
        let typed = TransferDSPContextTyped::from_rdf(rdf).unwrap();
        assert_eq!(typed.provider_pid.as_deref(), Some("urn:uuid:pp"));
        assert_eq!(typed.consumer_pid.as_deref(), Some("urn:uuid:cc"));
        assert_eq!(
            typed.data_address.unwrap().endpoint,
            "http://example.com/data"
        );
        assert!(matches!(
            typed.message,
            TransferDSPMessageType::TransferStartMessage
        ));
    }

    #[tokio::test]
    async fn extractor_tolerates_missing_optional_fields() {
        // A message without dataAddress or pids extracts to None, not an error.
        let rdf = rdf_from(
            r#"{"@context":"https://w3id.org/dspace/2025/1/context.jsonld","@type":"TransferProcess"}"#,
        )
        .await;
        let typed = TransferDSPContextTyped::from_rdf(rdf).unwrap();
        assert!(typed.provider_pid.is_none());
        assert!(typed.data_address.is_none());
    }

    async fn rdf_from(body: &'static str) -> TransferDSPContextRdf {
        let raw = TransferContextRaw::<TransferDSPAuthn>::from_request(request_with_mate(body))
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&raw.body_bytes).unwrap();
        let parsed = TransferDSPContextParsed::from_raw(
            raw,
            DSPProtocolVersions::V2025_1,
            TransferDSPMessageType::TransferStartMessage,
            json,
        )
        .unwrap();
        TransferDSPContextRdf::from_parsed(parsed).await.unwrap()
    }
}
