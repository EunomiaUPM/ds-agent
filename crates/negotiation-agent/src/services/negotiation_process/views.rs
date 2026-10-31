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

//! Negotiation process output views.

use crate::data::sea_orm::orm::negotiation_process::Model as NegotiationProcessModel;
use crate::entities::negotiation_process::NegotiationCorrelation;
use chrono::{DateTime, Utc};
use ymir::types::oauth::RolePath;
use ymir::types::participants::Visibility;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use std::collections::HashMap;

/// Management view of a negotiation process with its correlation.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NegotiationProcessView {
    pub id: String,
    pub user_id: String,
    pub user_role: RolePath,
    pub visibility: Visibility,
    pub role: String,
    pub protocol: String,
    pub state: String,
    pub state_attribute: Option<String>,
    pub correlation: NegotiationCorrelation,
    pub properties: Json,
    pub error_details: Option<Json>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: u64,
}

ymir::impl_owned!(NegotiationProcessView);

impl NegotiationProcessView {
    /// View of `process` with its stored `identifiers`.
    pub fn assemble(process: NegotiationProcessModel, identifiers: HashMap<String, String>) -> Self {
        let created_at = process.created_at.with_timezone(&Utc);
        let updated_at = process.updated_at.map_or(created_at, |at| at.with_timezone(&Utc));
        let correlation = NegotiationCorrelation::new(
            identifiers,
            process.callback_address,
            process.associated_agent_peer,
        );
        Self {
            id: process.id,
            user_id: process.user_id,
            user_role: process.user_role,
            visibility: process.visibility,
            role: process.role,
            protocol: process.protocol,
            state: process.state,
            state_attribute: process.state_attribute,
            correlation,
            properties: process.properties,
            error_details: process.error_details,
            created_at,
            updated_at,
            // The table has no counter; every write stamps `updated_at`.
            version: u64::try_from(updated_at.timestamp_micros()).unwrap_or_default(),
        }
    }
}
