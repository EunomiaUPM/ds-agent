use crate::entities::dataplane_transfer_logs::DataplaneTransferLogsEntitiesTrait;
use axum::extract::{FromRef, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use rainbow_common::errors::{CommonErrors, ErrorLog};
use std::sync::Arc;
use tracing::error;
use crate::http::common::parse_urn;

#[derive(Clone)]
pub struct DataplaneTransferLogsRouter {
    logs_entity: Arc<dyn DataplaneTransferLogsEntitiesTrait>,
}

impl FromRef<DataplaneTransferLogsRouter> for Arc<dyn DataplaneTransferLogsEntitiesTrait> {
    fn from_ref(state: &DataplaneTransferLogsRouter) -> Self {
        state.logs_entity.clone()
    }
}

impl DataplaneTransferLogsRouter {
    pub fn new(logs_entity: Arc<dyn DataplaneTransferLogsEntitiesTrait>) -> Self {
        Self { logs_entity }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/{dataplane_process_id}/logs", get(Self::handle_get_logs_by_dataplane_process_id))
            .with_state(self)
    }

    async fn handle_get_logs_by_dataplane_process_id(
        State(state): State<DataplaneTransferLogsRouter>,
        Path(dataplane_process_id): Path<String>,
    ) -> impl IntoResponse {
        let dataplane_process_id = match parse_urn(&dataplane_process_id) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };

        match state.logs_entity.get_transfer_logs_by_dataplane_process_id(&dataplane_process_id).await {
            Ok(logs) => (StatusCode::OK, Json(logs)).into_response(),
            Err(e) => {
                let err = CommonErrors::database_new(&e.to_string());
                error!("{}", err.log());
                err.into_response()
            }
        }
    }
}
