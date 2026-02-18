use crate::errors::error_adapter::CustomToResponse;
use crate::utils::extract_payload;
use crate::well_known::dspace_version::dspace_version::WellKnownDSpaceVersionService;
use crate::well_known::dspace_version::WellKnownDSpaceVersionTrait;
use crate::well_known::rpc::{WellKnownRPCRequest, WellKnownRPCTrait};
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRef, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use std::sync::Arc;

#[derive(Clone)]
pub struct WellKnownRouter {
    pub dspace_version_service: WellKnownDSpaceVersionService,
    pub dspace_version_rpc: Arc<dyn WellKnownRPCTrait>,
}

impl FromRef<WellKnownRouter> for Arc<dyn WellKnownRPCTrait> {
    fn from_ref(state: &WellKnownRouter) -> Self {
        state.dspace_version_rpc.clone()
    }
}

impl FromRef<WellKnownRouter> for WellKnownDSpaceVersionService {
    fn from_ref(state: &WellKnownRouter) -> Self {
        state.dspace_version_service.clone()
    }
}

impl WellKnownRouter {
    pub fn new(
        dspace_version_service: WellKnownDSpaceVersionService,
        dspace_version_rpc: Arc<dyn WellKnownRPCTrait>,
    ) -> WellKnownRouter {
        WellKnownRouter { dspace_version_service, dspace_version_rpc }
    }
    pub fn router(self) -> Router {
        Router::new()
            .route(
                "/.well-known/dspace-version",
                get(Self::handle_get_well_known_version),
            )
            .route(
                "/.well-known/dspace-version/{version}",
                get(Self::handle_get_well_known_version_version),
            )
            .route(
                "/rpc/.well-known/dspace-version",
                post(Self::handle_post_well_known_version_from_participant),
            )
            .route(
                "/rpc/.well-known/dspace-version/path",
                post(Self::handle_post_well_known_version_from_participant_path),
            )
            .with_state(self)
    }

    async fn handle_get_well_known_version(
        State(state): State<WellKnownRouter>,
    ) -> impl IntoResponse {
        let response = state.dspace_version_service.get_dspace_version().unwrap();
        (StatusCode::OK, Json(response)).into_response()
    }
    async fn handle_get_well_known_version_version(
        State(state): State<WellKnownRouter>,
        Path(version): Path<String>,
    ) -> impl IntoResponse {
        let response = state.dspace_version_service.get_dspace_version_str(&version).unwrap();
        (StatusCode::OK, Json(response)).into_response()
    }
    async fn handle_post_well_known_version_from_participant(
        State(state): State<WellKnownRouter>,
        input: Result<Json<WellKnownRPCRequest>, JsonRejection>,
    ) -> impl IntoResponse {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e,
        };
        match state.dspace_version_rpc.fetch_dataspace_well_known(&input).await {
            Ok(res) => (StatusCode::OK, Json(res.0)).into_response(),
            Err(err) => err.to_response(),
        }
    }
    async fn handle_post_well_known_version_from_participant_path(
        State(state): State<WellKnownRouter>,
        input: Result<Json<WellKnownRPCRequest>, JsonRejection>,
    ) -> impl IntoResponse {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e,
        };
        match state.dspace_version_rpc.fetch_dataspace_current_path(&input).await {
            Ok(res) => (StatusCode::OK, Json(res)).into_response(),
            Err(err) => err.to_response(),
        }
    }
}
