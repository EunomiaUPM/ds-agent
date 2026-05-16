use std::sync::Arc;

use axum::extract::{Form, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use ymir::errors::AppResult;

use crate::http::forms::{
    OpenIdConfiguration, PasswordGrantRequest, RefreshRequest, RevokeRequest,
};
use crate::http::helpers::bearer;
use crate::services::token_service::TokenServiceTrait;
use crate::services::token_service::views::TokenResponse;
use crate::services::user_service::UserServiceTrait;
use crate::services::user_service::views::UserInfo;

#[derive(Clone)]
pub(crate) struct TokenRouter {
    token_svc: Arc<dyn TokenServiceTrait>,
    user_svc: Arc<dyn UserServiceTrait>,
    discovery: OpenIdConfiguration,
}

impl TokenRouter {
    pub(crate) fn new(
        token_svc: Arc<dyn TokenServiceTrait>,
        user_svc: Arc<dyn UserServiceTrait>,
        issuer: impl Into<String>,
    ) -> Self {
        let issuer = issuer.into();
        Self {
            token_svc,
            user_svc,
            discovery: OpenIdConfiguration::build(&issuer),
        }
    }

    pub(crate) fn router(self) -> Router {
        Router::new()
            .route("/token", post(Self::handle_token))
            .route("/refresh", post(Self::handle_refresh))
            .route("/revoke", post(Self::handle_revoke))
            .route("/userinfo", get(Self::handle_userinfo))
            .route(
                "/.well-known/openid-configuration",
                get(Self::handle_discovery),
            )
            .with_state(self)
    }

    async fn handle_token(
        State(s): State<Self>,
        Form(r): Form<PasswordGrantRequest>,
    ) -> AppResult<Json<TokenResponse>> {
        Ok(Json(
            s.token_svc.issue_token(&r.username, &r.password).await?,
        ))
    }

    async fn handle_refresh(
        State(s): State<Self>,
        Form(r): Form<RefreshRequest>,
    ) -> AppResult<Json<TokenResponse>> {
        Ok(Json(s.token_svc.refresh_token(&r.refresh_token).await?))
    }

    async fn handle_revoke(
        State(s): State<Self>,
        Form(r): Form<RevokeRequest>,
    ) -> AppResult<StatusCode> {
        s.token_svc.revoke_refresh_token(&r.token).await?;
        Ok(StatusCode::OK)
    }

    async fn handle_userinfo(
        State(s): State<Self>,
        headers: HeaderMap,
    ) -> AppResult<Json<UserInfo>> {
        let claims = s.token_svc.validate_token(bearer(&headers)?).await?;
        Ok(Json(s.user_svc.userinfo(&claims.sub).await?))
    }

    async fn handle_discovery(State(s): State<Self>) -> Json<OpenIdConfiguration> {
        Json(s.discovery.clone())
    }
}
