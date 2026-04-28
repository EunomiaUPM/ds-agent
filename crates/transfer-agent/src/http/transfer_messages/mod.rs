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

use crate::entities::transfer_messages::{NewTransferMessageDto, TransferAgentMessagesTrait};
use axum::{
    extract::{rejection::JsonRejection, FromRef, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use common::config::services::TransferConfig;
use serde::Deserialize;
use std::sync::Arc;
use ymir::utils::{extract_path_urn, extract_payload};

#[derive(Clone)]
pub struct TransferAgentMessagesRouter {
    service: Arc<dyn TransferAgentMessagesTrait>,
    config: Arc<TransferConfig>,
}

#[derive(Deserialize)]
pub struct PaginationParams {
    pub limit: Option<u64>,
    pub page: Option<u64>,
}

impl FromRef<TransferAgentMessagesRouter> for Arc<dyn TransferAgentMessagesTrait> {
    fn from_ref(state: &TransferAgentMessagesRouter) -> Self {
        state.service.clone()
    }
}

impl FromRef<TransferAgentMessagesRouter> for Arc<TransferConfig> {
    fn from_ref(state: &TransferAgentMessagesRouter) -> Self {
        state.config.clone()
    }
}

impl TransferAgentMessagesRouter {
    pub fn new(service: Arc<dyn TransferAgentMessagesTrait>, config: Arc<TransferConfig>) -> Self {
        Self { service, config }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route(
                "/",
                get(Self::handle_get_all_messages).post(Self::handle_create_message),
            )
            .route(
                "/{id}",
                get(Self::handle_get_message_by_id).delete(Self::handle_delete_message),
            )
            .route(
                "/process/{process_id}",
                get(Self::handle_get_messages_by_process_id),
            )
            .with_state(self)
    }

    async fn handle_get_all_messages(
        State(state): State<TransferAgentMessagesRouter>,
        Query(params): Query<PaginationParams>,
    ) -> impl IntoResponse {
        match state
            .service
            .get_all_transfer_messages(params.limit, params.page)
            .await
        {
            Ok(messages) => (StatusCode::OK, Json(messages)).into_response(),
            Err(err) => err.into_response(),
        }
    }

    async fn handle_create_message(
        State(state): State<TransferAgentMessagesRouter>,
        input: Result<Json<NewTransferMessageDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.service.create_transfer_message(&input).await {
            Ok(created) => (StatusCode::CREATED, Json(created)).into_response(),
            Err(err) => err.into_response(),
        }
    }

    async fn handle_get_message_by_id(
        State(state): State<TransferAgentMessagesRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        let id_urn = match extract_path_urn(&id) {
            Ok(urn) => urn,
            Err(err) => return err.into_response(),
        };
        match state.service.get_transfer_message_by_id(&id_urn).await {
            Ok(message) => (StatusCode::OK, Json(message)).into_response(),
            Err(err) => err.into_response(),
        }
    }

    async fn handle_delete_message(
        State(state): State<TransferAgentMessagesRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        let id_urn = match extract_path_urn(&id) {
            Ok(urn) => urn,
            Err(err) => return err.into_response(),
        };
        match state.service.delete_transfer_message(&id_urn).await {
            Ok(_) => (StatusCode::NO_CONTENT).into_response(),
            Err(err) => err.into_response(),
        }
    }

    async fn handle_get_messages_by_process_id(
        State(state): State<TransferAgentMessagesRouter>,
        Path(process_id): Path<String>,
    ) -> impl IntoResponse {
        let process_urn = match extract_path_urn(&process_id) {
            Ok(urn) => urn,
            Err(err) => return err.into_response(),
        };
        match state.service.get_messages_by_process_id(&process_urn).await {
            Ok(messages) => (StatusCode::OK, Json(messages)).into_response(),
            Err(err) => err.into_response(),
        }
    }
}
