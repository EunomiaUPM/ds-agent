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

use axum::extract::rejection::FormRejection;
use axum::extract::{Path, State};
use axum::http::header::CONTENT_TYPE;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Form, Json, Router};
use serde_json::json;
use ymir::errors::AppResult;
use ymir::http::routes::verifier;
use ymir::types::gnap::InteractionFinishResponse;
use ymir::types::vcs::VPDef;
use ymir::types::verification::request_object::REQUEST_OBJECT_MEDIA_TYPE;
use ymir::types::verification::{DirectPostResponse, RequestUriPost, VerifyPayload};
use ymir::utils::extract_form_payload;

use crate::modules::VerifierModule;

/// OID4VP routes where a peer's wallet fetches the presentation definition and posts its
/// presentation.
pub struct VerifierRouter {
    verifier: Arc<dyn VerifierModule>,
}

impl VerifierRouter {
    // ==========================================================================================
    // Sub-routers
    // ==========================================================================================

    pub fn new(verifier: Arc<dyn VerifierModule>) -> Self {
        Self { verifier }
    }

    /// OID4VP endpoints called by the peer's wallet; no OAuth guard.
    pub fn external(self) -> Router {
        Router::new()
            .route(verifier::PD, get(Self::vp_definition))
            .route(verifier::VERIFY, post(Self::verify))
            .route(
                verifier::REQUEST,
                get(Self::request_object).post(Self::request_object_post),
            )
            .with_state(self.verifier)
    }

    // ==========================================================================================
    // External requests: peers, authorities and wallets, authenticated by the protocol (no user)
    // ==========================================================================================

    async fn vp_definition(
        State(verifier): State<Arc<dyn VerifierModule>>,
        Path(state): Path<String>,
    ) -> AppResult<Json<VPDef>> {
        Ok(Json(verifier.get_vpd(state).await?))
    }

    async fn verify(
        State(verifier): State<Arc<dyn VerifierModule>>,
        Path(state): Path<String>,
        payload: Result<Form<VerifyPayload>, FormRejection>,
    ) -> AppResult {
        let payload = extract_form_payload(payload)?;

        let refused = payload.refusal().is_some();
        let (verified, redirect_uri, reason) = match verifier.verify(state, payload).await? {
            InteractionFinishResponse::Success(uri) => (true, uri, None),
            InteractionFinishResponse::Failure(uri, reason) => (false, uri, reason),
        };
        if verified || refused {
            return Ok((StatusCode::OK, Json(DirectPostResponse::new(redirect_uri))).into_response());
        }
        let error = json!({
            "error": "invalid_request",
            "error_description": reason.as_deref().unwrap_or("the presentation did not verify"),
            "redirect_uri": redirect_uri,
        });
        Ok((StatusCode::BAD_REQUEST, Json(error)).into_response())
    }

    /// The Request Object of a request by reference (OpenID4VP 1.0 §5.10), fetched with GET.
    async fn request_object(
        State(verifier): State<Arc<dyn VerifierModule>>,
        Path(state): Path<String>,
    ) -> AppResult {
        Self::serve_request_object(verifier, state, None).await
    }

    /// The same with `request_uri_method=post`: the wallet sends its metadata and a
    /// `wallet_nonce` that the Request Object echoes.
    async fn request_object_post(
        State(verifier): State<Arc<dyn VerifierModule>>,
        Path(state): Path<String>,
        payload: Result<Form<RequestUriPost>, FormRejection>,
    ) -> AppResult {
        let payload = extract_form_payload(payload)?;
        Self::serve_request_object(verifier, state, payload.wallet_nonce).await
    }

    async fn serve_request_object(
        verifier: Arc<dyn VerifierModule>,
        state: String,
        wallet_nonce: Option<String>,
    ) -> AppResult {
        let jwt = verifier.request_object(state, wallet_nonce).await?;
        Ok(([(CONTENT_TYPE, REQUEST_OBJECT_MEDIA_TYPE)], jwt).into_response())
    }
}
