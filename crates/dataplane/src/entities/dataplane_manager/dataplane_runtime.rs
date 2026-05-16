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

use connector::ApiKeyLocation;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DataplaneRuntime {
    pub auth: ResolvedAuthCredentials,
    pub subscription: serde_json::Value,
    pub unsubscription: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(tag = "type")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResolvedAuthCredentials {
    #[default]
    NoAuth,
    BearerToken {
        token: String,
    },
    ApiKey {
        key: String,
        value: String,
        location: ApiKeyLocation,
    },
    BasicAuth {
        username: String,
        password: String,
    },
    OAuth2 {
        access_token: String,
        token_type: String,
        /// Unix timestamp (seconds) at which the token expires, if known.
        expires_at: Option<u64>,
    },
}
