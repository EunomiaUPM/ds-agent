use chrono::{DateTime, Utc};
use serde::Serialize;
use crate::entities::ids::{CorrelationId, MessageId, ParticipantId, RequestId, TenantId, TransferProcessId};
use crate::entities::message_envelope::Direction;
use crate::entities::protocol::{ProtocolId, ProtocolMessageType};
use crate::entities::transfer_message::{MessageEnvelope, MessageProcessingResult, TransferMessage};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TransferMessageView {
    pub id: MessageId,
    pub transfer_process_id: TransferProcessId,
    pub tenant_id: TenantId,
    pub direction: Direction,
    pub protocol: ProtocolId,
    pub message_type: ProtocolMessageType,
    pub protocol_version: String,
    pub envelope: MessageEnvelope,
    pub occurred_at: DateTime<Utc>,
    pub correlation_id: Option<CorrelationId>,
    pub request_id: RequestId,
    pub peer_participant_id: ParticipantId,
    pub processing_result: MessageProcessingResult,
}

impl TransferMessageView {
    pub(crate) fn assemble(msg: TransferMessage) -> Self {
        Self {
            id: msg.id,
            transfer_process_id: msg.transfer_process_id,
            tenant_id: msg.tenant_id,
            direction: msg.direction,
            protocol: msg.protocol,
            message_type: msg.message_type,
            protocol_version: msg.protocol_version.to_string(),
            envelope: msg.envelope,
            occurred_at: msg.occurred_at,
            correlation_id: msg.correlation_id,
            request_id: msg.request_id,
            peer_participant_id: msg.peer_participant_id,
            processing_result: msg.processing_result,
        }
    }
}