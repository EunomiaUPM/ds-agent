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

use crate::entities::odrl_policies::{NewOdrlPolicyDto, OdrlPolicyEntityTrait};
use crate::http::common::to_camel_case::ToCamelCase;
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRef, Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use common::batch_requests::BatchRequests;
use common::config::services::CatalogConfig;
use common::errors::CommonErrors;
use serde::Deserialize;
use std::sync::Arc;
use ymir::utils::{extract_path_urn, extract_payload};

#[derive(Clone)]
pub struct OdrlOfferEntityRouter {
    service: Arc<dyn OdrlPolicyEntityTrait>,
    config: Arc<CatalogConfig>,
}

#[derive(Deserialize)]
pub struct PaginationParams {
    pub limit: Option<u64>,
    pub page: Option<u64>,
}

impl FromRef<OdrlOfferEntityRouter> for Arc<dyn OdrlPolicyEntityTrait> {
    fn from_ref(state: &OdrlOfferEntityRouter) -> Self {
        state.service.clone()
    }
}

impl FromRef<OdrlOfferEntityRouter> for Arc<CatalogConfig> {
    fn from_ref(state: &OdrlOfferEntityRouter) -> Self {
        state.config.clone()
    }
}

impl OdrlOfferEntityRouter {
    pub fn new(service: Arc<dyn OdrlPolicyEntityTrait>, config: Arc<CatalogConfig>) -> Self {
        Self { service, config }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/", get(Self::handle_get_all_odrl_offers))
            .route(
                "/entity/{entity_id}",
                get(Self::handle_get_all_odrl_offers_by_entity),
            )
            .route("/", post(Self::handle_create_odrl_offer))
            .route("/batch", post(Self::handle_get_batch_odrl_offers))
            .route("/{id}", get(Self::handle_get_odrl_offer_by_id))
            .route("/{id}", delete(Self::handle_delete_odrl_offer_by_id))
            .route(
                "/entity/{entity_id}",
                delete(Self::handle_delete_odrl_offers_by_entity),
            )
            .with_state(self)
    }

    async fn handle_get_all_odrl_offers(
        State(state): State<OdrlOfferEntityRouter>,
        Query(params): Query<PaginationParams>,
    ) -> impl IntoResponse {
        match state
            .service
            .get_all_odrl_offers(params.limit, params.page)
            .await
        {
            Ok(offers) => (StatusCode::OK, Json(ToCamelCase(offers))).into_response(),
            Err(e) => return e.into_response(),
        }
    }
    async fn handle_get_batch_odrl_offers(
        State(state): State<OdrlOfferEntityRouter>,
        input: Result<Json<BatchRequests>, JsonRejection>,
    ) -> impl IntoResponse {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.service.get_batch_odrl_offers(&input.ids).await {
            Ok(offers) => (StatusCode::OK, Json(ToCamelCase(offers))).into_response(),
            Err(e) => return e.into_response(),
        }
    }
    async fn handle_get_all_odrl_offers_by_entity(
        State(state): State<OdrlOfferEntityRouter>,
        Path(entity_id): Path<String>,
    ) -> impl IntoResponse {
        let entity_id = match extract_path_urn(&entity_id) {
            Ok(urn) => urn,
            Err(resp) => return resp.into_response(),
        };
        match state
            .service
            .get_all_odrl_offers_by_entity(&entity_id)
            .await
        {
            Ok(offers) => (StatusCode::OK, Json(ToCamelCase(offers))).into_response(),
            Err(e) => return e.into_response(),
        }
    }
    async fn handle_get_odrl_offer_by_id(
        State(state): State<OdrlOfferEntityRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        let id_urn = match extract_path_urn(&id) {
            Ok(urn) => urn,
            Err(resp) => return resp.into_response(),
        };
        match state.service.get_odrl_offer_by_id(&id_urn).await {
            Ok(Some(offer)) => (StatusCode::OK, Json(ToCamelCase(offer))).into_response(),
            Ok(None) => {
                let err = CommonErrors::missing_resource_new(id.as_str(), "Odrl policy not found");
                err.into_response()
            }
            Err(e) => return e.into_response(),
        }
    }
    async fn handle_create_odrl_offer(
        State(state): State<OdrlOfferEntityRouter>,
        input: Result<Json<NewOdrlPolicyDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.service.create_odrl_offer(&input).await {
            Ok(offer) => (StatusCode::OK, Json(ToCamelCase(offer))).into_response(),
            Err(e) => return e.into_response(),
        }
    }

    async fn handle_delete_odrl_offer_by_id(
        State(state): State<OdrlOfferEntityRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        let id_urn = match extract_path_urn(&id) {
            Ok(urn) => urn,
            Err(resp) => return resp.into_response(),
        };
        match state.service.delete_odrl_offer_by_id(&id_urn).await {
            Ok(_) => StatusCode::ACCEPTED.into_response(),
            Err(e) => return e.into_response(),
        }
    }
    async fn handle_delete_odrl_offers_by_entity(
        State(state): State<OdrlOfferEntityRouter>,
        Path(entity): Path<String>,
    ) -> impl IntoResponse {
        let id_urn = match extract_path_urn(&entity) {
            Ok(urn) => urn,
            Err(resp) => return resp.into_response(),
        };
        match state.service.delete_odrl_offers_by_entity(&id_urn).await {
            Ok(_) => StatusCode::ACCEPTED.into_response(),
            Err(e) => return e.into_response(),
        }
    }
}
