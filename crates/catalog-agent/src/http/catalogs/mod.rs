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

use crate::entities::catalogs::{CatalogEntityTrait, EditCatalogDto, NewCatalogDto};
use crate::http::common::to_camel_case::ToCamelCase;
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRef, Path, Query, State};
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use common::batch_requests::BatchRequests;
use common::config::services::CatalogConfig;
use common::errors::CommonErrors;
use reqwest::StatusCode;
use serde::Deserialize;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::Errors;
use ymir::utils::{extract_path_urn, extract_payload};

#[derive(Clone)]
pub struct CatalogEntityRouter {
    service: Arc<dyn CatalogEntityTrait>,
    config: Arc<CatalogConfig>,
}

#[derive(Deserialize)]
pub struct PaginationParams {
    pub limit: Option<u64>,
    pub page: Option<u64>,
    pub with_main_catalog: Option<bool>,
}

impl FromRef<CatalogEntityRouter> for Arc<dyn CatalogEntityTrait> {
    fn from_ref(state: &CatalogEntityRouter) -> Self {
        state.service.clone()
    }
}

impl FromRef<CatalogEntityRouter> for Arc<CatalogConfig> {
    fn from_ref(state: &CatalogEntityRouter) -> Self {
        state.config.clone()
    }
}

impl CatalogEntityRouter {
    pub fn new(service: Arc<dyn CatalogEntityTrait>, config: Arc<CatalogConfig>) -> Self {
        Self { service, config }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/", get(Self::handle_get_all_catalogs))
            .route("/", post(Self::handle_create_catalog))
            .route("/main", get(Self::handle_get_main_catalog))
            .route("/main", post(Self::handle_create_main_catalog))
            .route("/batch", post(Self::handle_get_batch_catalogs))
            .route("/{id}", get(Self::handle_get_catalog_by_id))
            .route("/{id}", put(Self::handle_put_catalog_by_id))
            .route("/{id}", delete(Self::handle_delete_catalog_by_id))
            .with_state(self)
    }

    async fn handle_get_all_catalogs(
        State(state): State<CatalogEntityRouter>,
        Query(params): Query<PaginationParams>,
    ) -> impl IntoResponse {
        let with_main_catalog = params.with_main_catalog.unwrap_or(true);
        match state
            .service
            .get_all_catalogs(params.limit, params.page, with_main_catalog)
            .await
        {
            Ok(catalogs) => (StatusCode::OK, Json(ToCamelCase(catalogs))).into_response(),
            Err(err) => err.into_response(),
        }
    }
    async fn handle_get_batch_catalogs(
        State(state): State<CatalogEntityRouter>,
        input: Result<Json<BatchRequests>, JsonRejection>,
    ) -> impl IntoResponse {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.service.get_batch_catalogs(&input.ids).await {
            Ok(catalogs) => (StatusCode::OK, Json(ToCamelCase(catalogs))).into_response(),
            Err(err) => err.into_response(),
        }
    }
    async fn handle_get_catalog_by_id(
        State(state): State<CatalogEntityRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        let id_urn = match extract_path_urn(&id) {
            Ok(urn) => urn,
            Err(err) => return err.into_response(),
        };
        match state.service.get_catalog_by_id(&id_urn).await {
            Ok(Some(catalog)) => (StatusCode::OK, Json(ToCamelCase(catalog))).into_response(),
            Ok(None) => {
                let err = CommonErrors::missing_resource_new(id.as_str(), "Catalog not found");
                err.into_response()
            }
            Err(err) => err.into_response(),
        }
    }
    async fn handle_get_main_catalog(
        State(state): State<CatalogEntityRouter>,
    ) -> impl IntoResponse {
        match state.service.get_main_catalog().await {
            Ok(Some(catalog)) => (StatusCode::OK, Json(ToCamelCase(catalog))).into_response(),
            Ok(None) => {
                let err = CommonErrors::missing_resource_new("main", "Main Catalog not found");
                err.into_response()
            }
            Err(err) => err.into_response(),
        }
    }
    async fn handle_put_catalog_by_id(
        State(state): State<CatalogEntityRouter>,
        Path(id): Path<String>,
        input: Result<Json<EditCatalogDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let id_urn = match extract_path_urn(&id) {
            Ok(urn) => urn,
            Err(err) => return err.into_response(),
        };
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.service.put_catalog_by_id(&id_urn, &input).await {
            Ok(catalog) => (StatusCode::ACCEPTED, Json(ToCamelCase(catalog))).into_response(),
            Err(err) => err.into_response(),
        }
    }
    async fn handle_create_catalog(
        State(state): State<CatalogEntityRouter>,
        input: Result<Json<NewCatalogDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.service.create_catalog(&input).await {
            Ok(catalog) => (StatusCode::CREATED, Json(ToCamelCase(catalog))).into_response(),
            Err(err) => err.into_response(),
        }
    }
    async fn handle_create_main_catalog(
        State(state): State<CatalogEntityRouter>,
        input: Result<Json<NewCatalogDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.service.create_main_catalog(&input).await {
            Ok(catalog) => (StatusCode::CREATED, Json(ToCamelCase(catalog))).into_response(),
            Err(err) => err.into_response(),
        }
    }
    async fn handle_delete_catalog_by_id(
        State(state): State<CatalogEntityRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        let id_urn = match extract_path_urn(&id) {
            Ok(urn) => urn,
            Err(err) => return err.into_response(),
        };
        match state.service.delete_catalog_by_id(&id_urn).await {
            Ok(_) => StatusCode::ACCEPTED.into_response(),
            Err(err) => err.into_response(),
        }
    }
}
