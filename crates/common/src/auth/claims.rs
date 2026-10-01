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

//! RBAC roles and decoded JWT claims representation.
//! Who's calling and which rol has?

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RbacRole {
    Admin,
    Owner,
    Reader,
}

impl RbacRole {
    /// Returns static string representation of the role.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Owner => "owner",
            Self::Reader => "reader",
        }
    }

    /// Whether this role is Admin.
    pub fn is_admin(&self) -> bool {
        matches!(self, Self::Admin)
    }

    /// Whether this role has write permissions (Admin or Owner).
    pub fn can_write(&self) -> bool {
        matches!(self, Self::Admin | Self::Owner)
    }
}

impl std::fmt::Display for RbacRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for RbacRole {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "admin" => Ok(RbacRole::Admin),
            "owner" => Ok(RbacRole::Owner),
            "reader" => Ok(RbacRole::Reader),
            other => Err(format!("unknown Rbac role: {other}")),
        }
    }
}

/// Decoded JWT access-token claims inserted into request extensions by auth middleware.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Tenant ID (subject).
    pub sub: String,
    pub role: RbacRole,
    pub iat: i64,
    pub exp: i64,
}

impl Claims {
    /// Returns the tenant identifier (`sub`).
    pub fn tenant_id(&self) -> &str {
        &self.sub
    }

    /// Returns whether the caller holds the Admin role.
    pub fn is_admin(&self) -> bool {
        self.role.is_admin()
    }

    /// Checks whether the token has expired against the given unix timestamp.
    pub fn is_expired(&self, now: i64) -> bool {
        self.exp < now
    }
}
