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
use common::config::services::CommonConfig;
use serde::Deserialize;
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;

/// Runtime configuration for the OAuth / OIDC service.
#[derive(Debug, Clone, Deserialize)]
pub struct OAuthConfig {
    /// HS256 signing secret for access, refresh, and ID tokens.
    pub jwt_secret: String,
    /// Access token lifetime in seconds (default: 3 600 = 1 h).
    pub access_token_ttl_secs: i64,
    /// Refresh token lifetime in seconds (default: 2 592 000 = 30 d).
    pub refresh_token_ttl_secs: i64,
    /// OIDC `iss` claim and the base URL for the discovery document.
    /// Example: `"https://auth.example.com"`.
    pub issuer: String,
    /// OIDC `aud` claim — identifies the intended audience of ID tokens.
    /// Typically the client application identifier.
    pub audience: String,
}

impl OAuthConfig {
    pub fn new(
        jwt_secret: impl Into<String>,
        issuer: impl Into<String>,
        audience: impl Into<String>,
    ) -> Self {
        Self {
            jwt_secret: jwt_secret.into(),
            access_token_ttl_secs: 3_600,
            refresh_token_ttl_secs: 2_592_000,
            issuer: issuer.into(),
            audience: audience.into(),
        }
    }

    pub fn with_access_ttl(mut self, secs: i64) -> Self {
        self.access_token_ttl_secs = secs;
        self
    }

    pub fn with_refresh_ttl(mut self, secs: i64) -> Self {
        self.refresh_token_ttl_secs = secs;
        self
    }
}

impl From<CommonConfig> for OAuthConfig {
    fn from(value: CommonConfig) -> Self {
        Self {
            jwt_secret: value.jwt_secret,
            access_token_ttl_secs: value.access_token_ttl,
            refresh_token_ttl_secs: value.refresh_token_ttl,
            issuer: value.hosts.get_host(HostType::Http),
            audience: "client".to_string(),
        }
    }
}
