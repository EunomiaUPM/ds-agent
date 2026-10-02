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

use crate::config::types::{AdminSeedConfig, ServiceClientConfig};
use serde::{Deserialize, Serialize};

/// Who issues the users' tokens, chosen in the yaml with `provider`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "provider", rename_all = "snake_case")]
pub enum OauthConfig {
    /// Keycloak behind the OAuth proxy; the proxy validates the tokens.
    Keycloak,
    /// The `oauth` crate issues and validates the tokens itself.
    BuiltIn(BuiltInOauthConfig),
}

impl OauthConfig {
    /// Built-in OAuth settings; `None` when another provider issues the tokens.
    pub fn built_in(&self) -> Option<&BuiltInOauthConfig> {
        match self {
            Self::BuiltIn(config) => Some(config),
            Self::Keycloak => None,
        }
    }

    /// Credentials to call the other services; the default ones outside the built-in provider.
    pub fn service_client(&self) -> ServiceClientConfig {
        self.built_in()
            .map(|config| config.service_client.clone())
            .unwrap_or_default()
    }
}

/// Built-in provider with its defaults, so nothing has to be configured.
impl Default for OauthConfig {
    fn default() -> Self {
        Self::BuiltIn(BuiltInOauthConfig::default())
    }
}

/// Settings of the built-in OAuth server.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BuiltInOauthConfig {
    /// HS256 signing secret; empty by default, which makes tokens forgeable.
    #[serde(default)]
    pub jwt_secret: String,
    #[serde(default = "default_access_token_ttl")]
    pub access_token_ttl: i64,
    #[serde(default = "default_refresh_token_ttl")]
    pub refresh_token_ttl: i64,
    #[serde(default)]
    pub admin_seed: AdminSeedConfig,
    #[serde(default)]
    pub service_client: ServiceClientConfig,
}

impl Default for BuiltInOauthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: String::new(),
            access_token_ttl: default_access_token_ttl(),
            refresh_token_ttl: default_refresh_token_ttl(),
            admin_seed: AdminSeedConfig::default(),
            service_client: ServiceClientConfig::default(),
        }
    }
}

fn default_access_token_ttl() -> i64 {
    3_600
}

fn default_refresh_token_ttl() -> i64 {
    2_592_000
}
