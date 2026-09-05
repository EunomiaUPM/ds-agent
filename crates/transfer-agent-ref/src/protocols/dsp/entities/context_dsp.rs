/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

//! The inbound DSP context, stage by stage: raw -> parsed -> rdf -> typed ->
//! domain. Each stage consumes the previous one, so the order cannot be skipped.

use crate::entities::ids::IdempotencyKey;
use crate::entities::protocol::{ProtocolId, TransferDirection, TransferRole};
use crate::protocols::dsp::entities::auth::TransferDSPAuthn;
use crate::protocols::dsp::entities::context_common::{BuildAuthn, TransferContextRaw};
use crate::protocols::dsp::entities::context_common::{
    TransferContextConnectorRole, TransferContextProcessSlot,
};
use crate::protocols::dsp::entities::message_types::TransferDSPMessageType;
use crate::protocols::dsp::entities::protocol_fields::TransferProtocolFields;
use crate::protocols::dsp::entities::rdf_extractor_dsp::DspTransfer;
use common::dsp_common::data_address::DataAddress;
use common::dsp_common::odrl::OdrlAgreement;
use common::dsp_common::well_known_types::DSPProtocolVersions;
use common::rdf::dsp::DspCanonicalizer;
use common::rdf::expanded::ExpandedDoc;
use common::rdf::extract::ExtractProtocolFields;
use http::request::Parts;
use sha2::{Digest, Sha256};
use std::str::FromStr;
use urn::Urn;
use ymir::data::entities::shared::participant::Model as Mates;
use ymir::errors::{BadFormat, Errors, Outcome};

