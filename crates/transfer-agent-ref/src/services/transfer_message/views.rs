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

use crate::entities::ids::{MessageId, TenantId, TransferProcessId};
use crate::entities::message_envelope::MessageEnvelope;
use crate::entities::protocol::{ProtocolId, ProtocolMessageType};
use crate::entities::transfer_message::{Direction, TransferMessage};
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TransferMessageView {
    pub id: MessageId,
    pub transfer_process_id: TransferProcessId,
    pub tenant_id: String,
    pub direction: Direction,
    pub protocol: ProtocolId,
    pub message_type: ProtocolMessageType,
    pub state_transition_from: String,
    pub state_transition_to: String,
    pub envelope: MessageEnvelope,
    pub occurred_at: DateTime<Utc>,
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
            state_transition_from: msg.state_transition_from,
            state_transition_to: msg.state_transition_to,
            envelope: msg.envelope,
            occurred_at: msg.occurred_at,
        }
    }
}
