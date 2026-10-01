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
use uuid::Uuid;

use crate::entities::pat::PersonalAccessToken;
use crate::entities::role::RbacRole;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePatResponse {
    pub id: Uuid,
    pub name: String,
    pub token: String,
    pub token_prefix: String,
    pub role: RbacRole,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatView {
    pub id: Uuid,
    pub name: String,
    pub token_prefix: String,
    pub role: RbacRole,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub revoked: bool,
}

impl PatView {
    pub fn assemble(p: PersonalAccessToken) -> Self {
        Self {
            id: p.id,
            name: p.name,
            token_prefix: p.token_prefix,
            role: p.role,
            scopes: p.scopes,
            expires_at: p.expires_at,
            created_at: p.created_at,
            last_used_at: p.last_used_at,
            revoked: p.revoked,
        }
    }
}
