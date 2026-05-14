use std::collections::BTreeMap;
use base64::Engine;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use compact_str::CompactString;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};
use crate::entities::ids::{CorrelationId, MessageId, ParticipantId, RequestId, TenantId, TransferProcessId};
use crate::entities::message_envelope::Direction;
use crate::entities::protocol::{ProtocolId, ProtocolMessageType, ProtocolState};

pub(crate) struct TransferMessage {
    pub id: MessageId,
    pub transfer_process_id: TransferProcessId,
    pub tenant_id: TenantId,
    pub direction: Direction,

    // Protocol
    pub protocol: ProtocolId,
    pub message_type: ProtocolMessageType,
    pub protocol_version: CompactString,

    // Wire
    pub envelope: MessageEnvelope,

    // Traceability
    pub occurred_at: DateTime<Utc>,
    pub correlation_id: Option<CorrelationId>,
    pub request_id: RequestId,
    pub peer_participant_id: ParticipantId,

    pub processing_result: MessageProcessingResult,
}

impl TransferMessage {
    pub fn id(&self) -> &MessageId { &self.id }
    pub fn transfer_process_id(&self) -> &TransferProcessId { &self.transfer_process_id }
    pub fn tenant_id(&self) -> &TenantId { &self.tenant_id }
    pub fn direction(&self) -> Direction { self.direction }
    pub fn protocol(&self) -> &ProtocolId { &self.protocol }
    pub fn message_type(&self) -> &ProtocolMessageType { &self.message_type }
    pub fn protocol_version(&self) -> &str { &self.protocol_version }
    pub fn envelope(&self) -> &MessageEnvelope { &self.envelope }
    pub fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    pub fn correlation_id(&self) -> Option<&CorrelationId> { self.correlation_id.as_ref() }
    pub fn request_id(&self) -> &RequestId { &self.request_id }
    pub fn peer_participant_id(&self) -> &ParticipantId { &self.peer_participant_id }
    pub fn processing_result(&self) -> &MessageProcessingResult { &self.processing_result }

    pub fn is_inbound(&self) -> bool { self.direction == Direction::Inbound }
    pub fn is_outbound(&self) -> bool { self.direction == Direction::Outbound }
    pub fn was_accepted(&self) -> bool {
        matches!(self.processing_result, MessageProcessingResult::Accepted { .. })
    }
    pub fn was_rejected(&self) -> bool {
        matches!(self.processing_result, MessageProcessingResult::Rejected { .. })
    }
    pub fn is_replay(&self) -> bool {
        matches!(self.processing_result, MessageProcessingResult::IdempotentReplay)
    }

    /// State the process transitioned to after this message was accepted.
    pub fn resulting_state(&self) -> Option<&ProtocolState> {
        match &self.processing_result {
            MessageProcessingResult::Accepted { resulting_state } => Some(resulting_state),
            _ => None,
        }
    }
}

// Wire envelope ─────────────────────────────────────────────────────────────

/// Deserialization input: raw_bytes as base64; content_hash is computed server-side.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MessageEnvelopeInput {
    raw_bytes: String,
    content_type: CompactString,
    #[serde(default)]
    headers: BTreeMap<CompactString, String>,
    canonical_form: Option<String>,
    signature: Option<MessageSignatureInput>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MessageSignatureInput {
    algorithm: CompactString,
    key_id: CompactString,
    value: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "MessageEnvelopeInput")]
pub(crate) struct MessageEnvelope {
    #[serde(serialize_with = "ser_bytes_b64")]
    pub raw_bytes: Bytes,
    pub content_type: CompactString,
    /// SHA-256 of raw_bytes. Used for deduplication and idempotency checks.
    #[serde(serialize_with = "ser_hash_hex")]
    pub content_hash: [u8; 32],
    pub headers: BTreeMap<CompactString, String>,
    /// URDNA2015 canonical form for DSP JSON-LD messages; None for non-RDF protocols.
    #[serde(serialize_with = "ser_opt_bytes_b64")]
    pub canonical_form: Option<Bytes>,
    #[serde(serialize_with = "ser_opt_hash_hex")]
    pub canonical_hash: Option<[u8; 32]>,
    /// Signature over canonical_hash when the protocol requires message signing.
    pub signature: Option<MessageSignature>,
}

impl TryFrom<MessageEnvelopeInput> for MessageEnvelope {
    type Error = String;

    fn try_from(input: MessageEnvelopeInput) -> Result<Self, Self::Error> {
        let raw = b64_decode(&input.raw_bytes)?;
        let content_hash: [u8; 32] = Sha256::digest(&raw).into();

        let (canonical_form, canonical_hash) = match input.canonical_form {
            Some(cf) => {
                let cf_raw = b64_decode(&cf)?;
                let cf_hash: [u8; 32] = Sha256::digest(&cf_raw).into();
                (Some(Bytes::from(cf_raw)), Some(cf_hash))
            }
            None => (None, None),
        };

        let signature = input
            .signature
            .map(|s| -> Result<MessageSignature, String> {
                let value = Bytes::from(b64_decode(&s.value)?);
                Ok(MessageSignature { algorithm: s.algorithm, key_id: s.key_id, value })
            })
            .transpose()?;

        Ok(Self {
            raw_bytes: Bytes::from(raw),
            content_type: input.content_type,
            content_hash,
            headers: input.headers,
            canonical_form,
            canonical_hash,
            signature,
        })
    }
}

impl MessageEnvelope {
    pub fn is_signed(&self) -> bool { self.signature.is_some() }
    pub fn is_canonicalized(&self) -> bool { self.canonical_form.is_some() }
    pub fn content_type(&self) -> &str { &self.content_type }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MessageSignature {
    /// Signing algorithm: "EdDSA", "ES256", etc.
    pub algorithm: CompactString,
    /// JWK key ID or signer URN.
    pub key_id: CompactString,
    #[serde(serialize_with = "ser_bytes_b64", deserialize_with = "de_b64_bytes")]
    pub value: Bytes,
}

// Processing result ─────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub(crate) enum MessageProcessingResult {
    Accepted { resulting_state: ProtocolState },
    Rejected { reason: String, error_code: Option<String> },
    IdempotentReplay,
}

// Serde helpers ─────────────────────────────────────────────────────────────

fn b64_decode(s: &str) -> Result<Vec<u8>, String> {
    base64::engine::general_purpose::STANDARD.decode(s).map_err(|e| e.to_string())
}

fn ser_bytes_b64<S: Serializer>(b: &Bytes, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&base64::engine::general_purpose::STANDARD.encode(b))
}

fn ser_opt_bytes_b64<S: Serializer>(b: &Option<Bytes>, s: S) -> Result<S::Ok, S::Error> {
    match b {
        Some(v) => s.serialize_some(&base64::engine::general_purpose::STANDARD.encode(v)),
        None => s.serialize_none(),
    }
}

fn ser_hash_hex<S: Serializer>(h: &[u8; 32], s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&h.iter().map(|b| format!("{b:02x}")).collect::<String>())
}

fn ser_opt_hash_hex<S: Serializer>(h: &Option<[u8; 32]>, s: S) -> Result<S::Ok, S::Error> {
    match h {
        Some(v) => s.serialize_some(&v.iter().map(|b| format!("{b:02x}")).collect::<String>()),
        None => s.serialize_none(),
    }
}

fn de_b64_bytes<'de, D: Deserializer<'de>>(d: D) -> Result<Bytes, D::Error> {
    let s = String::deserialize(d)?;
    b64_decode(&s).map(Bytes::from).map_err(serde::de::Error::custom)
}