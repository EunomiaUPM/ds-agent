use std::collections::BTreeMap;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use compact_str::CompactString;
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
        matches!(self.processing_result, MessageProcessingResult::Accepted { .. })
    }

    pub fn was_rejected(&self) -> bool {
        matches!(self.processing_result, MessageProcessingResult::Rejected { .. })
    }

    pub fn is_replay(&self) -> bool {
        matches!(self.processing_result, MessageProcessingResult::IdempotentReplay)
    }

    /// State the process was in before this message was processed.
    pub fn resulting_state(&self) -> Option<&ProtocolState> {
        match &self.processing_result {
            MessageProcessingResult::Accepted { resulting_state } => Some(resulting_state),
            _ => None,
        }
    }
}

// ── Wire envelope ─────────────────────────────────────────────────────────────

pub(crate) struct MessageEnvelope {
    pub raw_bytes: Bytes,
    pub content_type: CompactString,
    /// SHA-256 of raw_bytes. Used for deduplication and idempotency checks.
    pub content_hash: [u8; 32],
    pub headers: BTreeMap<CompactString, String>,
    /// URDNA2015 canonical form for DSP JSON-LD messages; None for non-RDF protocols.
    pub canonical_form: Option<Bytes>,
    pub canonical_hash: Option<[u8; 32]>,
    /// Signature over canonical_hash when the protocol requires message signing.
    pub signature: Option<MessageSignature>,
}

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

pub(crate) struct MessageSignature {
    /// Signing algorithm: "EdDSA", "ES256", etc.
    pub algorithm: CompactString,
    /// JWK key ID or signer URN.
    pub key_id: CompactString,
    pub value: Bytes,
}

// ── Processing result ─────────────────────────────────────────────────────────

pub(crate) enum MessageProcessingResult {
    Accepted { resulting_state: ProtocolState },
    Rejected { reason: String, error_code: Option<String> },
    IdempotentReplay,
}