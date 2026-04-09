/*
 *
 *  * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::entities::transfer_process::{
    EditTransferProcessDto, NewTransferProcessDto, TransferAgentProcessesTrait,
};
use crate::http::common::{extract_payload, parse_urn};
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRef, Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use common::batch_requests::BatchRequests;
use common::config::services::TransferConfig;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone)]
pub struct TransferAgentProcessesRouter {
    service: Arc<dyn TransferAgentProcessesTrait>,
    config: Arc<TransferConfig>,
}

#[derive(Deserialize)]
pub struct PaginationParams {
    pub limit: Option<u64>,
    pub page: Option<u64>,
}

impl FromRef<TransferAgentProcessesRouter> for Arc<dyn TransferAgentProcessesTrait> {
    fn from_ref(state: &TransferAgentProcessesRouter) -> Self {
        state.service.clone()
    }
}

impl FromRef<TransferAgentProcessesRouter> for Arc<TransferConfig> {
    fn from_ref(state: &TransferAgentProcessesRouter) -> Self {
        state.config.clone()
    }
}

impl TransferAgentProcessesRouter {
    pub fn new(service: Arc<dyn TransferAgentProcessesTrait>, config: Arc<TransferConfig>) -> Self {
        Self { service, config }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route(
                "/",
                get(Self::handle_get_all_processes).post(Self::handle_create_process),
            )
            .route("/batch", post(Self::handle_get_batch_processes))
            .route(
                "/{id}",
                get(Self::handle_get_process_by_id)
                    .put(Self::handle_put_process)
                    .delete(Self::handle_delete_process),
            )
            .route(
                "/{id}/key/{key_id}",
                get(Self::handle_get_process_by_key_id),
            )
            .with_state(self)
    }

    async fn handle_get_all_processes(
        State(state): State<TransferAgentProcessesRouter>,
        Query(params): Query<PaginationParams>,
    ) -> impl IntoResponse {
        match state
            .service
            .get_all_transfer_processes(params.limit, params.page)
            .await
        {
            Ok(processes) => (StatusCode::OK, Json(processes)).into_response(),
            Err(err) => err.into_response(),
        }
    }

    async fn handle_create_process(
        State(state): State<TransferAgentProcessesRouter>,
        input: Result<Json<NewTransferProcessDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e,
        };
        match state.service.create_transfer_process(&input).await {
            Ok(created_process) => (StatusCode::CREATED, Json(created_process)).into_response(),
            Err(err) => err.into_response(),
        }
    }

    async fn handle_get_batch_processes(
        State(state): State<TransferAgentProcessesRouter>,
        input: Result<Json<BatchRequests>, JsonRejection>,
    ) -> impl IntoResponse {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e,
        };
        match state.service.get_batch_transfer_processes(&input.ids).await {
            Ok(processes) => (StatusCode::OK, Json(processes)).into_response(),
            Err(err) => err.into_response(),
        }
    }

    async fn handle_get_process_by_id(
        State(state): State<TransferAgentProcessesRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        let id_urn = match parse_urn(&id) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };
        match state.service.get_transfer_process_by_id(&id_urn).await {
            Ok(process) => (StatusCode::OK, Json(process)).into_response(),
            Err(err) => err.into_response(),
        }
    }

    async fn handle_put_process(
        State(state): State<TransferAgentProcessesRouter>,
        Path(id): Path<String>,
        input: Result<Json<EditTransferProcessDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let id_urn = match parse_urn(&id) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e,
        };
        match state.service.put_transfer_process(&id_urn, &input).await {
            Ok(updated_process) => (StatusCode::OK, Json(updated_process)).into_response(),
            Err(err) => err.into_response(),
        }
    }

    async fn handle_delete_process(
        State(state): State<TransferAgentProcessesRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        let id_urn = match parse_urn(&id) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };
        match state.service.delete_transfer_process(&id_urn).await {
            Ok(_) => (StatusCode::NO_CONTENT).into_response(),
            Err(err) => err.into_response(),
        }
    }

    async fn handle_get_process_by_key_id(
        State(state): State<TransferAgentProcessesRouter>,
        Path((id, key_id)): Path<(String, String)>,
    ) -> impl IntoResponse {
        let id_urn = match parse_urn(&id) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };
        match state
            .service
            .get_transfer_process_by_key_id(&key_id, &id_urn)
            .await
        {
            Ok(process) => (StatusCode::OK, Json(process)).into_response(),
            Err(err) => err.into_response(),
        }
    }
}
