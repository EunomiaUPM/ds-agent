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

use serde::Serialize;

use crate::entities::role::RbacRole;
use crate::entities::user::User;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UserView {
    pub tenant_id: String,
    pub email: String,
    pub role: RbacRole,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub extra_fields: serde_json::Value,
}

impl UserView {
    pub(crate) fn assemble(u: User) -> Self {
        Self {
            tenant_id: u.tenant_id,
            email: u.email,
            role: u.role,
            created_at: u.created_at,
            extra_fields: u.extra_fields,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct UserInfo {
    pub sub: String,
    pub email: String,
    pub role: RbacRole,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl UserInfo {
    pub(crate) fn assemble(u: User) -> Self {
        let extra = match u.extra_fields {
            serde_json::Value::Object(m) => m,
            _ => serde_json::Map::new(),
        };
        Self {
            sub: u.tenant_id,
            email: u.email,
            role: u.role,
            extra,
        }
    }
}
