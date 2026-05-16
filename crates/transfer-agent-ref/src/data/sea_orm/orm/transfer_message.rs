use chrono::Utc;
use compact_str::CompactString;
use sea_orm::ActiveValue::Set;
use sea_orm::entity::prelude::*;
use ymir::errors::Outcome;

use crate::data::sea_orm::orm::helpers::{deser_enum, deser_json, parse_urn, ser_enum, ser_json};
use crate::entities::ids::{CorrelationId, MessageId, ParticipantId, RequestId, TenantId};
use crate::entities::message_envelope::Direction;
use crate::entities::protocol::{ProtocolId, ProtocolMessageType};
use crate::entities::transfer_message::{
    MessageEnvelope, MessageProcessingResult, TransferMessage,
};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "transfer_messages")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub transfer_process_id: String,
    pub tenant_id: String,
    pub direction: String,
    pub protocol: String,
    pub message_type: String,
    pub protocol_version: String,
    pub envelope: Json,
    pub occurred_at: DateTimeWithTimeZone,
    pub correlation_id: Option<String>,
    pub request_id: String,
    pub peer_participant_id: String,
    pub processing_result: Json,
    /// Denormalized from processing_result.resulting_state for efficient filtering.
    pub state_transition_to: Option<String>,
}

impl Model {
    pub(crate) fn into_domain(self) -> Outcome<TransferMessage> {
        use crate::entities::ids::TransferProcessId;

        let id = MessageId::new(parse_urn(&self.id, "transfer_message.id")?);
        let transfer_process_id = TransferProcessId::new(parse_urn(
            &self.transfer_process_id,
            "transfer_message.transfer_process_id",
        )?);
        let tenant_id = TenantId::new(self.tenant_id);
        let direction = deser_enum::<Direction>(&self.direction)?;
        let protocol = deser_enum::<ProtocolId>(&self.protocol)?;
        let message_type = ProtocolMessageType(CompactString::from(self.message_type));
        let protocol_version = CompactString::from(self.protocol_version);
        let envelope = deser_json::<MessageEnvelope>(self.envelope, "transfer_message.envelope")?;
        let occurred_at = self.occurred_at.with_timezone(&Utc);
        let correlation_id = self.correlation_id.map(CorrelationId::new);
        let request_id = RequestId::new(self.request_id);
        let peer_participant_id = ParticipantId::new(parse_urn(
            &self.peer_participant_id,
            "transfer_message.peer_participant_id",
        )?);
        let processing_result = deser_json::<MessageProcessingResult>(
            self.processing_result,
            "transfer_message.processing_result",
        )?;

        Ok(TransferMessage {
            id,
            transfer_process_id,
            tenant_id,
            direction,
            protocol,
            message_type,
            protocol_version,
            envelope,
            occurred_at,
            correlation_id,
            request_id,
            peer_participant_id,
            processing_result,
        })
    }
}

impl ActiveModel {
    pub(crate) fn from_domain(msg: &TransferMessage) -> Self {
        let state_transition_to = msg.resulting_state().map(|s| s.0.to_string());
        Self {
            id: Set(msg.id().to_string()),
            transfer_process_id: Set(msg.transfer_process_id().to_string()),
            tenant_id: Set(msg.tenant_id().as_str().to_string()),
            direction: Set(ser_enum(&msg.direction())),
            protocol: Set(ser_enum(msg.protocol())),
            message_type: Set(msg.message_type().0.to_string()),
            protocol_version: Set(msg.protocol_version().to_string()),
            envelope: Set(ser_json(msg.envelope())),
            occurred_at: Set(msg.occurred_at().into()),
            correlation_id: Set(msg.correlation_id().map(|c| c.as_str().to_string())),
            request_id: Set(msg.request_id().as_str().to_string()),
            peer_participant_id: Set(msg.peer_participant_id().to_string()),
            processing_result: Set(ser_json(msg.processing_result())),
            state_transition_to: Set(state_transition_to),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
