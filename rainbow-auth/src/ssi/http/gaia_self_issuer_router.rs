/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::rejection::{FormRejection, JsonRejection};
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Form, Json, Router};
use tracing::error;
use ymir::errors::{CustomToResponse, ErrorLogTrait, Errors};
use ymir::types::errors::BadFormat;
use ymir::types::issuing::{CredentialRequest, CredentialRequestsss, TokenRequest};
use ymir::utils::extract_bearer_token;

use crate::ssi::core::traits::CoreGaiaSelfIssuerTrait;

pub struct GaiaSelfIssuerRouter {
    self_issuer: Arc<dyn CoreGaiaSelfIssuerTrait>,
}

impl GaiaSelfIssuerRouter {
    pub fn new(self_issuer: Arc<dyn CoreGaiaSelfIssuerTrait>) -> Self {
        GaiaSelfIssuerRouter { self_issuer }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/credentialOffer", get(Self::cred_offer))
            .route("/.well-known/openid-credential-issuer", get(Self::get_issuer))
            .route("/.well-known/oauth-authorization-server", get(Self::get_oauth_server))
            .route("/token", post(Self::get_token))
            .route("/credential", post(Self::post_credential))
            .route("/credential-batch", post(Self::batch_credential))
            .route("/credential/generate", post(Self::gen_credential))
            .route("/credential/request", post(Self::req_credential))
            .with_state(self.self_issuer)
    }

    async fn gen_credential(
        State(self_issuer): State<Arc<dyn CoreGaiaSelfIssuerTrait>>,
    ) -> impl IntoResponse {
        match self_issuer.generate_gaia_vcs().await {
            Ok(_) => StatusCode::OK.into_response(),
            Err(e) => e.to_response(),
        }
    }

    async fn cred_offer(
        State(self_issuer): State<Arc<dyn CoreGaiaSelfIssuerTrait>>,
        Query(params): Query<HashMap<String, String>>,
    ) -> impl IntoResponse {
        let id = match params.get("id") {
            Some(hash) => hash.clone(),
            None => {
                let error = Errors::format_new(
                    BadFormat::Received,
                    "Unable to retrieve hash from callback",
                );
                error!("{}", error.log());
                return error.into_response();
            }
        };

        match self_issuer.get_cred_offer_data(id).await {
            Ok(data) => (StatusCode::OK, Json(data)).into_response(),
            Err(e) => e.to_response(),
        }
    }

    async fn get_issuer(
        State(self_issuer): State<Arc<dyn CoreGaiaSelfIssuerTrait>>,
    ) -> impl IntoResponse {
        let data = self_issuer.get_issuer_data();
        (StatusCode::OK, Json(data)).into_response()
    }

    async fn get_oauth_server(
        State(self_issuer): State<Arc<dyn CoreGaiaSelfIssuerTrait>>,
    ) -> impl IntoResponse {
        let data = self_issuer.get_oauth_server_data();
        (StatusCode::OK, Json(data)).into_response()
    }

    async fn get_token(
        State(self_issuer): State<Arc<dyn CoreGaiaSelfIssuerTrait>>,
        payload: Result<Form<TokenRequest>, FormRejection>,
    ) -> impl IntoResponse {
        let payload = match payload {
            Ok(Form(data)) => data,
            Err(e) => {
                error!("{}", e.to_string());
                return e.into_response();
            }
        };

        match self_issuer.get_token(payload).await {
            Ok(data) => (StatusCode::OK, Json(data)).into_response(),
            Err(e) => e.to_response(),
        }
    }

    async fn post_credential(
        State(self_issuer): State<Arc<dyn CoreGaiaSelfIssuerTrait>>,
        headers: HeaderMap,
        payload: Result<Json<CredentialRequest>, JsonRejection>,
    ) -> impl IntoResponse {
        let payload = match payload {
            Ok(Json(data)) => data,
            Err(e) => return e.to_response(),
        };

        let token = match extract_bearer_token(headers) {
            Some(token) => token,
            None => {
                let error = Errors::unauthorized_new("Missing token");
                error!("{}", error.log());
                return error.into_response();
            }
        };

        match self_issuer.issue_cred(payload, token).await {
            Ok(data) => (StatusCode::OK, Json(data)).into_response(),
            Err(e) => e.to_response(),
        }
    }

    async fn batch_credential(
        State(self_issuer): State<Arc<dyn CoreGaiaSelfIssuerTrait>>,
        headers: HeaderMap,
        payload: Result<Json<CredentialRequestsss>, JsonRejection>,
    ) -> impl IntoResponse {
        let payload = match payload {
            Ok(Json(data)) => data,
            Err(e) => return e.to_response(),
        };

        let token = match extract_bearer_token(headers) {
            Some(token) => token,
            None => {
                let error = Errors::unauthorized_new("Missing token");
                error!("{}", error.log());
                return error.into_response();
            }
        };

        match self_issuer.issue_some_cred(payload, token).await {
            Ok(data) => (StatusCode::OK, Json(data)).into_response(),
            Err(e) => e.to_response(),
        }
    }

    async fn req_credential(
        State(self_issuer): State<Arc<dyn CoreGaiaSelfIssuerTrait>>,
    ) -> impl IntoResponse {
        match self_issuer.request_gaia_vc().await {
            Ok(data) => (StatusCode::OK, Json(data)).into_response(),
            Err(e) => e.to_response(),
        }
    }
}
