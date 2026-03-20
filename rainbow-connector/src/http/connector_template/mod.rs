#![allow(unused)]
use crate::entities::connector_template::{ConnectorTemplateDto, ConnectorTemplateEntitiesTrait};
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRef, Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use rainbow_common::config::services::CatalogConfig;
use rainbow_common::errors::CommonErrors;
use serde::Deserialize;
use std::sync::Arc;
use ymir::utils::extract_payload;

#[derive(Clone)]
pub struct ConnectorTemplateRouter {
    service: Arc<dyn ConnectorTemplateEntitiesTrait>,
    config: Arc<CatalogConfig>,
}

#[derive(Deserialize)]
pub struct PaginationParams {
    pub limit: Option<u64>,
    pub page: Option<u64>,
}

impl FromRef<ConnectorTemplateRouter> for Arc<dyn ConnectorTemplateEntitiesTrait> {
    fn from_ref(state: &ConnectorTemplateRouter) -> Self {
        state.service.clone()
    }
}

impl ConnectorTemplateRouter {
    pub fn new(
        service: Arc<dyn ConnectorTemplateEntitiesTrait>,
        config: Arc<CatalogConfig>,
    ) -> Self {
        Self { service, config }
    }
    pub fn router(self) -> Router {
        Router::new()
            .route("/", get(Self::handle_get_all_templates))
            .route("/", post(Self::handle_create_template))
            .route("/{id}", get(Self::handle_get_templates_by_id))
            .route(
                "/{name}/{version}",
                get(Self::handle_get_template_by_name_and_version),
            )
            .route(
                "/{name}/{version}",
                delete(Self::handle_delete_template_by_name_and_version),
            )
            .with_state(self)
    }

    async fn handle_get_all_templates(
        State(state): State<ConnectorTemplateRouter>,
        Query(params): Query<PaginationParams>,
    ) -> impl IntoResponse {
        match state.service.get_all_templates(params.limit, params.page).await {
            Ok(templates) => (StatusCode::OK, Json(templates)).into_response(),
            Err(err) => err.into_response(),
        }
    }
    async fn handle_create_template(
        State(state): State<ConnectorTemplateRouter>,
        input: Result<Json<ConnectorTemplateDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let mut input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.service.create_template(&mut input).await {
            Ok(template) => (StatusCode::OK, Json(template)).into_response(),
            Err(err) => err.into_response(),
        }
    }
    async fn handle_get_templates_by_id(
        State(state): State<ConnectorTemplateRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        match state.service.get_templates_by_id(&id).await {
            Ok(templates) => (StatusCode::OK, Json(templates)).into_response(),
            Err(err) => err.into_response(),
        }
    }
    async fn handle_get_template_by_name_and_version(
        State(state): State<ConnectorTemplateRouter>,
        Path((name, version)): Path<(String, String)>,
    ) -> impl IntoResponse {
        match state.service.get_template_by_name_and_version(&name, &version).await {
            Ok(Some(template)) => (StatusCode::OK, Json(template)).into_response(),
            Ok(None) => {
                let err = CommonErrors::missing_resource_new("main", "Main Catalog not found");
                err.into_response()
            }
            Err(err) => err.into_response(),
        }
    }
    async fn handle_delete_template_by_name_and_version(
        State(state): State<ConnectorTemplateRouter>,
        Path((name, version)): Path<(String, String)>,
    ) -> impl IntoResponse {
        match state.service.delete_template_by_name_and_version(&name, &version).await {
            Ok(_) => StatusCode::ACCEPTED.into_response(),
            Err(err) => err.into_response(),
        }
    }
}
