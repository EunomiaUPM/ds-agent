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

//! Domain filters for transfer processes and messages.

use chrono::{DateTime, Utc};
use common::query::{QueryFilter, validate_date_range};
use serde::{Deserialize, Serialize};
use ymir::errors::Outcome;

/// Filter criteria for querying transfer processes in transfer-agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TransferProcessFilter {
    pub state: Option<String>,
    pub role: Option<String>,
    pub protocol: Option<String>,
    pub agreement_id: Option<String>,
    pub associated_agent_peer: Option<String>,
    pub connector_instance_id: Option<String>,
    pub transfer_direction: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

impl QueryFilter for TransferProcessFilter {
    fn is_empty(&self) -> bool {
        self.state.is_none()
            && self.role.is_none()
            && self.protocol.is_none()
            && self.agreement_id.is_none()
            && self.associated_agent_peer.is_none()
            && self.connector_instance_id.is_none()
            && self.transfer_direction.is_none()
            && self.created_after.is_none()
            && self.created_before.is_none()
    }

    fn validate(&self) -> Outcome<()> {
        validate_date_range(self.created_after, self.created_before)
    }
}

/// Filter criteria for querying transfer messages in transfer-agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TransferMessageFilter {
    pub process_id: Option<String>,
    pub protocol: Option<String>,
    pub message_type: Option<String>,
    pub direction: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

impl QueryFilter for TransferMessageFilter {
    fn is_empty(&self) -> bool {
        self.process_id.is_none()
            && self.protocol.is_none()
            && self.message_type.is_none()
            && self.direction.is_none()
            && self.created_after.is_none()
            && self.created_before.is_none()
    }

    fn validate(&self) -> Outcome<()> {
        validate_date_range(self.created_after, self.created_before)
    }
}
