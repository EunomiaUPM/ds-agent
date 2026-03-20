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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use crate::protocols::dsp::orchestrator::rpc::types::{
    RpcCatalogRequestMessageDto, RpcDatasetRequestMessageDto,
};
use crate::protocols::dsp::orchestrator::OrchestratorTrait;
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRef, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use std::sync::Arc;
use ymir::utils::extract_payload;

#[derive(Clone)]
pub struct RpcRouter {
    orchestrator: Arc<dyn OrchestratorTrait>,
}

impl FromRef<RpcRouter> for Arc<dyn OrchestratorTrait> {
    fn from_ref(state: &RpcRouter) -> Self {
        state.orchestrator.clone()
    }
}

impl RpcRouter {
    pub fn new(orchestrator: Arc<dyn OrchestratorTrait>) -> Self {
        Self { orchestrator }
    }
    pub fn router(self) -> Router {
        Router::new()
            .route("/rpc/setup-catalog-request", post(Self::handle_rpc_catalog_request))
            .route("/rpc/setup-dataset-request", post(Self::handle_rpc_dataset_request))
            .with_state(self)
    }

    async fn handle_rpc_catalog_request(
        State(state): State<RpcRouter>,
        input: Result<Json<RpcCatalogRequestMessageDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.orchestrator.get_rpc_service().setup_catalog_request_rpc(&input).await {
            Ok(catalog) => (StatusCode::OK, Json(catalog)).into_response(),
            Err(err) => err.into_response(),
        }
    }
    async fn handle_rpc_dataset_request(
        State(state): State<RpcRouter>,
        input: Result<Json<RpcDatasetRequestMessageDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.orchestrator.get_rpc_service().setup_dataset_request_rpc(&input).await {
            Ok(dataset) => (StatusCode::OK, Json(dataset)).into_response(),
            Err(err) => err.into_response(),
        }
    }
}
