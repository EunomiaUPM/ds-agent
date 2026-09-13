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

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::entities::client::Client;
use crate::entities::query::{Page, Sort, UserFilter};
use crate::entities::role::RbacRole;
use common::query::QuerySpec;

/// Unified OAuth 2.0 token request supporting multiple grant types.
#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct TokenRequest {
    pub grant_type: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
    // PKCE authorization_code grant fields:
    pub code: Option<String>,
    pub redirect_uri: Option<String>,
    pub code_verifier: Option<String>,
    // RFC 7523 JWT Bearer grant:
    pub assertion: Option<String>,
    // RFC 7523 Client assertion:
    pub client_assertion_type: Option<String>,
    pub client_assertion: Option<String>,
}

/// RFC 7636 Authorization request for PKCE flow.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct AuthorizeRequest {
    pub response_type: String,
    pub client_id: String,
    pub redirect_uri: Option<String>,
    pub scope: Option<String>,
    pub state: Option<String>,
    pub code_challenge: String,
    pub code_challenge_method: Option<String>,
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizeResponse {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_uri: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PasswordGrantRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RefreshRequest {
    pub refresh_token: String,
}

/// RFC 7009 revocation request.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RevokeRequest {
    pub token: String,
    pub token_type_hint: Option<String>,
}

/// RFC 7662 token introspection request.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct IntrospectRequest {
    pub token: String,
    pub token_type_hint: Option<String>,
}

/// RFC 7662 token introspection response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntrospectResponse {
    pub active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iat: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

impl IntrospectResponse {
    pub fn inactive() -> Self {
        Self {
            active: false,
            scope: None,
            client_id: None,
            sub: None,
            exp: None,
            iat: None,
            token_type: None,
            role: None,
        }
    }
}

/// Public representation of an OAuth client.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientView {
    pub client_id: String,
    pub client_name: String,
    pub role: RbacRole,
    pub scopes: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl ClientView {
    pub fn assemble(client: Client) -> Self {
        Self {
            client_id: client.client_id,
            client_name: client.client_name,
            role: client.role,
            scopes: client.scopes,
            created_at: client.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OpenIdConfiguration {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
    pub revocation_endpoint: String,
    pub introspection_endpoint: String,
    pub response_types_supported: Vec<String>,
    pub subject_types_supported: Vec<String>,
    pub id_token_signing_alg_values_supported: Vec<String>,
    pub grant_types_supported: Vec<String>,
    pub code_challenge_methods_supported: Vec<String>,
    pub token_endpoint_auth_methods_supported: Vec<String>,
    pub revocation_endpoint_auth_methods_supported: Vec<String>,
    pub introspection_endpoint_auth_methods_supported: Vec<String>,
    pub scopes_supported: Vec<String>,
    pub claims_supported: Vec<String>,
}

impl OpenIdConfiguration {
    pub(crate) fn build(issuer: &str) -> Self {
        Self {
            issuer: issuer.to_string(),
            authorization_endpoint: format!("{issuer}/authorize"),
            token_endpoint: format!("{issuer}/token"),
            userinfo_endpoint: format!("{issuer}/userinfo"),
            revocation_endpoint: format!("{issuer}/revoke"),
            introspection_endpoint: format!("{issuer}/introspect"),
            response_types_supported: vec!["code".into(), "token".into(), "id_token token".into()],
            subject_types_supported: vec!["public".into()],
            id_token_signing_alg_values_supported: vec!["HS256".into()],
            grant_types_supported: vec![
                "password".into(),
                "client_credentials".into(),
                "refresh_token".into(),
                "authorization_code".into(),
                "urn:ietf:params:oauth:grant-type:jwt-bearer".into(),
            ],
            code_challenge_methods_supported: vec!["S256".into(), "plain".into()],
            token_endpoint_auth_methods_supported: vec![
                "client_secret_basic".into(),
                "client_secret_post".into(),
                "private_key_jwt".into(),
            ],
            revocation_endpoint_auth_methods_supported: vec![
                "client_secret_basic".into(),
                "client_secret_post".into(),
                "none".into(),
            ],
            introspection_endpoint_auth_methods_supported: vec![
                "client_secret_basic".into(),
                "client_secret_post".into(),
            ],
            scopes_supported: vec!["openid".into(), "profile".into(), "email".into()],
            claims_supported: vec![
                "sub".into(),
                "iss".into(),
                "aud".into(),
                "exp".into(),
                "iat".into(),
                "email".into(),
                "role".into(),
                "scope".into(),
                "client_id".into(),
            ],
        }
    }
}

pub(crate) type UserListQuery = QuerySpec<UserFilter, Sort>;
