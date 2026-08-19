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

use crate::entities::query::{Page, Sort, UserFilter};

#[derive(Deserialize)]
pub(crate) struct PasswordGrantRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub(crate) struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Deserialize)]
pub(crate) struct RevokeRequest {
    pub token: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct OpenIdConfiguration {
    pub issuer: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
    pub response_types_supported: Vec<String>,
    pub subject_types_supported: Vec<String>,
    pub id_token_signing_alg_values_supported: Vec<String>,
    pub grant_types_supported: Vec<String>,
    pub scopes_supported: Vec<String>,
    pub claims_supported: Vec<String>,
}

impl OpenIdConfiguration {
    pub(crate) fn build(issuer: &str) -> Self {
        Self {
            issuer: issuer.to_string(),
            token_endpoint: format!("{issuer}/token"),
            userinfo_endpoint: format!("{issuer}/userinfo"),
            response_types_supported: vec!["token".into(), "id_token token".into()],
            subject_types_supported: vec!["public".into()],
            id_token_signing_alg_values_supported: vec!["HS256".into()],
            grant_types_supported: vec!["password".into(), "refresh_token".into()],
            scopes_supported: vec!["openid".into(), "profile".into(), "email".into()],
            claims_supported: vec![
                "sub".into(),
                "iss".into(),
                "aud".into(),
                "exp".into(),
                "iat".into(),
                "email".into(),
                "role".into(),
            ],
        }
    }
}

#[derive(Deserialize, Default)]
pub(crate) struct UserListQuery {
    #[serde(flatten)]
    pub filter: UserFilter,
    #[serde(flatten)]
    pub page: Page,
    #[serde(default)]
    pub sort: Sort,
}
