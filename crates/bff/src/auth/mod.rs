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

use std::sync::Arc;

use axum::extract::Request;
use axum::http::header::{AUTHORIZATION, WWW_AUTHENTICATE};
use axum::http::{HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use common::auth::claims::Claims;
use common::auth::middleware::OauthTokenValidator;
use serde_json::json;

/// Middleware extracting and validating OAuth bearer tokens from headers or query parameters.
#[derive(Clone)]
pub struct BffAuthMiddleware {
    validator: Option<Arc<dyn OauthTokenValidator>>,
    strict: bool,
}

impl BffAuthMiddleware {
    /// Create a new auth middleware with optional token validator.
    pub fn new(validator: Option<Arc<dyn OauthTokenValidator>>, strict: bool) -> Self {
        Self { validator, strict }
    }

    /// Extract bearer token from Authorization header or URL query string.
    pub fn extract_token(req: &Request) -> Option<String> {
        if let Some(auth_val) = req
            .headers()
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
        {
            if let Some(token) = auth_val.strip_prefix("Bearer ") {
                return Some(token.trim().to_string());
            }
        }

        if let Some(query) = req.uri().query() {
            for pair in query.split('&') {
                if let Some((k, v)) = pair.split_once('=') {
                    if k == "token" || k == "access_token" {
                        return Some(v.to_string());
                    }
                }
            }
        }

        None
    }

    /// Middleware handler injecting security headers and validating identity.
    pub async fn handle(&self, mut req: Request, next: Next) -> Response {
        let token_opt = Self::extract_token(&req);

        if let (Some(validator), Some(token)) = (&self.validator, token_opt) {
            match validator.validate_token(&token).await {
                Ok(claims) => {
                    req.extensions_mut().insert(claims);
                }
                Err(e) if self.strict => {
                    return Self::unauthorized_response(&format!("invalid token: {e}"));
                }
                Err(_) => {}
            }
        } else if self.strict && self.validator.is_some() {
            return Self::unauthorized_response("missing authentication token");
        }

        let mut resp = next.run(req).await;
        Self::apply_security_headers(&mut resp);
        resp
    }

    fn apply_security_headers(resp: &mut Response) {
        let headers = resp.headers_mut();
        headers.insert(
            "x-content-type-options",
            HeaderValue::from_static("nosniff"),
        );
        headers.insert("x-frame-options", HeaderValue::from_static("SAMEORIGIN"));
        headers.insert(
            "x-xss-protection",
            HeaderValue::from_static("1; mode=block"),
        );
        headers.insert(
            "referrer-policy",
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        );
    }

    fn unauthorized_response(msg: &str) -> Response {
        let body = Json(json!({
            "error": "unauthorized",
            "error_description": msg
        }));
        let mut resp = (StatusCode::UNAUTHORIZED, body).into_response();
        resp.headers_mut().insert(
            WWW_AUTHENTICATE,
            HeaderValue::from_static("Bearer realm=\"ds-gateway\", error=\"invalid_token\""),
        );
        Self::apply_security_headers(&mut resp);
        resp
    }
}
