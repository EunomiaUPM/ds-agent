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

use std::collections::BTreeMap;
use std::str::FromStr;

use base64::Engine;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use compact_str::CompactString;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};
use thiserror::Error;
use urn::Urn;

use crate::entities::commands::NewTransferMessageCommand;
use crate::entities::ids::{
    CorrelationId, MessageId, ParticipantId, RequestId, TenantId, TransferProcessId,
};
use crate::entities::message_envelope::Direction;
use crate::entities::protocol::{ProtocolId, ProtocolMessageType, ProtocolState};
use ymir::errors::{Errors, Outcome};

#[derive(Clone)]
pub(crate) struct TransferMessage {
    pub(crate) id: MessageId,
    pub(crate) transfer_process_id: TransferProcessId,
    pub(crate) tenant_id: TenantId,
    pub(crate) direction: Direction,

    // Protocol
    pub(crate) protocol: ProtocolId,
    pub(crate) message_type: ProtocolMessageType,
    pub(crate) protocol_version: CompactString,

    // Wire
    pub(crate) envelope: MessageEnvelope,

    // Traceability
    pub(crate) occurred_at: DateTime<Utc>,
    pub(crate) correlation_id: Option<CorrelationId>,
    pub(crate) request_id: RequestId,
    pub(crate) peer_participant_id: ParticipantId,

    pub(crate) processing_result: MessageProcessingResult,
}

#[allow(dead_code, clippy::result_large_err)]
impl TransferMessage {
    pub(crate) fn from_cmd(cmd: &NewTransferMessageCommand) -> Outcome<Self> {
        let id = cmd.id.clone().unwrap_or_else(MessageId::generate);
        let peer_participant_id = cmd.peer_participant_id.clone().unwrap_or_else(|| {
            ParticipantId::new(Urn::from_str("urn:unknown:participant").expect("static URN"))
        });
        let request_id = cmd.request_id.clone().unwrap_or_else(RequestId::generate);
        let protocol_version =
            CompactString::from(cmd.protocol_version.as_deref().unwrap_or("1.0"));
        let tenant_id = cmd
            .tenant_id
            .clone()
            .ok_or_else(|| Errors::crazy("tenant_id must be resolved before reaching the domain", None))?;
        Ok(Self {
            id,
            transfer_process_id: cmd.transfer_process_id.clone(),
            tenant_id,
            direction: cmd.direction,
            protocol: cmd.protocol.clone(),
            message_type: cmd.message_type.clone(),
            protocol_version,
            envelope: cmd.envelope.clone(),
            occurred_at: Utc::now(),
            correlation_id: cmd.correlation_id.clone(),
            request_id,
            peer_participant_id,
            processing_result: MessageProcessingResult::Accepted {
                resulting_state: cmd.state_transition_to.clone(),
            },
        })
    }

    pub fn id(&self) -> &MessageId {
        &self.id
    }
    pub fn transfer_process_id(&self) -> &TransferProcessId {
        &self.transfer_process_id
    }
    pub fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }
    pub fn direction(&self) -> Direction {
        self.direction
    }
    pub fn protocol(&self) -> &ProtocolId {
        &self.protocol
    }
    pub fn message_type(&self) -> &ProtocolMessageType {
        &self.message_type
    }
    pub fn protocol_version(&self) -> &str {
        &self.protocol_version
    }
    pub fn envelope(&self) -> &MessageEnvelope {
        &self.envelope
    }
    pub fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
    pub fn correlation_id(&self) -> Option<&CorrelationId> {
        self.correlation_id.as_ref()
    }
    pub fn request_id(&self) -> &RequestId {
        &self.request_id
    }
    pub fn peer_participant_id(&self) -> &ParticipantId {
        &self.peer_participant_id
    }
    pub fn processing_result(&self) -> &MessageProcessingResult {
        &self.processing_result
    }

    pub fn is_inbound(&self) -> bool {
        self.direction == Direction::Inbound
    }
    pub fn is_outbound(&self) -> bool {
        self.direction == Direction::Outbound
    }
    pub fn was_accepted(&self) -> bool {
        matches!(
            self.processing_result,
            MessageProcessingResult::Accepted { .. }
        )
    }
    pub fn was_rejected(&self) -> bool {
        matches!(
            self.processing_result,
            MessageProcessingResult::Rejected { .. }
        )
    }
    pub fn is_replay(&self) -> bool {
        matches!(
            self.processing_result,
            MessageProcessingResult::IdempotentReplay
        )
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

#[derive(Clone, Serialize, Deserialize)]
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

#[derive(Debug, Error)]
pub(crate) enum EnvelopeError {
    #[error("invalid base64 in field '{field}': {source}")]
    InvalidBase64 {
        field: &'static str,
        source: base64::DecodeError,
    },
}

impl TryFrom<MessageEnvelopeInput> for MessageEnvelope {
    type Error = EnvelopeError;

    fn try_from(input: MessageEnvelopeInput) -> Result<Self, Self::Error> {
        let raw = b64_decode(&input.raw_bytes, "rawBytes")?;
        let content_hash: [u8; 32] = Sha256::digest(&raw).into();

        let (canonical_form, canonical_hash) = match input.canonical_form {
            Some(cf) => {
                let cf_raw = b64_decode(&cf, "canonicalForm")?;
                let cf_hash: [u8; 32] = Sha256::digest(&cf_raw).into();
                (Some(Bytes::from(cf_raw)), Some(cf_hash))
            }
            None => (None, None),
        };

        let signature = input
            .signature
            .map(|s| -> Result<MessageSignature, EnvelopeError> {
                let value = Bytes::from(b64_decode(&s.value, "signature.value")?);
                Ok(MessageSignature {
                    algorithm: s.algorithm,
                    key_id: s.key_id,
                    value,
                })
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

#[allow(dead_code)]
impl MessageEnvelope {
    pub fn is_signed(&self) -> bool {
        self.signature.is_some()
    }
    pub fn is_canonicalized(&self) -> bool {
        self.canonical_form.is_some()
    }
    pub fn content_type(&self) -> &str {
        &self.content_type
    }
}

#[derive(Clone, Serialize, Deserialize)]
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

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub(crate) enum MessageProcessingResult {
    Accepted {
        resulting_state: ProtocolState,
    },
    Rejected {
        reason: String,
        error_code: Option<String>,
    },
    IdempotentReplay,
}

// Serde helpers ─────────────────────────────────────────────────────────────

fn b64_decode(s: &str, field: &'static str) -> Result<Vec<u8>, EnvelopeError> {
    base64::engine::general_purpose::STANDARD
        .decode(s)
        .map_err(|source| EnvelopeError::InvalidBase64 { field, source })
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

fn bytes_to_hex(h: &[u8; 32]) -> String {
    use std::fmt::Write;
    let mut buf = String::with_capacity(64);
    for b in h {
        write!(buf, "{b:02x}").unwrap();
    }
    buf
}

fn ser_hash_hex<S: Serializer>(h: &[u8; 32], s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&bytes_to_hex(h))
}

fn ser_opt_hash_hex<S: Serializer>(h: &Option<[u8; 32]>, s: S) -> Result<S::Ok, S::Error> {
    match h {
        Some(v) => s.serialize_some(&bytes_to_hex(v)),
        None => s.serialize_none(),
    }
}

fn de_b64_bytes<'de, D: Deserializer<'de>>(d: D) -> Result<Bytes, D::Error> {
    let s = String::deserialize(d)?;
    b64_decode(&s, "bytes")
        .map(Bytes::from)
        .map_err(serde::de::Error::custom)
}
