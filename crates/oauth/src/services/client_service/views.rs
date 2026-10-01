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

use crate::entities::client::Client;
use crate::entities::role::RbacRole;

/// Public read-model presentation of an OAuth client.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientView {
    pub client_id: String,
    pub tenant_id: String,
    pub client_name: String,
    pub role: RbacRole,
    pub scopes: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl ClientView {
    /// Assembles a client view from the domain model.
    pub fn assemble(client: Client) -> Self {
        Self {
            client_id: client.client_id,
            tenant_id: client.tenant_id,
            client_name: client.client_name,
            role: client.role,
            scopes: client.scopes,
            created_at: client.created_at,
        }
    }
}
