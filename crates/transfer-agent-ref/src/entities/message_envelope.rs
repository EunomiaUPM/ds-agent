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
use serde::{Deserialize, Serialize};
use urn::Urn;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Direction {
    Inbound,
    Outbound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MessageEnvelopeRef {
    pub id: Urn,
    pub direction: Direction,
    pub message_type: String,
    pub state_transition_from: String,
    pub state_transition_to: String,
    pub recorded_at: DateTime<Utc>,
}

impl MessageEnvelopeRef {
    pub fn new(
        id: Urn,
        direction: Direction,
        message_type: impl Into<String>,
        state_transition_from: impl Into<String>,
        state_transition_to: impl Into<String>,
    ) -> Self {
        Self {
            id,
            direction,
            message_type: message_type.into(),
            state_transition_from: state_transition_from.into(),
            state_transition_to: state_transition_to.into(),
            recorded_at: Utc::now(),
        }
    }
}
