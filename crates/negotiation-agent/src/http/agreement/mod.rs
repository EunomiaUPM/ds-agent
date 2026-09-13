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

use crate::entities::agreement::{
    AgreementDto, EditAgreementDto, NegotiationAgentAgreementsTrait,
    NewAgreementDto,
};
use crate::entities::filters::AgreementFilter;
use crate::errors::error_adapter::CustomToResponse;
use crate::http::common::{extract_payload, parse_urn};
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRef, Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use common::batch_requests::BatchRequests;
use common::config::services::ContractsConfig;
use common::query::QuerySpec;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone)]
pub struct NegotiationAgentAgreementsRouter {
    service: Arc<dyn NegotiationAgentAgreementsTrait>,
    config: Arc<ContractsConfig>,
}

pub use common::paginated_spec::PaginationParams;
pub type AgreementQuery = QuerySpec<AgreementFilter>;

impl FromRef<NegotiationAgentAgreementsRouter> for Arc<dyn NegotiationAgentAgreementsTrait> {
    fn from_ref(state: &NegotiationAgentAgreementsRouter) -> Self {
        state.service.clone()
    }
}

impl FromRef<NegotiationAgentAgreementsRouter> for Arc<ContractsConfig> {
    fn from_ref(state: &NegotiationAgentAgreementsRouter) -> Self {
        state.config.clone()
    }
}

impl NegotiationAgentAgreementsRouter {
    pub fn new(
        service: Arc<dyn NegotiationAgentAgreementsTrait>,
        config: Arc<ContractsConfig>,
    ) -> Self {
        Self { service, config }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route(
                "/",
                get(Self::handle_get_all_agreements).post(Self::handle_create_agreement),
            )
            .route("/batch", post(Self::handle_get_batch_agreements))
            .route(
                "/{id}",
                get(Self::handle_get_agreement_by_id)
                    .put(Self::handle_put_agreement)
                    .delete(Self::handle_delete_agreement),
            )
            .route(
                "/process/{process_id}",
                get(Self::handle_get_agreement_by_negotiation_process),
            )
            .route(
                "/message/{message_id}",
                get(Self::handle_get_agreement_by_negotiation_message),
            )
            .route(
                "/assignee/{assignee}",
                get(Self::handle_get_agreement_by_assignee),
            )
            .route(
                "/assigner/{assigner}",
                get(Self::handle_get_agreement_by_assigner),
            )
            .with_state(self)
    }

    async fn handle_get_all_agreements(
        State(state): State<NegotiationAgentAgreementsRouter>,
        Query(query): Query<AgreementQuery>,
    ) -> impl IntoResponse {
        match state
            .service
            .get_all_agreements(&query.filter, &query.page, &query.sort)
            .await
        {
            Ok(agreements) => (StatusCode::OK, Json(agreements)).into_response(),
            Err(err) => err.to_response(),
        }
    }

    async fn handle_get_batch_agreements(
        State(state): State<NegotiationAgentAgreementsRouter>,
        input: Result<Json<BatchRequests>, JsonRejection>,
    ) -> impl IntoResponse {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e,
        };
        match state.service.get_batch_agreements(&input.ids).await {
            Ok(agreements) => (StatusCode::OK, Json(agreements)).into_response(),
            Err(err) => err.to_response(),
        }
    }

    async fn handle_create_agreement(
        State(state): State<NegotiationAgentAgreementsRouter>,
        input: Result<Json<NewAgreementDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e,
        };
        match state.service.create_agreement(&input).await {
            Ok(created) => (StatusCode::CREATED, Json(created)).into_response(),
            Err(err) => err.to_response(),
        }
    }

    async fn handle_get_agreement_by_id(
        State(state): State<NegotiationAgentAgreementsRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        let id_urn = match parse_urn(&id) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };
        match state.service.get_agreement_by_id(&id_urn).await {
            Ok(Some(agreement)) => (StatusCode::OK, Json(agreement)).into_response(),
            Ok(None) => (StatusCode::NOT_FOUND).into_response(),
            Err(err) => err.to_response(),
        }
    }

    async fn handle_put_agreement(
        State(state): State<NegotiationAgentAgreementsRouter>,
        Path(id): Path<String>,
        input: Result<Json<EditAgreementDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let id_urn = match parse_urn(&id) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e,
        };
        match state.service.put_agreement(&id_urn, &input).await {
            Ok(updated) => (StatusCode::OK, Json(updated)).into_response(),
            Err(err) => err.to_response(),
        }
    }

    async fn handle_delete_agreement(
        State(state): State<NegotiationAgentAgreementsRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        let id_urn = match parse_urn(&id) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };
        match state.service.delete_agreement(&id_urn).await {
            Ok(_) => (StatusCode::NO_CONTENT).into_response(),
            Err(err) => err.to_response(),
        }
    }

    async fn handle_get_agreement_by_negotiation_process(
        State(state): State<NegotiationAgentAgreementsRouter>,
        Path(process_id): Path<String>,
    ) -> impl IntoResponse {
        let process_urn = match parse_urn(&process_id) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };
        match state
            .service
            .get_agreement_by_negotiation_process(&process_urn)
            .await
        {
            Ok(Some(agreement)) => (StatusCode::OK, Json(agreement)).into_response(),
            Ok(None) => (StatusCode::NOT_FOUND).into_response(),
            Err(err) => err.to_response(),
        }
    }

    async fn handle_get_agreement_by_negotiation_message(
        State(state): State<NegotiationAgentAgreementsRouter>,
        Path(message_id): Path<String>,
    ) -> impl IntoResponse {
        let message_urn = match parse_urn(&message_id) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };
        match state
            .service
            .get_agreement_by_negotiation_message(&message_urn)
            .await
        {
            Ok(Some(agreement)) => (StatusCode::OK, Json(agreement)).into_response(),
            Ok(None) => (StatusCode::NOT_FOUND).into_response(),
            Err(err) => err.to_response(),
        }
    }

    async fn handle_get_agreement_by_assignee(
        State(state): State<NegotiationAgentAgreementsRouter>,
        Path(assignee): Path<String>,
    ) -> impl IntoResponse {
        match state.service.get_agreements_by_assignee(&assignee).await {
            Ok(agreements) => (StatusCode::OK, Json(agreements)).into_response(),
            Err(err) => err.to_response(),
        }
    }

    async fn handle_get_agreement_by_assigner(
        State(state): State<NegotiationAgentAgreementsRouter>,
        Path(assigner): Path<String>,
    ) -> impl IntoResponse {
        match state.service.get_agreements_by_assigner(&assigner).await {
            Ok(agreements) => (StatusCode::OK, Json(agreements)).into_response(),
            Err(err) => err.to_response(),
        }
    }
}
