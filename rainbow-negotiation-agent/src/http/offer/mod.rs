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

use crate::entities::offer::{NegotiationAgentOffersTrait, NewOfferDto};
use crate::errors::error_adapter::CustomToResponse;
use crate::http::common::{extract_payload, parse_urn};
use axum::{
    Json, Router,
    extract::{FromRef, Path, Query, State, rejection::JsonRejection},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use rainbow_common::batch_requests::BatchRequests;
use rainbow_common::config::services::ContractsConfig;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone)]
pub struct NegotiationAgentOffersRouter {
    service: Arc<dyn NegotiationAgentOffersTrait>,
    config: Arc<ContractsConfig>,
}

#[derive(Deserialize)]
pub struct PaginationParams {
    pub limit: Option<u64>,
    pub page: Option<u64>,
}

impl FromRef<NegotiationAgentOffersRouter> for Arc<dyn NegotiationAgentOffersTrait> {
    fn from_ref(state: &NegotiationAgentOffersRouter) -> Self {
        state.service.clone()
    }
}

impl FromRef<NegotiationAgentOffersRouter> for Arc<ContractsConfig> {
    fn from_ref(state: &NegotiationAgentOffersRouter) -> Self {
        state.config.clone()
    }
}

impl NegotiationAgentOffersRouter {
    pub fn new(
        service: Arc<dyn NegotiationAgentOffersTrait>,
        config: Arc<ContractsConfig>,
    ) -> Self {
        Self { service, config }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/", get(Self::handle_get_all_offers).post(Self::handle_create_offer))
            .route("/batch", post(Self::handle_get_batch_offers))
            .route(
                "/{id}",
                get(Self::handle_get_offer_by_id).delete(Self::handle_delete_offer),
            )
            .route(
                "/process/{process_id}",
                get(Self::handle_get_offers_by_negotiation_process),
            )
            .route(
                "/message/{message_id}",
                get(Self::handle_get_offer_by_negotiation_message),
            )
            .route("/offer-id/{offer_id}", get(Self::handle_get_offer_by_offer_id))
            .with_state(self)
    }

    async fn handle_get_all_offers(
        State(state): State<NegotiationAgentOffersRouter>,
        Query(params): Query<PaginationParams>,
    ) -> impl IntoResponse {
        match state.service.get_all_offers(params.limit, params.page).await {
            Ok(offers) => (StatusCode::OK, Json(offers)).into_response(),
            Err(err) => err.to_response(),
        }
    }

    async fn handle_get_batch_offers(
        State(state): State<NegotiationAgentOffersRouter>,
        input: Result<Json<BatchRequests>, JsonRejection>,
    ) -> impl IntoResponse {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e,
        };
        match state.service.get_batch_offers(&input.ids).await {
            Ok(offers) => (StatusCode::OK, Json(offers)).into_response(),
            Err(err) => err.to_response(),
        }
    }

    async fn handle_create_offer(
        State(state): State<NegotiationAgentOffersRouter>,
        input: Result<Json<NewOfferDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e,
        };
        match state.service.create_offer(&input).await {
            Ok(created) => (StatusCode::CREATED, Json(created)).into_response(),
            Err(err) => err.to_response(),
        }
    }

    async fn handle_get_offer_by_id(
        State(state): State<NegotiationAgentOffersRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        let id_urn = match parse_urn(&id) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };
        match state.service.get_offer_by_id(&id_urn).await {
            Ok(Some(offer)) => (StatusCode::OK, Json(offer)).into_response(),
            Ok(None) => (StatusCode::NOT_FOUND).into_response(),
            Err(err) => err.to_response(),
        }
    }

    async fn handle_delete_offer(
        State(state): State<NegotiationAgentOffersRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        let id_urn = match parse_urn(&id) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };
        match state.service.delete_offer(&id_urn).await {
            Ok(_) => (StatusCode::NO_CONTENT).into_response(),
            Err(err) => err.to_response(),
        }
    }

    async fn handle_get_offers_by_negotiation_process(
        State(state): State<NegotiationAgentOffersRouter>,
        Path(process_id): Path<String>,
    ) -> impl IntoResponse {
        let process_urn = match parse_urn(&process_id) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };
        match state.service.get_offers_by_negotiation_process(&process_urn).await {
            Ok(offers) => (StatusCode::OK, Json(offers)).into_response(),
            Err(err) => err.to_response(),
        }
    }

    async fn handle_get_offer_by_negotiation_message(
        State(state): State<NegotiationAgentOffersRouter>,
        Path(message_id): Path<String>,
    ) -> impl IntoResponse {
        let message_urn = match parse_urn(&message_id) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };
        match state.service.get_offer_by_negotiation_message(&message_urn).await {
            Ok(Some(offer)) => (StatusCode::OK, Json(offer)).into_response(),
            Ok(None) => (StatusCode::NOT_FOUND).into_response(),
            Err(err) => err.to_response(),
        }
    }

    async fn handle_get_offer_by_offer_id(
        State(state): State<NegotiationAgentOffersRouter>,
        Path(offer_id): Path<String>,
    ) -> impl IntoResponse {
        let id_urn = match parse_urn(&offer_id) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };
        match state.service.get_offer_by_offer_id(&id_urn).await {
            Ok(Some(offer)) => (StatusCode::OK, Json(offer)).into_response(),
            Ok(None) => (StatusCode::NOT_FOUND).into_response(),
            Err(err) => err.to_response(),
        }
    }
}
