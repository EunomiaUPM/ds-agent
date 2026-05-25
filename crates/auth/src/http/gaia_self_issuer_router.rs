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
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Form, Json, Router};
use serde_json::Value;
use ymir::errors::AppResult;
use ymir::types::issuing::{
    AuthServerMetadata, CredentialRequestsss, IssuerMetadata, IssuingToken, TokenRequest,
    VCCredOffer,
};
use ymir::types::wallet::waltid::OidcUri;
use ymir::utils::{
    extract_bearer_token, extract_form_payload, extract_payload, extract_query_param,
};

use crate::core::traits::CoreGaiaSelfIssuerTrait;

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
            .route(
                "/.well-known/openid-credential-issuer",
                get(Self::get_issuer),
            )
            .route(
                "/.well-known/oauth-authorization-server",
                get(Self::get_oauth_server),
            )
            .route("/token", post(Self::get_token))
            // .route("/credential", post(Self::post_credential))
            .route("/credential-batch", post(Self::batch_credential))
            .route("/credential/generate", post(Self::gen_credential))
            .route("/credential/request", post(Self::req_credential))
            .with_state(self.self_issuer)
    }

    pub fn well_known(&self) -> Router {
        Router::new()
            .route(
                "/.well-known/openid-credential-issuer",
                get(Self::get_issuer),
            )
            .route(
                "/.well-known/oauth-authorization-server",
                get(Self::get_oauth_server),
            )
            .with_state(self.self_issuer.clone())
    }

    async fn gen_credential(
        State(self_issuer): State<Arc<dyn CoreGaiaSelfIssuerTrait>>,
    ) -> AppResult {
        Ok(match self_issuer.generate_gaia_vcs().await {
            Ok(Some(data)) => data.into_response(),
            Ok(None) => ().into_response(),
            Err(e) => e.into_response(),
        })
    }

    async fn cred_offer(
        State(self_issuer): State<Arc<dyn CoreGaiaSelfIssuerTrait>>,
        Query(params): Query<HashMap<String, String>>,
    ) -> AppResult<Json<VCCredOffer>> {
        let id = extract_query_param(&params, "id")?;
        Ok(Json(self_issuer.get_cred_offer_data(id).await?))
    }

    async fn get_issuer(
        State(issuer): State<Arc<dyn CoreGaiaSelfIssuerTrait>>,
    ) -> AppResult<Json<IssuerMetadata>> {
        Ok(Json(issuer.issuer_metadata()))
    }

    async fn get_oauth_server(
        State(issuer): State<Arc<dyn CoreGaiaSelfIssuerTrait>>,
    ) -> AppResult<Json<AuthServerMetadata>> {
        Ok(Json(issuer.oauth_server_metadata()))
    }

    async fn get_token(
        State(self_issuer): State<Arc<dyn CoreGaiaSelfIssuerTrait>>,
        payload: Result<Form<TokenRequest>, FormRejection>,
    ) -> AppResult<Json<IssuingToken>> {
        let payload = extract_form_payload(payload)?;
        Ok(Json(self_issuer.get_token(payload).await?))
    }

    async fn batch_credential(
        State(self_issuer): State<Arc<dyn CoreGaiaSelfIssuerTrait>>,
        headers: HeaderMap,
        payload: Result<Json<CredentialRequestsss>, JsonRejection>,
    ) -> AppResult<Json<Value>> {
        let payload = extract_payload(payload)?;
        let token = extract_bearer_token(&headers)?;
        Ok(Json(self_issuer.issue_some_cred(payload, token).await?))
    }

    async fn req_credential(
        State(self_issuer): State<Arc<dyn CoreGaiaSelfIssuerTrait>>,
        payload: Result<Json<OidcUri>, JsonRejection>,
    ) -> AppResult<()> {
        let payload = extract_payload(payload)?;
        self_issuer.request_gaia_vc(&payload.uri).await
    }
}
