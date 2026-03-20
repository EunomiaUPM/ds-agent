use crate::entities::data_services::{
    DataServiceEntityTrait, EditDataServiceDto, NewDataServiceDto,
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
use serde::Deserialize;
use std::sync::Arc;
use ymir::errors::Errors;
use ymir::utils::{extract_path_urn, extract_payload};

#[derive(Clone)]
pub struct DataServiceEntityRouter {
    service: Arc<dyn DataServiceEntityTrait>,
    config: Arc<CatalogConfig>,
}

#[derive(Deserialize)]
pub struct PaginationParams {
    pub limit: Option<u64>,
    pub page: Option<u64>,
}

impl FromRef<DataServiceEntityRouter> for Arc<dyn DataServiceEntityTrait> {
    fn from_ref(state: &DataServiceEntityRouter) -> Self {
        state.service.clone()
    }
}

impl FromRef<DataServiceEntityRouter> for Arc<CatalogConfig> {
    fn from_ref(state: &DataServiceEntityRouter) -> Self {
        state.config.clone()
    }
}

impl DataServiceEntityRouter {
    pub fn new(service: Arc<dyn DataServiceEntityTrait>, config: Arc<CatalogConfig>) -> Self {
        Self { service, config }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/", get(Self::handle_get_all_data_services))
            .route("/catalog/{id}", get(Self::handle_get_data_services_by_catalog_id))
            .route("/", post(Self::handle_create_data_service))
            .route("/batch", post(Self::handle_get_batch_data_services))
            .route("/main", get(Self::handle_get_main_data_service))
            .route("/main", post(Self::handle_create_main_data_service))
            .route("/{id}", get(Self::handle_get_data_service_by_id))
            .route("/{id}", put(Self::handle_put_data_service_by_id))
            .route("/{id}", delete(Self::handle_delete_data_service_by_id))
            .with_state(self)
    }

    async fn handle_get_all_data_services(
        State(state): State<DataServiceEntityRouter>,
        Query(params): Query<PaginationParams>,
    ) -> impl IntoResponse {
        match state.service.get_all_data_services(params.limit, params.page).await {
            Ok(data_services) => (StatusCode::OK, Json(ToCamelCase(data_services))).into_response(),
            Err(e) => return e.into_response(),        }
    }
    async fn handle_get_batch_data_services(
        State(state): State<DataServiceEntityRouter>,
        input: Result<Json<BatchRequests>, JsonRejection>,
    ) -> impl IntoResponse {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.service.get_batch_data_services(&input.ids).await {
            Ok(data_services) => (StatusCode::OK, Json(ToCamelCase(data_services))).into_response(),
            Err(e) => return e.into_response(),        }
    }
    async fn handle_get_data_services_by_catalog_id(
        State(state): State<DataServiceEntityRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        let id_urn = match extract_path_urn(&id) {
            Ok(urn) => urn,
            Err(resp) => return resp.into_response(),
        };
        match state.service.get_data_services_by_catalog_id(&id_urn).await {
            Ok(data_services) => (StatusCode::OK, Json(ToCamelCase(data_services))).into_response(),
            Err(e) => return e.into_response(),
        }
    }
    async fn handle_get_data_service_by_id(
        State(state): State<DataServiceEntityRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        let id_urn = match extract_path_urn(&id) {
            Ok(urn) => urn,
            Err(resp) => return resp.into_response(),
        };
        match state.service.get_data_service_by_id(&id_urn).await {
            Ok(Some(data_service)) => {
                (StatusCode::OK, Json(ToCamelCase(data_service))).into_response()
            }
            Ok(None) => {
                let err = Errors::missing_resource(id.as_str(), "Data service not found", None);
                err.into_response()
            }
            Err(e) => return e.into_response(),        }
    }

    async fn handle_get_main_data_service(
        State(state): State<DataServiceEntityRouter>,
    ) -> impl IntoResponse {
        match state.service.get_main_data_service().await {
            Ok(Some(data_service)) => {
                (StatusCode::OK, Json(ToCamelCase(data_service))).into_response()
            }
            Ok(None) => {
                let err = Errors::missing_resource("", "Main Data service not found", None);
                err.into_response()
            }
            Err(e) => return e.into_response(),        }
    }

    async fn handle_put_data_service_by_id(
        State(state): State<DataServiceEntityRouter>,
        Path(id): Path<String>,
        input: Result<Json<EditDataServiceDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let id_urn = match extract_path_urn(&id) {
            Ok(urn) => urn,
            Err(resp) => return resp.into_response(),
        };
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.service.put_data_service_by_id(&id_urn, &input).await {
            Ok(data_service) => (StatusCode::OK, Json(ToCamelCase(data_service))).into_response(),
            Err(e) => return e.into_response(),
        }
    }
    async fn handle_create_data_service(
        State(state): State<DataServiceEntityRouter>,
        input: Result<Json<NewDataServiceDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.service.create_data_service(&input).await {
            Ok(data_service) => (StatusCode::OK, Json(ToCamelCase(data_service))).into_response(),
            Err(e) => return e.into_response(),
        }
    }

    async fn handle_create_main_data_service(
        State(state): State<DataServiceEntityRouter>,
        input: Result<Json<NewDataServiceDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.service.create_main_data_service(&input).await {
            Ok(data_service) => (StatusCode::OK, Json(ToCamelCase(data_service))).into_response(),
            Err(e) => return e.into_response(),
        }
    }

    async fn handle_delete_data_service_by_id(
        State(state): State<DataServiceEntityRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        let id_urn = match extract_path_urn(&id) {
            Ok(urn) => urn,
            Err(resp) => return resp.into_response(),
        };
        match state.service.delete_data_service_by_id(&id_urn).await {
            Ok(_) => StatusCode::ACCEPTED.into_response(),
            Err(e) => return e.into_response(),
        }
    }
}
