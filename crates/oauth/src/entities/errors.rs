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

/// RFC 6749 §5.2 standard OAuth 2.0 error codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OAuthErrorCode {
    InvalidRequest,
    InvalidClient,
    InvalidGrant,
    UnauthorizedClient,
    UnsupportedGrantType,
    InvalidScope,
    ServerError,
}

/// RFC 6749 §5.2 standard error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthError {
    pub error: OAuthErrorCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_description: Option<String>,
}

impl OAuthError {
    pub fn new(error: OAuthErrorCode, description: impl Into<String>) -> Self {
        Self {
            error,
            error_description: Some(description.into()),
        }
    }

    pub fn invalid_request(desc: impl Into<String>) -> Self {
        Self::new(OAuthErrorCode::InvalidRequest, desc)
    }

    pub fn invalid_client(desc: impl Into<String>) -> Self {
        Self::new(OAuthErrorCode::InvalidClient, desc)
    }

    pub fn invalid_grant(desc: impl Into<String>) -> Self {
        Self::new(OAuthErrorCode::InvalidGrant, desc)
    }

    pub fn unsupported_grant_type(desc: impl Into<String>) -> Self {
        Self::new(OAuthErrorCode::UnsupportedGrantType, desc)
    }

    pub fn unauthorized_client(desc: impl Into<String>) -> Self {
        Self::new(OAuthErrorCode::UnauthorizedClient, desc)
    }

    pub fn invalid_scope(desc: impl Into<String>) -> Self {
        Self::new(OAuthErrorCode::InvalidScope, desc)
    }

    pub fn server_error(desc: impl Into<String>) -> Self {
        Self::new(OAuthErrorCode::ServerError, desc)
    }
}
