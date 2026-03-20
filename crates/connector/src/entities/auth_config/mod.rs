/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

//! Authentication configuration types.
//!
//! [`AuthenticationConfig`] is the top-level enum stored in a connector template
//! and resolved into a connector instance.  It captures all the information
//! needed to authenticate against the remote endpoint.
//!
//! # Secret fields
//!
//! Fields typed as [`SecretString`] (passwords, tokens, client secrets, API key
//! values) are intentionally **not** parameterisable via `{{__NAME__}}`
//! placeholders.  Only the non-secret fields (username, key name, token URL,
//! client ID, scopes) support template substitution.

pub mod api_key;
pub mod basic_auth;
pub mod oauth;

pub use api_key::*;
pub use basic_auth::*;
pub use oauth::*;

use crate::entities::common::secret_management::SecretString;
use crate::entities::parameters::{TemplateString, TemplateVecString};
use serde::{Deserialize, Serialize};

/// The authentication strategy for a connector.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthenticationConfig {
    /// No authentication required.
    NoAuth,

    /// HTTP Basic Authentication.
    ///
    /// `username` supports `{{__PARAM__}}` placeholders.
    /// `password` is a [`SecretString`] and is not parameterisable.
    BasicAuth(BasicAuthConfig),

    /// Bearer token authentication.
    ///
    /// `token` is a [`SecretString`] and is not parameterisable.
    BearerToken { token: SecretString },

    /// API key authentication (header or query parameter).
    ///
    /// `key` (the header/query-parameter name) supports `{{__PARAM__}}` placeholders.
    /// `value` is a [`SecretString`] and is not parameterisable.
    ApiKey { key: TemplateString, value: SecretString, location: ApiKeyLocation },

    /// OAuth 2.0 client-credentials or authorization-code flow.
    ///
    /// `token_url`, `client_id`, and `scopes` support `{{__PARAM__}}` placeholders.
    /// `client_secret` is a [`SecretString`] and is not parameterisable.
    OAuth2 {
        grant_type: OAuthGrantType,
        token_url: TemplateString,
        client_id: TemplateString,
        client_secret: SecretString,
        scopes: TemplateVecString,
    },
}
