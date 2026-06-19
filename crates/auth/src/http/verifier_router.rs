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

use std::sync::Arc;

use axum::extract::rejection::FormRejection;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Form, Json, Router};
use ymir::errors::AppResult;
use ymir::types::vcs::VPDef;
use ymir::types::verifying::VerifyPayload;
use ymir::utils::extract_form_payload;

use crate::modules::VerifierModuleTrait;

pub struct VerifierRouter {
    verifier: Arc<dyn VerifierModuleTrait>,
}

impl VerifierRouter {
    pub fn new(verifier: Arc<dyn VerifierModuleTrait>) -> Self {
        Self { verifier }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/pd/{state}", get(Self::vp_definition))
            .route("/verify/{state}", post(Self::verify))
            .with_state(self.verifier)
    }

    async fn vp_definition(
        State(verifier): State<Arc<dyn VerifierModuleTrait>>,
        Path(state): Path<String>,
    ) -> AppResult<Json<VPDef>> {
        Ok(Json(verifier.get_vpd(state).await?))
    }

    async fn verify(
        State(verifier): State<Arc<dyn VerifierModuleTrait>>,
        Path(state): Path<String>,
        payload: Result<Form<VerifyPayload>, FormRejection>,
    ) -> AppResult {
        let payload = extract_form_payload(payload)?;
        Ok(match verifier.verify(state, payload).await {
            Ok(Some(uri)) => uri.into_response(),
            Ok(None) => ().into_response(),
            Err(e) => e.into_response(),
        })
    }
}
