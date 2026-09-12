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

use axum::extract::{FromRequest, Request};
use axum::http::{HeaderMap, header};
use base64::Engine;
use serde::de::DeserializeOwned;

use crate::http::errors::OAuthError;

/// Flexible extractor supporting both application/x-www-form-urlencoded and application/json.
pub(crate) struct OAuthPayload<T>(pub T);

impl<T, S> FromRequest<S> for OAuthPayload<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = OAuthError;

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        let content_type = req
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|h| h.to_str().ok())
            .unwrap_or("")
            .to_string();

        let bytes = axum::body::to_bytes(req.into_body(), 1024 * 64)
            .await
            .map_err(|e| OAuthError::invalid_request(format!("failed to read body: {e}")))?;

        if content_type.starts_with("application/json") {
            serde_json::from_slice::<T>(&bytes)
                .map(OAuthPayload)
                .map_err(|e| OAuthError::invalid_request(format!("malformed JSON payload: {e}")))
        } else {
            serde_urlencoded::from_bytes::<T>(&bytes)
                .map(OAuthPayload)
                .map_err(|e| OAuthError::invalid_request(format!("malformed form payload: {e}")))
        }
    }
}

/// Extracted client credentials from either HTTP Basic auth or request body.
pub(crate) struct ClientAuth {
    pub client_id: String,
    pub client_secret: String,
}

impl ClientAuth {
    pub fn extract(
        headers: &HeaderMap,
        body_id: Option<&str>,
        body_secret: Option<&str>,
    ) -> Option<Self> {
        if let Some(auth_header) = headers
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
        {
            if let Some(encoded) = auth_header.strip_prefix("Basic ") {
                if let Ok(decoded) =
                    base64::engine::general_purpose::STANDARD.decode(encoded.trim())
                {
                    if let Ok(s) = String::from_utf8(decoded) {
                        if let Some((id, secret)) = s.split_once(':') {
                            return Some(Self {
                                client_id: id.to_string(),
                                client_secret: secret.to_string(),
                            });
                        }
                    }
                }
            }
        }
        if let (Some(id), Some(secret)) = (body_id, body_secret) {
            return Some(Self {
                client_id: id.to_string(),
                client_secret: secret.to_string(),
            });
        }
        None
    }
}
