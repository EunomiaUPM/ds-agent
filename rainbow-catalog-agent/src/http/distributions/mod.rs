use crate::entities::distributions::{
    DistributionEntityTrait, EditDistributionDto, NewDistributionDto,
};
use crate::errors::error_adapter::CustomToResponse;
use crate::http::common::to_camel_case::ToCamelCase;
use crate::http::common::{extract_payload, parse_urn};
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRef, Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use rainbow_common::batch_requests::BatchRequests;
use rainbow_common::config::services::CatalogConfig;
use rainbow_common::dcat_formats::DctFormats;
use rainbow_common::errors::CommonErrors;
use serde::Deserialize;
use std::str::FromStr;
use std::sync::Arc;

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
            Err(err) => err.to_response(),
        }
    }
    async fn handle_get_batch_distributions(
        State(state): State<DistributionEntityRouter>,
        input: Result<Json<BatchRequests>, JsonRejection>,
    ) -> impl IntoResponse {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e,
        };
        match state.service.get_batch_distributions(&input.ids).await {
            Ok(distributions) => (StatusCode::OK, Json(ToCamelCase(distributions))).into_response(),
            Err(err) => err.to_response(),
        }
    }
    async fn handle_get_distributions_by_dataset_id(
        State(state): State<DistributionEntityRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        let id_urn = match parse_urn(&id) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };
        match state.service.get_distributions_by_dataset_id(&id_urn).await {
            Ok(distributions) => (StatusCode::OK, Json(ToCamelCase(distributions))).into_response(),
            Err(err) => match err.downcast::<CommonErrors>() {
                Ok(ce) => match ce {
                    CommonErrors::DatabaseError { ref cause, .. } => {
                        if cause.contains("not found") {
                            let err = CommonErrors::missing_resource_new("", cause.as_str());
                            return err.into_response();
                        } else {
                            ce.into_response()
                        }
                    }
                    e => return e.into_response(),
                },
                Err(e) => e.to_response(),
            },
        }
    }
    async fn handle_get_distribution_by_dataset_id_and_dct_format(
        State(state): State<DistributionEntityRouter>,
        Path((id, dct_format)): Path<(String, String)>,
    ) -> impl IntoResponse {
        let id_urn = match parse_urn(&id) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };
        match state
            .service
            .get_distribution_by_dataset_id_and_dct_format(&id_urn, &dct_format)
            .await
        {
            Ok(distribution) => (StatusCode::OK, Json(ToCamelCase(distribution))).into_response(),
            Err(err) => match err.downcast::<CommonErrors>() {
                Ok(ce) => match ce {
                    CommonErrors::DatabaseError { ref cause, .. } => {
                        if cause.contains("not found") {
                            let err = CommonErrors::missing_resource_new("", cause.as_str());
                            return err.into_response();
                        } else {
                            ce.into_response()
                        }
                    }
                    e => return e.into_response(),
                },
                Err(e) => e.to_response(),
            },
        }
    }
    async fn handle_get_distribution_by_id(
        State(state): State<DistributionEntityRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        let id_urn = match parse_urn(&id) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };
        match state.service.get_distribution_by_id(&id_urn).await {
            Ok(Some(distribution)) => {
                (StatusCode::OK, Json(ToCamelCase(distribution))).into_response()
            }
            Ok(None) => {
                let err = CommonErrors::missing_resource_new(id.as_str(), "Distribution not found");
                err.into_response()
            }
            Err(err) => err.to_response(),
        }
    }
    async fn handle_put_distribution_by_id(
        State(state): State<DistributionEntityRouter>,
        Path(id): Path<String>,
        input: Result<Json<EditDistributionDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let id_urn = match parse_urn(&id) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e,
        };
        match state.service.put_distribution_by_id(&id_urn, &input).await {
            Ok(distribution) => (StatusCode::OK, Json(ToCamelCase(distribution))).into_response(),
            Err(err) => match err.downcast::<CommonErrors>() {
                Ok(ce) => match ce {
                    CommonErrors::DatabaseError { ref cause, .. } => {
                        if cause.contains("not found") {
                            let err = CommonErrors::missing_resource_new("", cause.as_str());
                            return err.into_response();
                        } else {
                            ce.into_response()
                        }
                    }
                    e => return e.into_response(),
                },
                Err(e) => e.to_response(),
            },
        }
    }
    async fn handle_create_distribution(
        State(state): State<DistributionEntityRouter>,
        input: Result<Json<NewDistributionDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e,
        };
        match state.service.create_distribution(&input).await {
            Ok(distribution) => (StatusCode::OK, Json(ToCamelCase(distribution))).into_response(),
            Err(err) => match err.downcast::<CommonErrors>() {
                Ok(ce) => match ce {
                    CommonErrors::DatabaseError { ref cause, .. } => {
                        if cause.contains("not found") {
                            let err = CommonErrors::missing_resource_new("", cause.as_str());
                            return err.into_response();
                        } else {
                            ce.into_response()
                        }
                    }
                    e => return e.into_response(),
                },
                Err(e) => e.to_response(),
            },
        }
    }
    async fn handle_delete_distribution_by_id(
        State(state): State<DistributionEntityRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        let id_urn = match parse_urn(&id) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };
        match state.service.delete_distribution_by_id(&id_urn).await {
            Ok(_) => StatusCode::ACCEPTED.into_response(),
            Err(err) => match err.downcast::<CommonErrors>() {
                Ok(ce) => match ce {
                    CommonErrors::DatabaseError { ref cause, .. } => {
                        if cause.contains("not found") {
                            let err = CommonErrors::missing_resource_new("", cause.as_str());
                            return err.into_response();
                        } else {
                            ce.into_response()
                        }
                    }
                    e => return e.into_response(),
                },
                Err(e) => e.to_response(),
            },
        }
    }
}
