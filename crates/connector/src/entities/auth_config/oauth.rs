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

use crate::entities::common::secret_management::SecretString;
use crate::entities::parameters::TemplateString;
use serde::{Deserialize, Serialize};

/// OAuth 2.0 grant type, carrying only the fields that are specific to each flow.
///
/// Common fields (`token_url`, `client_id`, `client_secret`, `scopes`) live in
/// the parent [`AuthenticationConfig::OAuth2`] variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OAuthGrantType {
    /// Client Credentials flow (machine-to-machine).
    ClientCredentials,
    /// Authorization Code flow (requires browser redirect).
    AuthorizationCode,
    /// Resource Owner Password Credentials flow.
    ///
    /// `username` supports `{{__PARAM__}}` placeholders.
    /// `password` is a [`SecretString`] and supports placeholders via its inner content.
    Password {
        username: TemplateString,
        password: SecretString,
    },
}

/// What the connector should do when the current OAuth2 access token expires.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TokenExpireAction {
    /// Re-authenticate from scratch using the configured grant flow.
    #[default]
    Refetch,
    /// Use the `refresh_token` returned by the token endpoint; fall back to
    /// [`Refetch`](Self::Refetch) if no refresh token is available.
    RefreshOrRefetch,
}
