use crate::entities::dataplane_transfers::{
    DataplaneTransferDto, DataplaneTransfersEntitiesTrait, EditDataplaneTransferDto,
    InteractionMode, NewDataplaneTransferDto, TransferState,
};
use crate::entities::transfer_events::TransferEventEntitiesTrait;
use crate::errors::error_adapter::CustomToResponse;
use crate::http::common::parse_urn;
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRef, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, patch, post, put};
use axum::{Json, Router};
use rainbow_common::batch_requests::BatchRequests;
use rainbow_common::errors::{CommonErrors, ErrorLog};
use rainbow_common::utils::extract_payload;
use serde_json::Value;
use std::sync::Arc;

#[derive(Clone)]
pub struct DataPlaneProcessesRouter {
    data_plane_process_entity: Arc<dyn DataplaneTransfersEntitiesTrait>,
    transfer_event_entity: Arc<dyn TransferEventEntitiesTrait>,
}

impl FromRef<DataPlaneProcessesRouter> for Arc<dyn DataplaneTransfersEntitiesTrait> {
    fn from_ref(state: &DataPlaneProcessesRouter) -> Self {
        state.data_plane_process_entity.clone()
    }
}

impl FromRef<DataPlaneProcessesRouter> for Arc<dyn TransferEventEntitiesTrait> {
    fn from_ref(state: &DataPlaneProcessesRouter) -> Self {
        state.transfer_event_entity.clone()
    }
}

impl DataPlaneProcessesRouter {
    pub fn new(
        data_plane_process_entity: Arc<dyn DataplaneTransfersEntitiesTrait>,
        transfer_event_entity: Arc<dyn TransferEventEntitiesTrait>,
    ) -> Self {
        Self { data_plane_process_entity, transfer_event_entity }
    }
    pub fn router(self) -> Router {
        Router::new()
            .route("/", get(Self::handle_get_all_dataplane_transfers))
            .route("/", post(Self::handle_create_dataplane_transfer))
            .route("/batch", post(Self::handle_get_batch_dataplane_transfers))
            .route("/{dataplane_id}", get(Self::handle_get_data_plane_by_id))
            .route("/{dataplane_id}", put(Self::handle_put_dataplane_transfer_by_id))
            .route("/{dataplane_id}", delete(Self::handle_delete_dataplane_transfer))
            .route("/{dataplane_id}/info", get(Self::handle_get_dataplane_info))
            .route("/transfer-process/{transfer_process_id}", get(Self::handle_get_dataplane_transfer_by_process_id))
            .with_state(self)
    }

    async fn handle_get_all_dataplane_transfers(
        State(state): State<DataPlaneProcessesRouter>,
    ) -> impl IntoResponse {
        match state.data_plane_process_entity.get_all_dataplane_transfers().await {
            Ok(transfers) => (StatusCode::OK, Json(transfers)).into_response(),
            Err(e) => e.to_response(),
        }
    }

    async fn handle_get_data_plane_by_id(
        State(state): State<DataPlaneProcessesRouter>,
        Path(dataplane_id): Path<String>,
    ) -> impl IntoResponse {
        let data_plane_id = match parse_urn(&dataplane_id) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };
        match state.data_plane_process_entity.get_dataplane_transfer_by_id(&data_plane_id).await {
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

    async fn handle_get_batch_dataplane_transfers(
        State(state): State<DataPlaneProcessesRouter>,
        input: Result<Json<BatchRequests>, JsonRejection>,
    ) -> impl IntoResponse {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e,
        };
        match state.data_plane_process_entity.get_batch_dataplane_transfers(&input.ids).await {
            Ok(transfers) => (StatusCode::OK, Json(transfers)).into_response(),
            Err(e) => e.to_response(),
        }
    }

    async fn handle_get_dataplane_transfer_by_process_id(
        State(state): State<DataPlaneProcessesRouter>,
        Path(transfer_process_id): Path<String>,
    ) -> impl IntoResponse {
        let process_urn = match parse_urn(&transfer_process_id) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };
        match state.data_plane_process_entity.get_dataplane_transfer_by_process_id(&process_urn).await {
            Ok(dataplane_session) => match dataplane_session {
                Some(dataplane_session) => {
                    (StatusCode::OK, Json(dataplane_session)).into_response()
                }
                None => {
                    let err = CommonErrors::missing_resource_new(
                        process_urn.to_string().as_str(),
                        "Transfer process not found",
                    );
                    tracing::error!("{}", err.log());
                    err.into_response()
                }
            },
            Err(e) => e.to_response(),
        }
    }

    async fn handle_create_dataplane_transfer(
        State(state): State<DataPlaneProcessesRouter>,
        input: Result<Json<NewDataplaneTransferDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let new_dataplane_transfer = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e,
        };
        match state.data_plane_process_entity.create_dataplane_transfer(&new_dataplane_transfer).await {
            Ok(transfer) => (StatusCode::CREATED, Json(transfer)).into_response(),
            Err(e) => e.to_response(),
        }
    }

    async fn handle_put_dataplane_transfer_by_id(
        State(state): State<DataPlaneProcessesRouter>,
        Path(dataplane_id): Path<String>,
        input: Result<Json<EditDataplaneTransferDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let data_plane_id = match parse_urn(&dataplane_id) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e,
        };
        match state.data_plane_process_entity.put_dataplane_transfer_by_id(&data_plane_id, &input).await {
            Ok(transfer) => (StatusCode::OK, Json(transfer)).into_response(),
            Err(e) => e.to_response(),
        }
    }

    async fn handle_delete_dataplane_transfer(
        State(state): State<DataPlaneProcessesRouter>,
        Path(dataplane_id): Path<String>,
    ) -> impl IntoResponse {
        let data_plane_id = match parse_urn(&dataplane_id) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };
        match state.data_plane_process_entity.delete_dataplane_transfer(&data_plane_id).await {
            Ok(_) => StatusCode::NO_CONTENT.into_response(),
            Err(e) => e.to_response(),
        }
    }

    async fn handle_get_dataplane_info(
        State(state): State<DataPlaneProcessesRouter>,
        Path(dataplane_id): Path<String>,
        headers: axum::http::HeaderMap,
    ) -> impl IntoResponse {
        let data_plane_id = match parse_urn(&dataplane_id) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };
        
        match state.data_plane_process_entity.get_dataplane_transfer_by_id(&data_plane_id).await {
            Ok(Some(transfer)) => {
                let mut ingress_url = None;
                if transfer.inner.interaction_mode == crate::entities::dataplane_transfers::InteractionMode::Pull {
                    if let Some(host) = headers.get("host").and_then(|h| h.to_str().ok()) {
                        // Assuming HTTP for now, or X-Forwarded-Proto if behind proxy
                        ingress_url = Some(format!("http://{}/dataplane/proxy/{}", host, data_plane_id));
                    }
                }
                
                let response = DataplaneInfoResponse {
                    id: transfer.inner.id,
                    interaction_mode: transfer.inner.interaction_mode.to_string(),
                    ingress_url,
                };
                (StatusCode::OK, Json(response)).into_response()
            },
             Ok(None) => {
                 CommonErrors::missing_resource_new(
                     data_plane_id.to_string().as_str(), 
                     "Dataplane transfer not found"
                 ).into_response()
             },
            Err(e) => e.to_response(),
        }
    }
}

#[derive(serde::Serialize)]
pub struct DataplaneInfoResponse {
    pub id: String,
    pub interaction_mode: String,
    pub ingress_url: Option<String>,
}
