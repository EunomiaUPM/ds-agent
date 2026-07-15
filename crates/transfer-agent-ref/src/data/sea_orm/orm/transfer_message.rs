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

use chrono::Utc;
use compact_str::CompactString;
use sea_orm::ActiveValue::Set;
use sea_orm::entity::prelude::*;
use ymir::errors::Outcome;

use crate::data::sea_orm::orm::helpers::{deser_enum, deser_json, parse_urn, ser_enum, ser_json};
use crate::entities::ids::{MessageId, TenantId};
use crate::entities::message_envelope::MessageEnvelope;
use crate::entities::protocol::{ProtocolId, ProtocolMessageType};
use crate::entities::transfer_message::{Direction, TransferMessage};

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
    pub state_transition_from: String,
    pub state_transition_to: String,
    pub envelope: Json,
    pub occurred_at: DateTimeWithTimeZone,
}

#[allow(clippy::result_large_err)]
impl Model {
    pub(crate) fn into_domain(self) -> Outcome<TransferMessage> {
        use crate::entities::ids::TransferProcessId;

        let id = MessageId::new(parse_urn(&self.id, "transfer_message.id")?);
        let transfer_process_id = TransferProcessId::new(parse_urn(
            &self.transfer_process_id,
            "transfer_message.transfer_process_id",
        )?);
        let tenant_id = self.tenant_id;
        let direction = deser_enum::<Direction>(&self.direction)?;
        let protocol = deser_enum::<ProtocolId>(&self.protocol)?;
        let message_type = ProtocolMessageType(CompactString::from(self.message_type));
        let envelope = deser_json::<MessageEnvelope>(self.envelope, "transfer_message.envelope")?;
        let occurred_at = self.occurred_at.with_timezone(&Utc);

        Ok(TransferMessage {
            id,
            transfer_process_id,
            tenant_id,
            direction,
            protocol,
            message_type,
            state_transition_from: self.state_transition_from,
            state_transition_to: self.state_transition_to,
            envelope,
            occurred_at,
        })
    }
}

impl ActiveModel {
    pub(crate) fn from_domain(msg: &TransferMessage) -> Self {
        Self {
            id: Set(msg.id().to_string()),
            transfer_process_id: Set(msg.transfer_process_id().to_string()),
            tenant_id: Set(msg.tenant_id().as_str().to_string()),
            direction: Set(ser_enum(&msg.direction())),
            protocol: Set(ser_enum(msg.protocol())),
            message_type: Set(msg.message_type().0.to_string()),
            state_transition_from: Set(msg.state_transition_from().to_string()),
            state_transition_to: Set(msg.state_transition_to().to_string()),
            envelope: Set(ser_json(msg.envelope())),
            occurred_at: Set(msg.occurred_at().into()),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
