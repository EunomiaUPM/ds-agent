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

use serde::{Deserialize, Serialize};

use crate::entities::role::RbacRole;

/// Which token a JWT is. All three share the signing key, so the `typ` claim is what keeps a
/// refresh or ID token from passing as an access token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    Access,
    Refresh,
    Id,
}

/// Claims of a token this service issues; verification rejects any other `typ`.
pub trait TypedClaims {
    const TYPE: TokenType;
    fn token_type(&self) -> TokenType;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessClaims {
    pub typ: TokenType,
    pub sub: String,
    pub role: RbacRole,
    pub iat: i64,
    pub exp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
}

impl TypedClaims for AccessClaims {
    const TYPE: TokenType = TokenType::Access;
    fn token_type(&self) -> TokenType {
        self.typ
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub typ: TokenType,
    pub sub: String,
    pub role: RbacRole,
    pub jti: String,
    pub iat: i64,
    pub exp: i64,
}

impl TypedClaims for RefreshClaims {
    const TYPE: TokenType = TokenType::Refresh;
    fn token_type(&self) -> TokenType {
        self.typ
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdTokenClaims {
    pub typ: TokenType,
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
    pub email: String,
    pub role: RbacRole,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl TypedClaims for IdTokenClaims {
    const TYPE: TokenType = TokenType::Id;
    fn token_type(&self) -> TokenType {
        self.typ
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtAssertionClaims {
    pub iss: String,
    pub sub: String,
    #[serde(default)]
    pub aud: Option<serde_json::Value>,
    pub exp: i64,
    #[serde(default)]
    pub iat: Option<i64>,
    #[serde(default)]
    pub jti: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
}

pub fn as_map(v: serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
    match v {
        serde_json::Value::Object(m) => m,
        _ => serde_json::Map::new(),
    }
}