// TransferContextRaw --
impl BuildAuthn for TransferDSPAuthn {
    fn from_request_parts(parts: &Parts) -> Outcome<Self> {
        let associated_participant = parts.extensions.get::<Mates>().cloned().ok_or_else(|| {
            Errors::crazy(
                "auth middleware did not resolve participant (Mates missing)",
                None,
            )
        })?;
        let raw = Self::header(&parts.headers, "authorization").unwrap_or_default();
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
    pub dsp_version: ProtocolId,
    pub dsp_message_type: TransferDSPMessageType,
    pub json_value: serde_json::Value,
}

impl TransferDSPContextParsed {
    /// Wrap the raw context with what the route already settled: protocol version
    /// and the message type the endpoint handles.
    pub fn from_raw(
        raw: TransferContextRaw<TransferDSPAuthn>,
        dsp_version: &ProtocolId,
        dsp_message_type: &TransferDSPMessageType,
        json_value: serde_json::Value,
    ) -> Outcome<Self> {
        Ok(Self {
            raw,
            dsp_version: dsp_version.clone(),
            dsp_message_type: dsp_message_type.clone(),
            json_value,
        })
    }
}

// TransferDSPContextRdf --

#[derive(Debug)]
pub struct TransferDSPContextRdf {
    pub parsed: TransferDSPContextParsed,
    /// The form extraction and payload validation read: invariant under the
    /// aliasing a peer is free to choose.
    pub expanded: serde_json::Value,
    /// URDNA2015 / RDFC-1.0 canonical n-quads of the expanded message.
    pub canonical_n_quads: String,
    pub canonical_hash: [u8; 32],
}

impl TransferDSPContextRdf {
    /// Expand once, keeping both products. `canonical_hash` is a semantic identity,
    /// not an attestation of the bytes — for that see [`TransferContextRaw::wire_hash`].
    pub async fn from_parsed(parsed: TransferDSPContextParsed) -> Outcome<Self> {
        let expansion = DspCanonicalizer::new(parsed.json_value.clone())
            .expand_once()
            .await?;
        let canonical_hash = Sha256::digest(expansion.canonical_n_quads.as_bytes()).into();
        Ok(Self {
            parsed,
            expanded: expansion.expanded,
            canonical_n_quads: expansion.canonical_n_quads,
            canonical_hash,
        })
    }
}

// TransferDSPContextTyped --

#[derive(Debug)]
pub struct TransferDSPContextTyped {
    pub rdf: TransferDSPContextRdf,
    /// What the **body** declares. The route's own type stays in
    /// `rdf.parsed.dsp_message_type`, for the manager to check against this.
    pub message: TransferDSPMessageType,
    /// The DSP 9.2 fields, as extracted. Passed on to the domain by itself.
    pub fields: TransferProtocolFields,
    /// Always present: derived from protocol identity, with the peer's header
    /// folded in when it sent one. See [`IdempotencyKey::derive`].
    pub idempotency_key: IdempotencyKey,
}

impl TransferDSPContextTyped {
    /// Build the typed context from the RDF stage. Extraction only: the message
    /// type comes from the body, and agreeing with the route is the manager's call.
    pub fn from_rdf(rdf: TransferDSPContextRdf) -> Outcome<Self> {
        let (message, fields) = {
            let doc = ExpandedDoc::new(&rdf.expanded).ok_or_else(|| {
                Errors::format(
                    BadFormat::Received,
                    "expanded JSON-LD is not an array of node objects",
                    None,
                )
            })?;
            let (message, node) = DspTransfer::root_message(&doc)?;
            (message, DspTransfer::extract(&node)?)
        };

        let idempotency_key = IdempotencyKey::derive(
            &rdf.parsed.raw,
            &rdf.parsed.dsp_version,
            &message,
            fields.consumer_pid.as_deref(),
            fields.provider_pid.as_deref(),
        );

        Ok(TransferDSPContextTyped {
            rdf,
            message,
            fields,
            idempotency_key,
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
    use crate::entities::transfer_message::Direction;
    use axum::extract::Request;
    use chrono::Utc;
    use serde_json::Value;
    use ymir::types::participants::ParticipantType;

    fn mate() -> Mates {
        let t = Utc::now();
        Mates {
            participant_id: "did:example:provider".into(),
            participant_type: ParticipantType::Agent,
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
        assert_eq!(typed.fields.provider_pid.as_deref(), Some("urn:uuid:pp"));
        assert_eq!(typed.fields.consumer_pid.as_deref(), Some("urn:uuid:cc"));
        assert!(
            !typed.idempotency_key.as_str().is_empty(),
            "the effective key is always derived, never left empty"
        );
        assert_eq!(
            typed.fields.data_address.unwrap().endpoint.as_deref(),
            Some("http://example.com/data")
        );
        assert!(matches!(
            typed.message,
            TransferDSPMessageType::TransferStartMessage
        ));
    }

    /// A `TransferRequestMessage` carries `agreementId`, `callbackAddress` and
    /// `format`; the extractor could already read them but nothing surfaced them.
    #[tokio::test]
    async fn typed_carries_the_request_message_fields() {
        let rdf = rdf_from_typed(
            r#"{"@context":"https://w3id.org/dspace/2025/1/context.jsonld","@type":"TransferRequestMessage","consumerPid":"urn:uuid:cc","agreementId":"urn:uuid:ag","callbackAddress":"https://example.com/callback","format":"example:HTTP_PUSH"}"#,
            TransferDSPMessageType::TransferRequestMessage,
        )
        .await;
        let typed = TransferDSPContextTyped::from_rdf(rdf).unwrap();
        assert_eq!(typed.fields.agreement_id.as_deref(), Some("urn:uuid:ag"));
        assert_eq!(
            typed.fields.callback_address.as_deref(),
            Some("https://example.com/callback")
        );
        assert_eq!(typed.fields.format.as_deref(), Some("example:HTTP_PUSH"));
    }

    /// The same message as one nested object and as an `@graph` of
    /// mutually-referencing nodes. Both expand to the same RDF graph — the
    /// canonicalizer gives them the same hash — so both must extract the same
    /// fields. Requiring the expansion to be a single node rejected the second.
    #[tokio::test]
    async fn graph_form_extracts_the_same_as_the_nested_form() {
        const NESTED: &str = r#"{
            "@context": "https://w3id.org/dspace/2025/1/context.jsonld",
            "@type": "TransferRequestMessage",
            "consumerPid": "urn:uuid:cc",
            "agreementId": "urn:uuid:ag",
            "format": "example:HTTP_PUSH",
            "callbackAddress": "https://example.com/callback",
            "dataAddress": {
                "@type": "DataAddress",
                "endpointType": "https://w3id.org/idsa/v4.1/HTTP",
                "endpoint": "http://example.com",
                "endpointProperties": [
                    {"@type": "EndpointProperty", "name": "authorization", "value": "TOKEN-ABCDEFG"},
                    {"@type": "EndpointProperty", "name": "authType", "value": "bearer"}
                ]
            }
        }"#;
        const GRAPH: &str = r#"{
            "@context": "https://w3id.org/dspace/2025/1/context.jsonld",
            "@graph": [
                {"@id": "_:msg", "@type": "TransferRequestMessage",
                 "consumerPid": "urn:uuid:cc",
                 "agreementId": "urn:uuid:ag",
                 "format": "example:HTTP_PUSH",
                 "callbackAddress": "https://example.com/callback",
                 "dataAddress": {"@id": "_:addr"}},
                {"@id": "_:addr", "@type": "DataAddress",
                 "endpointType": "https://w3id.org/idsa/v4.1/HTTP",
                 "endpoint": "http://example.com",
                 "endpointProperties": [{"@id": "_:p1"}, {"@id": "_:p2"}]},
                {"@id": "_:p1", "@type": "EndpointProperty", "name": "authorization", "value": "TOKEN-ABCDEFG"},
                {"@id": "_:p2", "@type": "EndpointProperty", "name": "authType", "value": "bearer"}
            ]
        }"#;

        let nested = rdf_from_typed(NESTED, TransferDSPMessageType::TransferRequestMessage).await;
        let graph = rdf_from_typed(GRAPH, TransferDSPMessageType::TransferRequestMessage).await;
        assert_eq!(
            nested.canonical_hash, graph.canonical_hash,
            "the two forms are the same graph"
        );

        let nested = TransferDSPContextTyped::from_rdf(nested).unwrap();
        let graph = TransferDSPContextTyped::from_rdf(graph).unwrap();

        assert_eq!(graph.fields, nested.fields, "both forms extract the same");
        assert_eq!(graph.idempotency_key, nested.idempotency_key);

        // The dataAddress arrives as a bare `{"@id": …}` reference in the graph
        // form, so it only resolves if references are followed.
        let address = graph.fields.data_address.expect("dataAddress must resolve");
        assert_eq!(address.endpoint.as_deref(), Some("http://example.com"));
        assert_eq!(address.endpoint_type, "https://w3id.org/idsa/v4.1/HTTP");
        let mut properties: Vec<(String, String)> = address
            .endpoint_properties
            .iter()
            .map(|p| (p.name.clone(), p.value.clone()))
            .collect();
        properties.sort();
        assert_eq!(
            properties,
            vec![
                ("authType".to_string(), "bearer".to_string()),
                ("authorization".to_string(), "TOKEN-ABCDEFG".to_string()),
            ],
            "endpointProperties are references too"
        );
    }

    /// The body's `@type` wins over the route's: extraction reports what arrived,
    /// and disagreeing with the endpoint is the manager's to reject.
    #[tokio::test]
    async fn the_body_type_is_read_even_when_the_route_disagrees() {
        let rdf = rdf_from_typed(
            r#"{"@context":"https://w3id.org/dspace/2025/1/context.jsonld","@type":"TransferProcess","consumerPid":"urn:uuid:cc"}"#,
            TransferDSPMessageType::TransferRequestMessage,
        )
        .await;
        let typed = TransferDSPContextTyped::from_rdf(rdf).unwrap();
        assert_eq!(typed.message, TransferDSPMessageType::TransferProcess);
        assert_eq!(
            typed.rdf.parsed.dsp_message_type,
            TransferDSPMessageType::TransferRequestMessage,
            "the route's own type stays available to compare against"
        );
        assert_eq!(typed.fields.consumer_pid.as_deref(), Some("urn:uuid:cc"));
    }

    /// The one thing extraction still cannot do: build fields out of a body that
    /// declares no DSP message type at all.
    #[tokio::test]
    async fn a_body_declaring_no_message_type_is_rejected() {
        let rdf = rdf_from(
            r#"{"@context":"https://w3id.org/dspace/2025/1/context.jsonld","@type":"DataAddress","endpointType":"https://w3id.org/idsa/v4.1/HTTP"}"#,
        )
        .await;
        assert!(TransferDSPContextTyped::from_rdf(rdf).is_err());
    }

    #[tokio::test]
    async fn extractor_tolerates_missing_optional_fields() {
        // A message without dataAddress or pids extracts to None, not an error.
        let rdf =
            rdf_from(r#"{"@context":"https://w3id.org/dspace/2025/1/context.jsonld","@type":"TransferStartMessage"}"#).await;
        let typed = TransferDSPContextTyped::from_rdf(rdf).unwrap();
        assert!(typed.fields.provider_pid.is_none());
        assert!(typed.fields.data_address.is_none());
    }

    async fn rdf_from_typed(
        body: &'static str,
        message_type: TransferDSPMessageType,
    ) -> TransferDSPContextRdf {
        let raw = TransferContextRaw::<TransferDSPAuthn>::from_request(request_with_mate(body))
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&raw.body_bytes).unwrap();
        let parsed =
            TransferDSPContextParsed::from_raw(raw, &ProtocolId::Dsp2025_1, &message_type, json)
                .unwrap();
        TransferDSPContextRdf::from_parsed(parsed).await.unwrap()
    }

    async fn rdf_from(body: &'static str) -> TransferDSPContextRdf {
        let raw = TransferContextRaw::<TransferDSPAuthn>::from_request(request_with_mate(body))
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&raw.body_bytes).unwrap();
        let parsed = TransferDSPContextParsed::from_raw(
            raw,
            &ProtocolId::Dsp2025_1,
            &TransferDSPMessageType::TransferStartMessage,
            json,
        )
        .unwrap();
        TransferDSPContextRdf::from_parsed(parsed).await.unwrap()
    }
}
