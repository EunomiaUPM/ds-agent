/*
 *
 * * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 * *
 * * This program is free software: you can redistribute it and/or modify
 * * it under the terms of the GNU General Public License as published by
 * * the Free Software Foundation, either version 3 of the License, or
 * * (at your option) any later version.
 * *
 * * This program is distributed in the hope that it will be useful,
 * * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * * GNU General Public License for more details.
 * *
 * * You should have received a copy of the GNU General Public License
 * * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::entities::negotiation_message::{
    NegotiationAgentMessagesTrait, NewNegotiationMessageDto,
};
use crate::errors::error_adapter::CustomToResponse;
use crate::http::common::{extract_payload, parse_urn};
use axum::{
    Json, Router,
    extract::{FromRef, Path, Query, State, rejection::JsonRejection},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use rainbow_common::config::services::ContractsConfig;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone)]
pub struct NegotiationAgentMessagesRouter {
    service: Arc<dyn NegotiationAgentMessagesTrait>,
    config: Arc<ContractsConfig>,
}

#[derive(Deserialize)]
pub struct PaginationParams {
    pub limit: Option<u64>,
    pub page: Option<u64>,
}

impl FromRef<NegotiationAgentMessagesRouter> for Arc<dyn NegotiationAgentMessagesTrait> {
    fn from_ref(state: &NegotiationAgentMessagesRouter) -> Self {
        state.service.clone()
    }
}

impl FromRef<NegotiationAgentMessagesRouter> for Arc<ContractsConfig> {
    fn from_ref(state: &NegotiationAgentMessagesRouter) -> Self {
        state.config.clone()
    }
}

impl NegotiationAgentMessagesRouter {
    pub fn new(
        service: Arc<dyn NegotiationAgentMessagesTrait>,
        config: Arc<ContractsConfig>,
    ) -> Self {
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
            .route("/process/{process_id}", get(Self::handle_get_messages_by_process_id))
            .with_state(self)
    }

    async fn handle_get_all_messages(
        State(state): State<NegotiationAgentMessagesRouter>,
        Query(params): Query<PaginationParams>,
    ) -> impl IntoResponse {
        match state.service.get_all_negotiation_messages(params.limit, params.page).await {
            Ok(messages) => (StatusCode::OK, Json(messages)).into_response(),
            Err(err) => err.to_response(),
        }
    }

    async fn handle_create_message(
        State(state): State<NegotiationAgentMessagesRouter>,
        input: Result<Json<NewNegotiationMessageDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e,
        };
        match state.service.create_negotiation_message(&input).await {
            Ok(created) => (StatusCode::CREATED, Json(created)).into_response(),
            Err(err) => err.to_response(),
        }
    }

    async fn handle_get_message_by_id(
        State(state): State<NegotiationAgentMessagesRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        let id_urn = match parse_urn(&id) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };
        match state.service.get_negotiation_message_by_id(&id_urn).await {
            Ok(Some(message)) => (StatusCode::OK, Json(message)).into_response(),
            Ok(None) => (StatusCode::NOT_FOUND).into_response(),
            Err(err) => err.to_response(),
        }
    }

    async fn handle_delete_message(
        State(state): State<NegotiationAgentMessagesRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        let id_urn = match parse_urn(&id) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };
        match state.service.delete_negotiation_message(&id_urn).await {
            Ok(_) => (StatusCode::NO_CONTENT).into_response(),
            Err(err) => err.to_response(),
        }
    }

    async fn handle_get_messages_by_process_id(
        State(state): State<NegotiationAgentMessagesRouter>,
        Path(process_id): Path<String>,
    ) -> impl IntoResponse {
        let process_urn = match parse_urn(&process_id) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };
        match state.service.get_messages_by_process_id(&process_urn).await {
            Ok(messages) => (StatusCode::OK, Json(messages)).into_response(),
            Err(err) => err.to_response(),
        }
    }
}
