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

//! Create and update commands.

use crate::entities::role::RbacRole;
use serde::{Deserialize, Serialize};

/// New user; `tenant_id` is also its identifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserCommand {
    pub tenant_id: String,
    pub email: String,
    pub password: String,
    pub role: RbacRole,
    #[serde(default)]
    pub extra_fields: serde_json::Value,
}

/// Partial user update; absent fields stay as they are.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchUserCommand {
    pub email: Option<String>,
    pub role: Option<RbacRole>,
    pub extra_fields: Option<serde_json::Value>,
}

/// New OAuth client; `tenant_id` is only honoured for admins.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateClientCommand {
    pub client_id: String,
    pub tenant_id: Option<String>,
    pub client_secret: String,
    pub client_name: String,
    pub role: RbacRole,
    #[serde(default)]
    pub scopes: Vec<String>,
}

/// New personal access token; without `expires_at` it never expires.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePatCommand {
    pub name: String,
    #[serde(default)]
    pub scopes: Vec<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}
