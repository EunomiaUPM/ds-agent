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
use common::query::{validate_date_range, QueryFilter};
use serde::{Deserialize, Serialize};
use ymir::errors::Outcome;

use crate::data::sea_orm::orm::dataplane_transfers::{
    InteractionMode, TransferRole, TransferState,
};
use crate::data::sea_orm::orm::transfer_event::LogLevel;

/// Filter for `DataplaneTransfer` related queries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DataplaneTransferFilter {
    /// None implies no tenant filter (allowed only for admins).
    pub tenant_id: Option<String>,
    pub transfer_process_id: Option<String>,
    pub role: Option<TransferRole>,
    pub interaction_mode: Option<InteractionMode>,
    pub state: Option<TransferState>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

impl QueryFilter for DataplaneTransferFilter {
    fn validate(&self) -> Outcome<()> {
        validate_date_range(self.created_after, self.created_before)
    }
}

/// Filter for `TransferEvent` related queries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TransferEventFilter {
    /// None implies no tenant filter (allowed only for admins).
    pub tenant_id: Option<String>,
    pub transfer_id: Option<String>,
    pub level: Option<LogLevel>,
    pub component: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

impl QueryFilter for TransferEventFilter {
    fn validate(&self) -> Outcome<()> {
        validate_date_range(self.created_after, self.created_before)
    }
}
