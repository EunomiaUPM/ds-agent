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

use crate::entities::ids::{ParticipantId, TenantId};
use crate::entities::message_envelope::Direction;
use crate::entities::protocol::{ProtocolId, ProtocolState, TransferRole};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use urn::Urn;

// Pagination ────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct Page {
    #[serde(default = "default_limit")]
    pub limit: u32,
    pub cursor: Option<String>,
}

fn default_limit() -> u32 {
    20
}

#[derive(Serialize)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
    pub total: Option<u64>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Sort {
    CreatedAtAsc,
    #[default]
    CreatedAtDesc,
    UpdatedAtDesc,
}

// Filters ───────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct TransferProcessFilter {
    /// `None` means no tenant restriction (admin queries). `Some` restricts to that tenant.
    pub tenant_id: Option<TenantId>,
    pub protocol: Option<ProtocolId>,
    pub state: Option<ProtocolState>,
    pub role: Option<TransferRole>,
    pub agreement_id: Option<Urn>,
    pub peer_participant_id: Option<ParticipantId>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct TransferMessageFilter {
    /// `None` means no tenant restriction (admin queries). `Some` restricts to that tenant.
    pub tenant_id: Option<TenantId>,
    pub direction: Option<Direction>,
    pub protocol: Option<ProtocolId>,
    pub state_transition_to: Option<ProtocolState>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}
