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

use chrono::{DateTime, Utc};

use crate::entities::commands::NewTransferMessageCommand;
use crate::entities::ids::{MessageId, TenantId, TransferProcessId};
use crate::entities::message_envelope::MessageEnvelope;
use crate::entities::protocol::{ProtocolId, ProtocolMessageType};
use ymir::errors::{Errors, Outcome};

/// Each `TransferProcess` created by a protocol interaction leaves messages.
/// For traceability those messages are saved under `TransferMessage`.
/// Some messages are Inbound (incoming protocol-shaped messages), some are
/// Outbound (outgoing protocol-shaped messages).
#[derive(Clone)]
pub(crate) struct TransferMessage {
    // Common
    pub(crate) id: MessageId,
    pub(crate) transfer_process_id: TransferProcessId,
    pub(crate) tenant_id: String,
    pub(crate) direction: Direction,

    // Protocol
    pub(crate) protocol: ProtocolId,
    pub(crate) message_type: ProtocolMessageType,
    pub(crate) state_transition_from: String,
    pub(crate) state_transition_to: String,

    // RDF Payload
    pub(crate) envelope: MessageEnvelope,

    // Traceability
    pub(crate) occurred_at: DateTime<Utc>,
}

#[allow(dead_code, clippy::result_large_err)]
impl TransferMessage {
    // Constructors ─────────────────────────────────────────────────────────────
    pub(crate) fn from_cmd(cmd: &NewTransferMessageCommand) -> Outcome<Self> {
        let id = cmd.id.clone().unwrap_or_else(MessageId::generate);
        let tenant_id = cmd.tenant_id.clone().ok_or_else(|| {
            Errors::crazy(
                "tenant_id must be resolved before reaching the domain",
                None,
            )
        })?;
        Ok(Self {
            id,
            transfer_process_id: cmd.transfer_process_id.clone(),
            tenant_id,
            direction: cmd.direction,
            protocol: cmd.protocol.clone(),
            message_type: cmd.message_type.clone(),
            state_transition_from: cmd.state_transition_from.0.to_string(),
            state_transition_to: cmd.state_transition_to.0.to_string(),
            envelope: cmd.envelope.clone(),
            occurred_at: Utc::now(),
        })
    }

    // Accessors ─────────────────────────────────────────────────────────────

    pub fn id(&self) -> &MessageId {
        &self.id
    }
    pub fn transfer_process_id(&self) -> &TransferProcessId {
        &self.transfer_process_id
    }
    pub fn tenant_id(&self) -> &String {
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
    pub fn state_transition_from(&self) -> &str {
        &self.state_transition_from
    }
    pub fn state_transition_to(&self) -> &str {
        &self.state_transition_to
    }
    pub fn envelope(&self) -> &MessageEnvelope {
        &self.envelope
    }
    pub fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
    pub fn is_inbound(&self) -> bool {
        self.direction == Direction::Inbound
    }
    pub fn is_outbound(&self) -> bool {
        self.direction == Direction::Outbound
    }
}

/// Direction of the message
/// Inbound, from peer to agent, or Outbound, from admin to agent, to create message to peer
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Direction {
    Inbound,
    Outbound,
}
