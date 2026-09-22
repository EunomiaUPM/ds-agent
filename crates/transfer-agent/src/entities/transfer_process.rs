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

use crate::entities::commands::EditTransferProcessCommand;
use crate::entities::ids::{TenantId, TransferProcessId};
use crate::entities::protocol::{
    ProtocolId, ProtocolState, StateMetadata, TransferCorrelation, TransferRole,
};
use chrono::{DateTime, Duration, Utc};
use common::utils::json_merge;

/// TransferProcess domain entity
#[derive(Debug, Clone)]
pub(crate) struct TransferProcess {
    // Common
    transfer_id: TransferProcessId,
    tenant_id: String,
    role: TransferRole,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    version: u32,

    // Protocol
    protocol: ProtocolId,
    protocol_state: ProtocolState,
    state_metadata: StateMetadata,
    correlation: TransferCorrelation,

    // Opened
    properties: serde_json::Value,
    error_details: Option<serde_json::Value>,
}

#[allow(dead_code)]
impl TransferProcess {
    // Constructors  ─────────────────────────────────────────────────────────────
    /// TransferProcess entity constructor
    pub fn new(
        tenant_id: String,
        role: TransferRole,
        protocol: ProtocolId,
        protocol_state: ProtocolState,
        correlation: TransferCorrelation,
    ) -> Self {
        let now = Utc::now();
        Self {
            transfer_id: TransferProcessId::generate(),
            tenant_id,
            role,
            created_at: now,
            updated_at: now,
            version: 0,
            protocol,
            protocol_state,
            state_metadata: StateMetadata::empty(),
            correlation,
            properties: serde_json::json!({}),
            error_details: None,
        }
    }

    /// TransferProcess entity constructor from arguments
    /// Is same as having all pub(crate) in struct definition
    /// But protecting version
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn rehydrate(
        transfer_id: TransferProcessId,
        tenant_id: String,
        role: TransferRole,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        version: u32,
        protocol: ProtocolId,
        protocol_state: ProtocolState,
        state_metadata: StateMetadata,
        correlation: TransferCorrelation,
        properties: serde_json::Value,
        error_details: Option<serde_json::Value>,
    ) -> Self {
        Self {
            transfer_id,
            tenant_id,
            role,
            created_at,
            updated_at,
            version,
            protocol,
            protocol_state,
            state_metadata,
            correlation,
            properties,
            error_details,
        }
    }

    // Mutators ──────────────────────────────────────────────────────────────

    /// Mutates TransferProcess entity with a `EditTransferProcessCommand`
    /// Useful when comes to mutate process to be persisted
    pub fn apply_edit(&mut self, cmd: EditTransferProcessCommand) {
        if let Some(state) = cmd.state {
            self.protocol_state = state;
            self.state_metadata = cmd.state_metadata.unwrap_or(StateMetadata::empty());
        }
        if let Some(ids) = cmd.identifiers {
            self.correlation.identifiers.extend(ids);
        }
        if let Some(props) = cmd.properties {
            json_merge(&mut self.properties, props);
        }
        if let Some(err) = cmd.error_details {
            self.error_details = Some(err);
        }
        self.bump();
    }

    // Bump version and update_at field at once
    fn bump(&mut self) {
        self.version = self.version.saturating_add(1);
        self.updated_at = Utc::now();
    }

    // Accessors ─────────────────────────────────────────────────────────────

    pub fn id(&self) -> &TransferProcessId {
        &self.transfer_id
    }
    pub fn tenant_id(&self) -> &String {
        &self.tenant_id
    }
    pub fn protocol(&self) -> &ProtocolId {
        &self.protocol
    }
    pub fn state(&self) -> &ProtocolState {
        &self.protocol_state
    }
    pub fn state_metadata(&self) -> &StateMetadata {
        &self.state_metadata
    }
    pub fn role(&self) -> TransferRole {
        self.role
    }
    pub fn version(&self) -> u64 {
        self.version as u64
    }
    pub fn correlation(&self) -> &TransferCorrelation {
        &self.correlation
    }
    pub fn properties(&self) -> &serde_json::Value {
        &self.properties
    }
    pub fn error_details(&self) -> Option<&serde_json::Value> {
        self.error_details.as_ref()
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    // Predicates ────────────────────────────────────────────────────────────

    pub fn belongs_to(&self, tenant: &String) -> bool {
        &self.tenant_id == tenant
    }
    pub fn uses_protocol(&self, protocol: &ProtocolId) -> bool {
        &self.protocol == protocol
    }
    pub fn age(&self) -> Duration {
        Utc::now() - self.created_at
    }
}
