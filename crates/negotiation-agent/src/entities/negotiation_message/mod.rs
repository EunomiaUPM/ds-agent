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

use crate::data::entities::negotiation_message::NewNegotiationMessageModel;
use serde::{Deserialize, Serialize};
use urn::Urn;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NewNegotiationMessageDto {
    pub id: Option<Urn>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    pub negotiation_agent_process_id: Urn,
    pub direction: String,
    pub protocol: String,
    pub message_type: String,
    pub state_transition_from: String,
    pub state_transition_to: String,
    pub payload: serde_json::Value,
}

impl NewNegotiationMessageDto {
    pub fn into_model(self, tenant_id: String) -> NewNegotiationMessageModel {
        NewNegotiationMessageModel {
            id: self.id,
            tenant_id,
            negotiation_agent_process_id: self.negotiation_agent_process_id,
            direction: self.direction,
            protocol: self.protocol,
            message_type: self.message_type,
            state_transition_from: self.state_transition_from,
            state_transition_to: self.state_transition_to,
            payload: self.payload,
        }
    }
}
