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

//! Axum HTTP authentication middleware and dual token extraction.

use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::header::{AUTHORIZATION, WWW_AUTHENTICATE};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use ymir::errors::{AppResult, Errors, Outcome};

use crate::auth::token::OauthTokenValidator;
use crate::auth::validators::AuthValidators;

/// HTTP authentication middleware supporting dual token extraction (header and query).
#[derive(Clone)]
pub struct AuthHttpMiddleware {
    validator: Option<Arc<dyn OauthTokenValidator>>,
    strict: bool,
}

impl AuthHttpMiddleware {
    /// Creates a new auth middleware with optional token validator and strict flag.
    pub fn new(validator: Option<Arc<dyn OauthTokenValidator>>, strict: bool) -> Self {
        Self { validator, strict }
    }

    /// Creates a strict auth middleware with a required token validator.
    pub fn strict(validator: Arc<dyn OauthTokenValidator>) -> Self {
        Self::new(Some(validator), true)
    }

    /// Creates a permissive auth middleware where unauthenticated requests pass through.
    pub fn permissive(validator: Option<Arc<dyn OauthTokenValidator>>) -> Self {
        Self::new(validator, false)
    }

    /// Axum middleware function extracting and validating token from state.
    pub async fn run(
        State(validator): State<Arc<dyn OauthTokenValidator>>,
        mut req: Request,
        next: Next,
    ) -> AppResult<Response> {
        let token = Self::extract_token(&req)
            .ok_or_else(|| Errors::unauthorized("missing or malformed Authorization header", None))?;
        let claims = validator.validate_token(&token).await?;
        AuthValidators::claims_validator()
            .validate(&claims)
            .map_err(|vs| Errors::unauthorized(vs.to_string(), None))?;
        req.extensions_mut().insert(claims);
        let mut resp = next.run(req).await;
        Self::apply_security_headers(&mut resp);
        Ok(resp)
    }

    /// Extracts bearer token from Authorization header or URL query string (`token`/`access_token`).
    pub fn extract_token(req: &Request) -> Option<String> {
        if let Ok(token) = Self::bearer(req.headers()) {
            return Some(token.to_string());
        }

        if let Some(query) = req.uri().query() {
            for pair in query.split('&') {
                if let Some((k, v)) = pair.split_once('=') {
                    if (k == "token" || k == "access_token") && !v.trim().is_empty() {
                        return Some(v.to_string());
                    }
                }
            }
        }

        None
    }

    /// Extracts the Bearer token string from the HTTP Authorization header.
    pub fn bearer(headers: &HeaderMap) -> Outcome<&str> {
        headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| Errors::unauthorized("missing or malformed Authorization header", None))
    }

    /// Middleware handler injecting security headers and validating identity.
    pub async fn handle(&self, mut req: Request, next: Next) -> Response {
        let token_opt = Self::extract_token(&req);

        if let (Some(validator), Some(token)) = (&self.validator, token_opt) {
            match validator.validate_token(&token).await {
                Ok(claims) => {
                    match AuthValidators::claims_validator().validate(&claims) {
                        Ok(()) => {
                            req.extensions_mut().insert(claims);
                        }
                        Err(vs) if self.strict => {
                            return Self::unauthorized_response(&format!("invalid claims: {vs}"));
                        }
                        Err(_) => {}
                    }
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

    /// Applies standard protective HTTP security headers to the response.
    pub fn apply_security_headers(resp: &mut Response) {
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

    /// Builds a standardized 401 Unauthorized response with security headers.
    pub fn unauthorized_response(msg: &str) -> Response {
        let body = axum::Json(json!({
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
