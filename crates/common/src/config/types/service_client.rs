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

/// OAuth client an agent authenticates with on service-to-service calls (client_credentials).
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct ServiceClientConfig {
    pub client_id: String,
    pub client_secret: String,
    /// OAuth token endpoint; defaults to `{own http host}/oauth/token` (monolith layout).
    pub token_url: Option<String>,
}

impl Default for ServiceClientConfig {
    fn default() -> Self {
        Self {
            client_id: "eunomia-services".to_string(),
            client_secret: "eunomia-services-secret".to_string(),
            token_url: None,
        }
    }
}
