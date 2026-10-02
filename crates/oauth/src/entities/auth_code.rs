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

//! Authorization codes of the PKCE flow.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::entities::role::RbacRole;

/// RFC 7636 Proof Key for Code Exchange (PKCE) authorization code record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthCode {
    pub code: String,
    pub client_id: String,
    pub redirect_uri: Option<String>,
    pub tenant_id: String,
    pub role: RbacRole,
    pub scopes: Vec<String>,
    pub code_challenge: String,
    pub code_challenge_method: String,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
}

impl AuthCode {
    /// Verify code_verifier against stored code_challenge according to RFC 7636.
    pub fn verify_pkce(&self, code_verifier: &str) -> bool {
        match self.code_challenge_method.as_str() {
            "S256" => {
                let digest = Sha256::digest(code_verifier.as_bytes());
                let encoded = URL_SAFE_NO_PAD.encode(digest);
                encoded == self.code_challenge
            }
            "plain" => code_verifier == self.code_challenge,
            _ => false,
        }
    }
}
