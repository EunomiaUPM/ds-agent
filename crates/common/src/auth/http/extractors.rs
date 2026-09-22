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

//! Axum request extractors for authenticated identity, claims, and tracing headers.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::{HeaderMap, HeaderName, HeaderValue};
use ymir::errors::Errors;

use crate::auth::access::AccessScope;
use crate::auth::claims::Claims;
use crate::auth::TENANT_HEADER;

/// Extractor extracting validated JWT claims from request extensions.
#[derive(Debug, Clone)]
pub struct AuthClaims(pub Claims);

impl std::ops::Deref for AuthClaims {
    type Target = Claims;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S: Send + Sync> FromRequestParts<S> for AuthClaims {
    type Rejection = Errors;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Claims>()
            .cloned()
            .map(AuthClaims)
            .ok_or_else(|| Errors::unauthorized("authentication required: missing claims", None))
    }
}

/// Automatic Axum extractor for authenticated `AccessScope`.
/// Delegates the tenant-boundary rule to `AccessScope::from_tenant_header`.
impl<S: Send + Sync> FromRequestParts<S> for AccessScope {
    type Rejection = Errors;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let claims =
            parts.extensions.get::<Claims>().cloned().ok_or_else(|| {
                Errors::unauthorized("authentication required: missing claims", None)
            })?;

        let requested = parts
            .headers
            .get(TENANT_HEADER)
            .and_then(|v| v.to_str().ok());

        AccessScope::from_tenant_header(&claims, requested)
    }
}

/// Request tracing headers extracted for propagation and response echoing.
#[derive(Debug, Clone)]
pub struct ExtractedHeaders {
    pub request_id: String,
    pub correlation_id: Option<String>,
}

impl ExtractedHeaders {
    /// Base response headers: echoes X-Request-ID and X-Correlation-ID.
    pub fn response_headers(&self) -> HeaderMap {
        let mut map = HeaderMap::new();
        Self::insert_str(&mut map, "x-request-id", &self.request_id);
        if let Some(c) = &self.correlation_id {
            Self::insert_str(&mut map, "x-correlation-id", c);
        }
        map
    }

    /// Adds X-Total-Count on top of the base response headers (for paginated responses).
    pub fn response_headers_paged(&self, total: Option<u64>) -> HeaderMap {
        let mut map = self.response_headers();
        if let Some(n) = total {
            Self::insert_str(&mut map, "x-total-count", &n.to_string());
        }
        map
    }

    fn get_header_str<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
        let key = HeaderName::from_bytes(name.as_bytes()).ok()?;
        headers.get(&key).and_then(|v| v.to_str().ok())
    }

    fn insert_str(map: &mut HeaderMap, name: &'static str, value: &str) {
        if let Ok(v) = HeaderValue::from_str(value) {
            map.insert(HeaderName::from_static(name), v);
        }
    }
}

/// Automatic Axum extractor for request tracing headers.
impl<S: Send + Sync> FromRequestParts<S> for ExtractedHeaders {
    type Rejection = Errors;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let request_id = Self::get_header_str(&parts.headers, "x-request-id")
            .map(|s| s.to_string())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let correlation_id =
            Self::get_header_str(&parts.headers, "x-correlation-id").map(|s| s.to_string());
        Ok(Self {
            request_id,
            correlation_id,
        })
    }
}
