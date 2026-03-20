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

use crate::entities::datasets::DatasetEntityTrait;
use crate::entities::datasets::{EditDatasetDto, NewDatasetDto};
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
use std::sync::Arc;
use ymir::utils::{extract_path_urn, extract_payload};

#[derive(Clone)]
pub struct DatasetEntityRouter {
    service: Arc<dyn DatasetEntityTrait>,
    config: Arc<CatalogConfig>,
}

#[derive(Deserialize)]
pub struct PaginationParams {
    pub limit: Option<u64>,
    pub page: Option<u64>,
}

impl FromRef<DatasetEntityRouter> for Arc<dyn DatasetEntityTrait> {
    fn from_ref(state: &DatasetEntityRouter) -> Self {
        state.service.clone()
    }
}

impl FromRef<DatasetEntityRouter> for Arc<CatalogConfig> {
    fn from_ref(state: &DatasetEntityRouter) -> Self {
        state.config.clone()
    }
}

impl DatasetEntityRouter {
    pub fn new(service: Arc<dyn DatasetEntityTrait>, config: Arc<CatalogConfig>) -> Self {
        Self { service, config }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/", get(Self::handle_get_all_datasets))
            .route("/catalog/{id}", get(Self::handle_get_datasets_by_catalog_id))
            .route("/", post(Self::handle_create_dataset))
            .route("/batch", post(Self::handle_get_batch_datasets))
            .route("/{id}", get(Self::handle_get_dataset_by_id))
            .route("/{id}", put(Self::handle_put_dataset_by_id))
            .route("/{id}", delete(Self::handle_delete_dataset_by_id))
            .with_state(self)
    }

    async fn handle_get_all_datasets(
        State(state): State<DatasetEntityRouter>,
        Query(params): Query<PaginationParams>,
    ) -> impl IntoResponse {
        match state.service.get_all_datasets(params.limit, params.page).await {
            Ok(datasets) => (StatusCode::OK, Json(ToCamelCase(datasets))).into_response(),
            Err(e) => return e.into_response(),
        }
    }
    async fn handle_get_batch_datasets(
        State(state): State<DatasetEntityRouter>,
        input: Result<Json<BatchRequests>, JsonRejection>,
    ) -> impl IntoResponse {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.service.get_batch_datasets(&input.ids).await {
            Ok(datasets) => (StatusCode::OK, Json(ToCamelCase(datasets))).into_response(),
            Err(e) => return e.into_response(),
        }
    }
    async fn handle_get_datasets_by_catalog_id(
        State(state): State<DatasetEntityRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        let id_urn = match extract_path_urn(&id) {
            Ok(urn) => urn,
            Err(resp) => return resp.into_response(),
        };
        match state.service.get_datasets_by_catalog_id(&id_urn).await {
            Ok(dataset) => (StatusCode::OK, Json(ToCamelCase(dataset))).into_response(),
            Err(e) => return e.into_response(),
        }
    }
    async fn handle_get_dataset_by_id(
        State(state): State<DatasetEntityRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        let id_urn = match extract_path_urn(&id) {
            Ok(urn) => urn,
            Err(resp) => return resp.into_response(),
        };
        match state.service.get_dataset_by_id(&id_urn).await {
            Ok(Some(dataset)) => (StatusCode::OK, Json(ToCamelCase(dataset))).into_response(),
            Ok(None) => {
                let err = CommonErrors::missing_resource_new(id.as_str(), "Dataset not found");
                err.into_response()
            }
            Err(e) => return e.into_response(),
        }
    }
    async fn handle_put_dataset_by_id(
        State(state): State<DatasetEntityRouter>,
        Path(id): Path<String>,
        input: Result<Json<EditDatasetDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let id_urn = match extract_path_urn(&id) {
            Ok(urn) => urn,
            Err(resp) => return resp.into_response(),
        };
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.service.put_dataset_by_id(&id_urn, &input).await {
            Ok(dataset) => (StatusCode::OK, Json(ToCamelCase(dataset))).into_response(),
            Err(e) => return e.into_response(),
        }
    }
    async fn handle_create_dataset(
        State(state): State<DatasetEntityRouter>,
        input: Result<Json<NewDatasetDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.service.create_dataset(&input).await {
            Ok(dataset) => (StatusCode::OK, Json(ToCamelCase(dataset))).into_response(),
            Err(e) => return e.into_response(),
        }
    }
    async fn handle_delete_dataset_by_id(
        State(state): State<DatasetEntityRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        let id_urn = match extract_path_urn(&id) {
            Ok(urn) => urn,
            Err(resp) => return resp.into_response(),
        };
        match state.service.delete_dataset_by_id(&id_urn).await {
            Ok(_) => StatusCode::ACCEPTED.into_response(),
            Err(e) => return e.into_response(),
        }
    }
}
