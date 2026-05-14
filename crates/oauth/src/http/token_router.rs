use std::sync::Arc;

use axum::extract::{Form, FromRef, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use ymir::errors::{AppResult, BadFormat, Errors};
use crate::entities::oidc::{TokenResponse, UserInfo};
use crate::services::AuthServiceTrait;

// Router ────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct TokenRouter {
    service: Arc<dyn AuthServiceTrait>,
    discovery: OpenIdConfiguration,
}

impl FromRef<TokenRouter> for Arc<dyn AuthServiceTrait> {
    fn from_ref(state: &TokenRouter) -> Self {
        state.service.clone()
    }
}

impl TokenRouter {
    pub fn new(service: Arc<dyn AuthServiceTrait>, issuer: impl Into<String>) -> Self {
        let issuer = issuer.into();
        let discovery = OpenIdConfiguration::build(&issuer);
        Self { service, discovery }
    }

    /// Mount under a prefix such as `/oauth`.
    ///
    /// | Method | Path                                | Description                         |
    /// |--------|-------------------------------------|-------------------------------------|
    /// | POST   | `/token`                            | Password grant                      |
    /// | POST   | `/refresh`                          | Refresh token rotation               |
    /// | POST   | `/revoke`                           | Token revocation (RFC 7009)         |
    /// | GET    | `/userinfo`                         | OIDC UserInfo                       |
    /// | GET    | `/.well-known/openid-configuration` | OIDC discovery document             |
    pub fn router(self) -> Router {
        Router::new()
            .route("/token", post(Self::handle_token))
            .route("/refresh", post(Self::handle_refresh))
            .route("/revoke", post(Self::handle_revoke))
            .route("/userinfo", get(Self::handle_userinfo))
            .route("/.well-known/openid-configuration", get(Self::handle_discovery))
            .with_state(self)
    }

    // Token endpoints ─────────────────────────────────────────────────────

    async fn handle_token(
        State(state): State<Self>,
        Form(req): Form<PasswordGrantRequest>,
    ) -> AppResult<Json<TokenResponse>> {
        Ok(Json(state.service.issue_token(&req.username, &req.password).await?))
    }

    async fn handle_refresh(
        State(state): State<Self>,
        Form(req): Form<RefreshRequest>,
    ) -> AppResult<Json<TokenResponse>> {
        Ok(Json(state.service.refresh_token(&req.refresh_token).await?))
    }

    async fn handle_revoke(
        State(state): State<Self>,
        Form(req): Form<RevokeRequest>,
    ) -> AppResult<StatusCode> {
        state.service.revoke_refresh_token(&req.token).await?;
        Ok(StatusCode::OK)
    }

    // OIDC endpoints ──────────────────────────────────────────────────────

    /// `GET /userinfo` — identity claims from a valid access token.
    async fn handle_userinfo(
        State(state): State<Self>,
        headers: HeaderMap,
    ) -> AppResult<Json<UserInfo>> {
        let token = extract_bearer(&headers)?;
        Ok(Json(state.service.userinfo(token).await?))
    }

    /// `GET /.well-known/openid-configuration`
    async fn handle_discovery(State(state): State<Self>) -> Json<OpenIdConfiguration> {
        Json(state.discovery.clone())
    }


}

// Form bodies ───────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct PasswordGrantRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Deserialize)]
pub struct RevokeRequest {
    pub token: String,
}

// OIDC discovery document ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct OpenIdConfiguration {
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
    fn build(issuer: &str) -> Self {
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
                "sub".into(), "iss".into(), "aud".into(), "exp".into(),
                "iat".into(), "email".into(), "role".into(),
            ],
        }
    }
}

// Helpers ───────────────────────────────────────────────────────────────────

fn extract_bearer(headers: &HeaderMap) -> ymir::errors::Outcome<&str> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| {
            Errors::format(BadFormat::Received, "missing or malformed Authorization header", None)
        })
}
