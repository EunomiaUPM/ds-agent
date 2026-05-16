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

use crate::entities::ids::{TenantId, TransferProcessId};
use crate::entities::protocol::{
    ProtocolId, ProtocolState, StateMetadata, TransferCorrelation, TransferRole,
    CONSUMER_PID_KEY, PROVIDER_PID_KEY,
};
use crate::entities::transfer_process::TransferProcess;
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value as Json;
use std::collections::HashMap;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TransferProcessView {
    pub id: TransferProcessId,
    pub tenant_id: TenantId,
    pub role: TransferRole,
    pub protocol: ProtocolId,
    pub state: ProtocolState,
    pub state_metadata: StateMetadata,
    pub correlation: TransferCorrelation,
    pub properties: Json,
    pub error_details: Option<Json>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: u64,
}

impl TransferProcessView {
    pub(crate) fn assemble(
        process: TransferProcess,
        extra_identifiers: HashMap<String, String>,
    ) -> Self {
        let mut correlation = process.correlation().clone();
        correlation.identifiers.extend(extra_identifiers);
        if correlation.consumer_pid.is_none() {
            if let Some(v) = correlation.identifiers.get(CONSUMER_PID_KEY) {
                correlation.consumer_pid = Some(v.clone());
            }
        }
        if correlation.provider_pid.is_none() {
            if let Some(v) = correlation.identifiers.get(PROVIDER_PID_KEY) {
                correlation.provider_pid = Some(v.clone());
            }
        }
        Self {
            id: process.id().clone(),
            tenant_id: process.tenant_id().clone(),
            role: process.role(),
            protocol: process.protocol().clone(),
            state: process.state().clone(),
            state_metadata: process.state_metadata().clone(),
            correlation,
            properties: process.properties().clone(),
            error_details: process.error_details().cloned(),
            created_at: process.created_at(),
            updated_at: process.updated_at(),
            version: process.version(),
        }
    }
}
