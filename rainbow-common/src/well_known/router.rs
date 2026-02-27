use std::sync::Arc;

use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRef, Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use ymir::errors::AppResult;
use ymir::utils::extract_payload;

use crate::dsp_common::well_known_types::{Version, VersionPath, VersionResponse};
use crate::well_known::dspace_version::dspace_version::WellKnownDSpaceVersionService;
use crate::well_known::dspace_version::WellKnownDSpaceVersionTrait;
use crate::well_known::rpc::{WellKnownRPCRequest, WellKnownRPCTrait};

#[derive(Clone)]
pub struct WellKnownRouter {
    pub dspace_version_service: WellKnownDSpaceVersionService,
    pub dspace_version_rpc: Arc<dyn WellKnownRPCTrait>
}

impl FromRef<WellKnownRouter> for Arc<dyn WellKnownRPCTrait> {
    fn from_ref(state: &WellKnownRouter) -> Self { state.dspace_version_rpc.clone() }
}

impl FromRef<WellKnownRouter> for WellKnownDSpaceVersionService {
    fn from_ref(state: &WellKnownRouter) -> Self { state.dspace_version_service.clone() }
}

impl WellKnownRouter {
    pub fn new(
        dspace_version_service: WellKnownDSpaceVersionService,
        dspace_version_rpc: Arc<dyn WellKnownRPCTrait>
    ) -> WellKnownRouter {
        WellKnownRouter { dspace_version_service, dspace_version_rpc }
    }
    pub fn router(self) -> Router {
        Router::new()
            .route(
                "/.well-known/dspace-version",
                get(Self::handle_get_well_known_version)
            )
            .route(
                "/.well-known/dspace-version/{version}",
                get(Self::handle_get_well_known_version_version)
            )
            .route(
                "/rpc/.well-known/dspace-version",
                post(Self::handle_post_well_known_version_from_participant)
            )
            .route(
                "/rpc/.well-known/dspace-version/path",
                post(Self::handle_post_well_known_version_from_participant_path)
            )
            .with_state(self)
    }

    async fn handle_get_well_known_version(
        State(state): State<WellKnownRouter>
    ) -> AppResult<Json<VersionResponse>> {
        Ok(Json(state.dspace_version_service.get_dspace_version()?))
    }
    async fn handle_get_well_known_version_version(
        State(state): State<WellKnownRouter>,
        Path(version): Path<String>
    ) -> AppResult<Json<Version>> {
        Ok(Json(state.dspace_version_service.get_dspace_version_str(&version)?))
    }
    async fn handle_post_well_known_version_from_participant(
        State(state): State<WellKnownRouter>,
        input: Result<Json<WellKnownRPCRequest>, JsonRejection>
    ) -> AppResult<Json<VersionResponse>> {
        let input = extract_payload(input)?;
        let (version, _) = state.dspace_version_rpc.fetch_dataspace_well_known(&input).await?;
        Ok(Json(version))
    }

    async fn handle_post_well_known_version_from_participant_path(
        State(state): State<WellKnownRouter>,
        input: Result<Json<WellKnownRPCRequest>, JsonRejection>
    ) -> AppResult<Json<VersionPath>> {
        let input = extract_payload(input)?;
        Ok(Json(
            state.dspace_version_rpc.fetch_dataspace_current_path(&input).await?
        ))
    }
}
