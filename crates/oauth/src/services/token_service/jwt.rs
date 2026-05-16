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

use crate::entities::role::Role;

#[derive(Serialize, Deserialize)]
pub(crate) struct AccessClaims {
    pub sub: String,
    pub role: Role,
    pub iat: i64,
    pub exp: i64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct RefreshClaims {
    pub sub: String,
    pub role: Role,
    pub jti: String,
    pub iat: i64,
    pub exp: i64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct IdTokenClaims {
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
    pub email: String,
    pub role: Role,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

pub(crate) fn as_map(v: serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
    match v {
        serde_json::Value::Object(m) => m,
        _ => serde_json::Map::new(),
    }
}
