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
use common::oauth::{Owner, RolePath, Visibility};
use compact_str::CompactString;
use sea_orm::ActiveValue::Set;
use sea_orm::entity::prelude::*;
use ymir::errors::Outcome;

use crate::data::sea_orm::orm::helpers::{deser_enum, deser_json, ser_enum, ser_json};
use crate::entities::ids::MessageId;
use crate::entities::message_envelope::MessageEnvelope;
use crate::entities::protocol::{ProtocolId, ProtocolMessageType};
use crate::entities::transfer_message::{Direction, TransferMessage};
use common::utils::parse_urn;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "transfer_messages")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub transfer_process_id: String,
    pub user_id: String,
    pub user_role: RolePath,
    pub visibility: Visibility,
    pub direction: String,
    pub protocol: String,
    pub message_type: String,
    pub state_transition_from: String,
    pub state_transition_to: String,
    pub envelope: Json,
    pub occurred_at: DateTimeWithTimeZone,
}

impl Model {
    pub fn into_domain(self) -> Outcome<TransferMessage> {
        use crate::entities::ids::TransferProcessId;

        let id = MessageId::new(parse_urn(&self.id)?);
        let transfer_process_id = TransferProcessId::new(parse_urn(&self.transfer_process_id)?);
        let owner = Owner::new(self.user_id, self.user_role, self.visibility);
        let direction = deser_enum::<Direction>(&self.direction)?;
        let protocol = deser_enum::<ProtocolId>(&self.protocol)?;
        let message_type = ProtocolMessageType(CompactString::from(self.message_type));
        let envelope = deser_json::<MessageEnvelope>(self.envelope, "transfer_message.envelope")?;
        let occurred_at = self.occurred_at.with_timezone(&Utc);

        Ok(TransferMessage {
            id,
            transfer_process_id,
            owner,
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
    pub fn from_domain(msg: &TransferMessage) -> Self {
        Self {
            id: Set(msg.id().to_string()),
            transfer_process_id: Set(msg.transfer_process_id().to_string()),
            user_id: Set(msg.owner().user_id.clone()),
            user_role: Set(msg.owner().role.clone()),
            visibility: Set(msg.owner().visibility.clone()),
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

common::impl_owned!(Model);
