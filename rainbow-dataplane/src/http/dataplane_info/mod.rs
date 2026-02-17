use crate::entities::dataplane_transfers::DataplaneTransfersEntitiesTrait;
use crate::entities::transfer_events::TransferEventEntitiesTrait;
use crate::errors::error_adapter::CustomToResponse;
use crate::http::common::parse_urn;
use axum::extract::{FromRef, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use rainbow_common::errors::{CommonErrors, ErrorLog};
use std::sync::Arc;

#[derive(Clone)]
pub struct DataPlaneRouter {
    data_plane_process_entity: Arc<dyn DataplaneTransfersEntitiesTrait>,
    transfer_event_entity: Arc<dyn TransferEventEntitiesTrait>,
}

impl FromRef<DataPlaneRouter> for Arc<dyn DataplaneTransfersEntitiesTrait> {
    fn from_ref(state: &DataPlaneRouter) -> Self {
        state.data_plane_process_entity.clone()
    }
}

impl FromRef<DataPlaneRouter> for Arc<dyn TransferEventEntitiesTrait> {
    fn from_ref(state: &DataPlaneRouter) -> Self {
        state.transfer_event_entity.clone()
    }
}

impl DataPlaneRouter {
    pub fn new(
        data_plane_process_entity: Arc<dyn DataplaneTransfersEntitiesTrait>,
        transfer_event_entity: Arc<dyn TransferEventEntitiesTrait>,
    ) -> Self {
        Self { data_plane_process_entity, transfer_event_entity }
    }
    pub fn router(self) -> Router {
        Router::new()
            .route("/{transfer_id}", get(Self::handle_get_data_plane_by_id))
            .with_state(self)
    }
    async fn handle_get_data_plane_by_id(
        State(dataplane_process_entity): State<Arc<dyn DataplaneTransfersEntitiesTrait>>,
        Path(transfer_id): Path<String>,
    ) -> impl IntoResponse {
        let data_plane_id = match parse_urn(&transfer_id) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };
        let uuid = match uuid::Uuid::parse_str(data_plane_id.nss()) {
            Ok(uuid) => uuid,
            Err(_) => {
                let err = CommonErrors::missing_resource_new(
                    data_plane_id.to_string().as_str(),
                    "Invalid UUID in URN",
                );
                return err.into_response();
            }
        };

        match dataplane_process_entity.get_dataplane_transfer_by_id(uuid).await {
            Ok(dataplane_session) => match dataplane_session {
                Some(dataplane_session) => {
                    (StatusCode::OK, Json(dataplane_session)).into_response()
                }
                None => {
                    let err = CommonErrors::missing_resource_new(
                        data_plane_id.to_string().as_str(),
                        "Data plane process not found",
                    );
                    tracing::error!("{}", err.log());
                    err.into_response()
                }
            },
            Err(e) => e.to_response(),
        }
    }
}
