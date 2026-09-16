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

use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use ymir::errors::AppResult;

use crate::http::errors::OAuthError;
use crate::http::extractors::{ClientAuth, OAuthPayload};
use crate::http::forms::{
    AuthorizeRequest, AuthorizeResponse, IntrospectRequest, OpenIdConfiguration, RefreshRequest,
    RevokeRequest, TokenRequest,
};
use crate::http::helpers::bearer;
use crate::services::token_service::TokenServiceTrait;
use crate::services::token_service::views::{IntrospectResponse, TokenResponse};
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
            .route(
                "/authorize",
                get(Self::handle_authorize_get).post(Self::handle_authorize_post),
            )
            .route("/token", post(Self::handle_token_issuance))
            .route("/refresh", post(Self::handle_token_refresh))
            .route("/revoke", post(Self::handle_token_revoke))
            .route("/introspect", post(Self::handle_token_introspect))
            .route("/userinfo", get(Self::handle_user_info))
            .route("/user-info", get(Self::handle_user_info))
            .route("/user_info", get(Self::handle_user_info))
            .route(
                "/.well-known/openid-configuration",
                get(Self::handle_oidc_config),
            )
            .with_state(self)
    }

    async fn handle_authorize_get(
        State(s): State<Self>,
        headers: HeaderMap,
        Query(req): Query<AuthorizeRequest>,
    ) -> Result<Json<AuthorizeResponse>, OAuthError> {
        s.process_authorize(headers, req).await
    }

    async fn handle_authorize_post(
        State(s): State<Self>,
        headers: HeaderMap,
        OAuthPayload(req): OAuthPayload<AuthorizeRequest>,
    ) -> Result<Json<AuthorizeResponse>, OAuthError> {
        s.process_authorize(headers, req).await
    }

    async fn process_authorize(
        &self,
        headers: HeaderMap,
        mut req: AuthorizeRequest,
    ) -> Result<Json<AuthorizeResponse>, OAuthError> {
        if req.response_type != "code" {
            return Err(OAuthError::invalid_request(
                "unsupported response_type: only 'code' is supported",
            ));
        }

        if req.code_challenge.trim().is_empty() {
            return Err(OAuthError::invalid_request(
                "code_challenge is required (PKCE)",
            ));
        }

        if req.user_id.is_none() {
            if let Ok(tok) = bearer(&headers) {
                if let Ok(claims) = self.token_svc.validate_token(tok).await {
                    req.user_id = Some(claims.sub);
                }
            }
        }

        let user_id = req
            .user_id
            .as_deref()
            .ok_or_else(|| OAuthError::invalid_request("user identification required"))?;

        let code = self
            .token_svc
            .issue_authorization_code(
                &req.client_id,
                req.redirect_uri.as_deref(),
                req.scope.as_deref(),
                &req.code_challenge,
                req.code_challenge_method.as_deref(),
                Some(user_id),
            )
            .await
            .map_err(|e| OAuthError::invalid_request(e.to_string()))?;

        Ok(Json(AuthorizeResponse {
            code,
            state: req.state,
            redirect_uri: req.redirect_uri,
        }))
    }

    async fn handle_token_issuance(
        State(s): State<Self>,
        headers: HeaderMap,
        OAuthPayload(r): OAuthPayload<TokenRequest>,
    ) -> Result<Json<TokenResponse>, OAuthError> {
        let grant_type = r.grant_type.as_deref().unwrap_or({
            if r.code.is_some() {
                "authorization_code"
            } else if r.assertion.is_some() {
                "urn:ietf:params:oauth:grant-type:jwt-bearer"
            } else if r.refresh_token.is_some() {
                "refresh_token"
            } else if r.username.is_some() {
                "password"
            } else {
                ""
            }
        });

        match grant_type {
            "password" => {
                let username = r
                    .username
                    .ok_or_else(|| OAuthError::invalid_request("missing username"))?;
                let password = r
                    .password
                    .ok_or_else(|| OAuthError::invalid_request("missing password"))?;

                s.token_svc
                    .issue_token_with_scope(&username, &password, r.scope.as_deref())
                    .await
                    .map(Json)
                    .map_err(|e| OAuthError::invalid_grant(e.to_string()))
            }
            "client_credentials" => {
                if let (Some(assertion_type), Some(assertion)) = (
                    r.client_assertion_type.as_deref(),
                    r.client_assertion.as_deref(),
                ) {
                    if assertion_type == "urn:ietf:params:oauth:client-assertion-type:jwt-bearer" {
                        return s
                            .token_svc
                            .issue_jwt_bearer_token(assertion, r.scope.as_deref())
                            .await
                            .map(Json)
                            .map_err(|e| OAuthError::invalid_client(e.to_string()));
                    }
                }

                let client_auth = ClientAuth::extract(
                    &headers,
                    r.client_id.as_deref(),
                    r.client_secret.as_deref(),
                )
                .ok_or_else(|| OAuthError::invalid_client("missing client credentials"))?;

                s.token_svc
                    .issue_client_credentials_token(
                        &client_auth.client_id,
                        &client_auth.client_secret,
                        r.scope.as_deref(),
                    )
                    .await
                    .map(Json)
                    .map_err(|e| {
                        let debug_str = format!("{e:?}");
                        let disp_str = e.to_string();
                        if debug_str.to_lowercase().contains("scope")
                            || disp_str.to_lowercase().contains("scope")
                        {
                            OAuthError::invalid_scope(disp_str)
                        } else {
                            OAuthError::invalid_client(disp_str)
                        }
                    })
            }
            "authorization_code" => {
                let code = r
                    .code
                    .ok_or_else(|| OAuthError::invalid_request("missing code"))?;
                let verifier = r
                    .code_verifier
                    .ok_or_else(|| OAuthError::invalid_request("missing code_verifier (PKCE)"))?;
                let client_auth = ClientAuth::extract(
                    &headers,
                    r.client_id.as_deref(),
                    r.client_secret.as_deref(),
                );
                let client_id = client_auth
                    .as_ref()
                    .map(|c| c.client_id.as_str())
                    .or(r.client_id.as_deref());

                s.token_svc
                    .exchange_authorization_code(
                        &code,
                        &verifier,
                        r.redirect_uri.as_deref(),
                        client_id,
                    )
                    .await
                    .map(Json)
                    .map_err(|e| OAuthError::invalid_grant(e.to_string()))
            }
            "urn:ietf:params:oauth:grant-type:jwt-bearer" => {
                let assertion = r
                    .assertion
                    .ok_or_else(|| OAuthError::invalid_request("missing assertion"))?;
                s.token_svc
                    .issue_jwt_bearer_token(&assertion, r.scope.as_deref())
                    .await
                    .map(Json)
                    .map_err(|e| OAuthError::invalid_grant(e.to_string()))
            }
            "refresh_token" => {
                let refresh_token = r
                    .refresh_token
                    .ok_or_else(|| OAuthError::invalid_request("missing refresh_token"))?;

                s.token_svc
                    .refresh_token_with_scope(&refresh_token, r.scope.as_deref())
                    .await
                    .map(Json)
                    .map_err(|e| OAuthError::invalid_grant(e.to_string()))
            }
            other => Err(OAuthError::unsupported_grant_type(format!(
                "grant_type '{other}' is not supported"
            ))),
        }
    }

    async fn handle_token_refresh(
        State(s): State<Self>,
        OAuthPayload(r): OAuthPayload<RefreshRequest>,
    ) -> Result<Json<TokenResponse>, OAuthError> {
        s.token_svc
            .refresh_token(&r.refresh_token)
            .await
            .map(Json)
            .map_err(|e| OAuthError::invalid_grant(e.to_string()))
    }

    async fn handle_token_revoke(
        State(s): State<Self>,
        OAuthPayload(r): OAuthPayload<RevokeRequest>,
    ) -> StatusCode {
        let _ = s
            .token_svc
            .revoke_token(&r.token, r.token_type_hint.as_deref())
            .await;
        StatusCode::OK
    }

    async fn handle_token_introspect(
        State(s): State<Self>,
        OAuthPayload(r): OAuthPayload<IntrospectRequest>,
    ) -> Json<IntrospectResponse> {
        let res = s
            .token_svc
            .introspect_token(&r.token, r.token_type_hint.as_deref())
            .await
            .unwrap_or_else(|_| IntrospectResponse::inactive());
        Json(res)
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
