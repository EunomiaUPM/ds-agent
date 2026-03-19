use crate::entities::connector_instance::{ConnectorInstanceTrait, ConnectorInstantiationDto};
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRef, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use rainbow_common::errors::error_adapter::CustomToResponse;
use rainbow_common::errors::CommonErrors;
use rainbow_common::utils::{parse_urn};
use std::sync::Arc;
use ymir::utils::extract_payload;

#[derive(Clone)]
pub struct ConnectorInstanceRouter {
    service: Arc<dyn ConnectorInstanceTrait>,
}

impl FromRef<ConnectorInstanceRouter> for Arc<dyn ConnectorInstanceTrait> {
    fn from_ref(state: &ConnectorInstanceRouter) -> Self {
        state.service.clone()
    }
}

impl ConnectorInstanceRouter {
    pub fn new(service: Arc<dyn ConnectorInstanceTrait>) -> Self {
        Self { service }
    }
    pub fn router(self) -> Router {
        Router::new()
            .route("/", post(Self::handle_upsert_instance))
            .route("/{id}", get(Self::handle_get_instance_by_id))
            .route("/distribution/{did}", get(Self::get_instance_by_distribution))
            .route("/{id}", delete(Self::handle_delete_instance_by_id))
            .with_state(self)
    }

    async fn handle_upsert_instance(
        State(state): State<ConnectorInstanceRouter>,
        input: Result<Json<ConnectorInstantiationDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let mut input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.service.upsert_instance(&mut input).await {
            Ok(instance) => (StatusCode::OK, Json(instance)).into_response(),
            Err(err) => err.into_response(),
        }
    }
    async fn handle_get_instance_by_id(
        State(state): State<ConnectorInstanceRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        let id = match parse_urn(&id) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };
        match state.service.get_instance_by_id(&id).await {
            Ok(Some(instance)) => (StatusCode::OK, Json(instance)).into_response(),
            Ok(None) => {
                let err = CommonErrors::missing_resource_new("instance", "Instance not found");
                err.into_response()
            }
            Err(err) => err.into_response(),
        }
    }
    async fn get_instance_by_distribution(
        State(state): State<ConnectorInstanceRouter>,
        Path(did): Path<String>,
    ) -> impl IntoResponse {
        let did = match parse_urn(&did) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };
        match state.service.get_instance_by_distribution(&did).await {
            Ok(Some(instance)) => (StatusCode::OK, Json(instance)).into_response(),
            Ok(None) => {
                let err = CommonErrors::missing_resource_new("instance", "Instance not found");
                err.into_response()
            }
            Err(err) => err.into_response(),
        }
    }
    async fn handle_delete_instance_by_id(
        State(state): State<ConnectorInstanceRouter>,
        Path(did): Path<String>,
    ) -> impl IntoResponse {
        let did = match parse_urn(&did) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };
        match state.service.delete_instance_by_id(&did).await {
            Ok(_) => StatusCode::ACCEPTED.into_response(),
            Err(err) => err.into_response(),
        }
    }
}
