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

use base64::Engine;
use chrono::{DateTime, Utc};
use rand::RngExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::entities::role::RbacRole;

/// Personal Access Token (PAT) for developer automation and service access.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalAccessToken {
    pub id: Uuid,
    pub tenant_id: String,
    pub name: String,
    pub token_prefix: String,
    pub token_hash: String,
    pub role: RbacRole,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub revoked: bool,
}

impl PersonalAccessToken {
    /// Compute SHA-256 hex hash of a raw PAT string.
    pub fn hash_token(raw: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(raw.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Generate a new PAT and return the entity alongside the plaintext token.
    pub fn generate(
        tenant_id: impl Into<String>,
        name: impl Into<String>,
        role: RbacRole,
        scopes: Vec<String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> (Self, String) {
        let mut rng = rand::rng();
        let random_bytes: [u8; 24] = rng.random();
        let suffix = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(random_bytes);
        let raw_token = format!("pat_{suffix}");
        let token_hash = Self::hash_token(&raw_token);
        let token_prefix = raw_token.chars().take(10).collect::<String>();

        let pat = Self {
            id: Uuid::new_v4(),
            tenant_id: tenant_id.into(),
            name: name.into(),
            token_prefix,
            token_hash,
            role,
            scopes,
            expires_at,
            created_at: Utc::now(),
            last_used_at: None,
            revoked: false,
        };

        (pat, raw_token)
    }

    /// Check if the token is currently active (not revoked and not expired).
    pub fn is_active(&self) -> bool {
        if self.revoked {
            return false;
        }
        if let Some(exp) = self.expires_at {
            if exp <= Utc::now() {
                return false;
            }
        }
        true
    }
}
