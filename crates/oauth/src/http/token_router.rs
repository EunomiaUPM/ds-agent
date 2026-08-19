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
    oidc_config: OpenIdConfiguration,
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
            oidc_config: OpenIdConfiguration::build(&issuer),
        }
    }

    pub(crate) fn router(self) -> Router {
        Router::new()
            .route("/token", post(Self::handle_token_issuance))
            .route("/refresh", post(Self::handle_token_refresh))
            .route("/revoke", post(Self::handle_token_revoke))
            .route("/userinfo", get(Self::handle_user_info))
            .route("/user-info", get(Self::handle_user_info))
            .route("/user_info", get(Self::handle_user_info))
            .route(
                "/.well-known/openid-configuration",
                get(Self::handle_oidc_config),
            )
            .with_state(self)
    }

    async fn handle_token_issuance(
        State(s): State<Self>,
        Form(r): Form<PasswordGrantRequest>,
    ) -> AppResult<Json<TokenResponse>> {
        Ok(Json(
            s.token_svc.issue_token(&r.username, &r.password).await?,
        ))
    }

    async fn handle_token_refresh(
        State(s): State<Self>,
        Form(r): Form<RefreshRequest>,
    ) -> AppResult<Json<TokenResponse>> {
        Ok(Json(s.token_svc.refresh_token(&r.refresh_token).await?))
    }

    async fn handle_token_revoke(
        State(s): State<Self>,
        Form(r): Form<RevokeRequest>,
    ) -> AppResult<StatusCode> {
        s.token_svc.revoke_refresh_token(&r.token).await?;
        Ok(StatusCode::OK)
    }

    async fn handle_user_info(
        State(s): State<Self>,
        headers: HeaderMap,
    ) -> AppResult<Json<UserInfo>> {
        let claims = s.token_svc.validate_token(bearer(&headers)?).await?;
        Ok(Json(s.user_svc.user_info(&claims.sub).await?))
    }

    async fn handle_oidc_config(State(s): State<Self>) -> Json<OpenIdConfiguration> {
        Json(s.oidc_config.clone())
    }
}
