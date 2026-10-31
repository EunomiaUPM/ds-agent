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

//! Negotiation message output views.

use crate::data::sea_orm::orm::negotiation_message::Model as NegotiationMessageModel;
use chrono::{DateTime, Utc};
use ymir::types::oauth::RolePath;
use ymir::types::participants::Visibility;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

/// Management view of a negotiation message.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NegotiationMessageView {
    pub id: String,
    pub negotiation_process_id: String,
    pub user_id: String,
    pub user_role: RolePath,
    pub visibility: Visibility,
    pub direction: String,
    pub protocol: String,
    pub message_type: String,
    pub state_transition_from: String,
    pub state_transition_to: String,
    pub payload: Json,
    pub occurred_at: DateTime<Utc>,
}

ymir::impl_owned!(NegotiationMessageView);

impl NegotiationMessageView {
    pub fn assemble(msg: NegotiationMessageModel) -> Self {
        Self {
            id: msg.id,
            negotiation_process_id: msg.negotiation_agent_process_id,
            user_id: msg.user_id,
            user_role: msg.user_role,
            visibility: msg.visibility,
            direction: msg.direction,
            protocol: msg.protocol,
            message_type: msg.message_type,
            state_transition_from: msg.state_transition_from,
            state_transition_to: msg.state_transition_to,
            payload: msg.payload,
            occurred_at: msg.created_at.with_timezone(&Utc),
        }
    }
}
