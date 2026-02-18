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

use std::sync::Arc;

use axum::extract::rejection::FormRejection;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Form, Json, Router};
use ymir::errors::CustomToResponse;
use ymir::types::verifying::VerifyPayload;

use crate::ssi::core::traits::CoreVerifierTrait;

pub struct VerifierRouter {
    verifier: Arc<dyn CoreVerifierTrait>,
}

impl VerifierRouter {
    pub fn new(verifier: Arc<dyn CoreVerifierTrait>) -> Self {
        Self { verifier }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/pd/{state}", get(Self::vp_definition))
            .route("/verify/{state}", post(Self::verify))
            .with_state(self.verifier)
    }

    async fn vp_definition(
        State(verifier): State<Arc<dyn CoreVerifierTrait>>,
        Path(state): Path<String>,
    ) -> impl IntoResponse {
        match verifier.get_vpd(state).await {
            Ok(data) => (StatusCode::OK, Json(data)).into_response(),
            Err(e) => e.to_response(),
        }
    }

    async fn verify(
        State(verifier): State<Arc<dyn CoreVerifierTrait>>,
        Path(state): Path<String>,
        payload: Result<Form<VerifyPayload>, FormRejection>,
    ) -> impl IntoResponse {
        let payload = match payload {
            Ok(Form(data)) => data,
            Err(e) => return e.into_response(),
        };

        match verifier.verify(state, payload).await {
            Ok(Some(uri)) => (StatusCode::OK, uri).into_response(),
            Ok(None) => StatusCode::OK.into_response(),
            Err(e) => e.to_response(),
        }
    }
}
