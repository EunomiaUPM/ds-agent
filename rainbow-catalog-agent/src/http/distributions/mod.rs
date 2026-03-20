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

use crate::entities::distributions::{
    DistributionEntityTrait, EditDistributionDto, NewDistributionDto,
};
use crate::http::common::to_camel_case::ToCamelCase;
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRef, Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use rainbow_common::batch_requests::BatchRequests;
use rainbow_common::config::services::CatalogConfig;
use rainbow_common::errors::CommonErrors;
use serde::Deserialize;
use std::str::FromStr;
use std::sync::Arc;
use ymir::utils::{extract_path_urn, extract_payload};

#[derive(Clone)]
pub struct DistributionEntityRouter {
    service: Arc<dyn DistributionEntityTrait>,
    config: Arc<CatalogConfig>,
}

#[derive(Deserialize)]
pub struct PaginationParams {
    pub limit: Option<u64>,
    pub page: Option<u64>,
}

impl FromRef<DistributionEntityRouter> for Arc<dyn DistributionEntityTrait> {
    fn from_ref(state: &DistributionEntityRouter) -> Self {
        state.service.clone()
    }
}

impl FromRef<DistributionEntityRouter> for Arc<CatalogConfig> {
    fn from_ref(state: &DistributionEntityRouter) -> Self {
        state.config.clone()
    }
}

impl DistributionEntityRouter {
    pub fn new(service: Arc<dyn DistributionEntityTrait>, config: Arc<CatalogConfig>) -> Self {
        Self { service, config }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/", get(Self::handle_get_all_distributions))
            .route("/dataset/{id}", get(Self::handle_get_distributions_by_dataset_id))
            .route(
                "/dataset/{id}/format/{format}",
                get(Self::handle_get_distribution_by_dataset_id_and_dct_format),
            )
            .route("/", post(Self::handle_create_distribution))
            .route("/batch", post(Self::handle_get_batch_distributions))
            .route("/{id}", get(Self::handle_get_distribution_by_id))
            .route("/{id}", put(Self::handle_put_distribution_by_id))
            .route("/{id}", delete(Self::handle_delete_distribution_by_id))
            .with_state(self)
    }

    async fn handle_get_all_distributions(
        State(state): State<DistributionEntityRouter>,
        Query(params): Query<PaginationParams>,
    ) -> impl IntoResponse {
        match state.service.get_all_distributions(params.limit, params.page).await {
            Ok(distributions) => (StatusCode::OK, Json(ToCamelCase(distributions))).into_response(),
            Err(e) => return e.into_response(),
        }
    }
    async fn handle_get_batch_distributions(
        State(state): State<DistributionEntityRouter>,
        input: Result<Json<BatchRequests>, JsonRejection>,
    ) -> impl IntoResponse {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.service.get_batch_distributions(&input.ids).await {
            Ok(distributions) => (StatusCode::OK, Json(ToCamelCase(distributions))).into_response(),
            Err(e) => return e.into_response(),
        }
    }
    async fn handle_get_distributions_by_dataset_id(
        State(state): State<DistributionEntityRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        let id_urn = match extract_path_urn(&id) {
            Ok(urn) => urn,
            Err(resp) => return resp.into_response(),
        };
        match state.service.get_distributions_by_dataset_id(&id_urn).await {
            Ok(distributions) => (StatusCode::OK, Json(ToCamelCase(distributions))).into_response(),
            Err(e) => return e.into_response(),
        }
    }
    async fn handle_get_distribution_by_dataset_id_and_dct_format(
        State(state): State<DistributionEntityRouter>,
        Path((id, dct_format)): Path<(String, String)>,
    ) -> impl IntoResponse {
        let id_urn = match extract_path_urn(&id) {
            Ok(urn) => urn,
            Err(resp) => return resp.into_response(),
        };
        match state
            .service
            .get_distribution_by_dataset_id_and_dct_format(&id_urn, &dct_format)
            .await
        {
            Ok(distribution) => (StatusCode::OK, Json(ToCamelCase(distribution))).into_response(),
            Err(e) => return e.into_response(),
        }
    }
    async fn handle_get_distribution_by_id(
        State(state): State<DistributionEntityRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        let id_urn = match extract_path_urn(&id) {
            Ok(urn) => urn,
            Err(resp) => return resp.into_response(),
        };
        match state.service.get_distribution_by_id(&id_urn).await {
            Ok(Some(distribution)) => {
                (StatusCode::OK, Json(ToCamelCase(distribution))).into_response()
            }
            Ok(None) => {
                let err = CommonErrors::missing_resource_new(id.as_str(), "Distribution not found");
                err.into_response()
            }
            Err(e) => return e.into_response(),
        }
    }
    async fn handle_put_distribution_by_id(
        State(state): State<DistributionEntityRouter>,
        Path(id): Path<String>,
        input: Result<Json<EditDistributionDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let id_urn = match extract_path_urn(&id) {
            Ok(urn) => urn,
            Err(resp) => return resp.into_response(),
        };
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.service.put_distribution_by_id(&id_urn, &input).await {
            Ok(distribution) => (StatusCode::OK, Json(ToCamelCase(distribution))).into_response(),
            Err(e) => return e.into_response(),
        }
    }
    async fn handle_create_distribution(
        State(state): State<DistributionEntityRouter>,
        input: Result<Json<NewDistributionDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.service.create_distribution(&input).await {
            Ok(distribution) => (StatusCode::OK, Json(ToCamelCase(distribution))).into_response(),
            Err(e) => return e.into_response(),
        }
    }
    async fn handle_delete_distribution_by_id(
        State(state): State<DistributionEntityRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        let id_urn = match extract_path_urn(&id) {
            Ok(urn) => urn,
            Err(resp) => return resp.into_response(),
        };
        match state.service.delete_distribution_by_id(&id_urn).await {
            Ok(_) => StatusCode::ACCEPTED.into_response(),
            Err(e) => return e.into_response(),
        }
    }
}
