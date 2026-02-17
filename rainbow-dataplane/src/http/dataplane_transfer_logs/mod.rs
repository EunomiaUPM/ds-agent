use crate::entities::dataplane_transfer_logs::DataplaneTransferLogsEntitiesTrait;
use axum::extract::{FromRef, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use rainbow_common::errors::{CommonErrors, ErrorLog};
use std::sync::Arc;
use tracing::error;

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
            .route("/{transfer_id}/logs", get(Self::handle_get_logs_by_transfer_id))
            .with_state(self)
    }

    async fn handle_get_logs_by_transfer_id(
        State(logs_entity): State<Arc<dyn DataplaneTransferLogsEntitiesTrait>>,
        Path(transfer_id): Path<String>,
    ) -> impl IntoResponse {
        // Try parsing as simple UUID first since logs usually use UUID in this codebase
        // But URN handling is present in other handlers.
        // Let's stick to Uuid since `get_transfer_logs_by_transfer_id` takes Uuid.
        let uuid = match uuid::Uuid::parse_str(&transfer_id) {
            Ok(uuid) => uuid,
            Err(_) => {
                // If not UUID, maybe it is a URN or just invalid.
                // For now, let's assume UUID or return error to keep it consistent with other ID usages if they were UUIDs.
                let err = CommonErrors::missing_resource_new(
                    &transfer_id,
                    "Invalid UUID for transfer ID",
                );
                return err.into_response();
            }
        };

        match logs_entity.get_transfer_logs_by_transfer_id(uuid).await {
            Ok(logs) => (StatusCode::OK, Json(logs)).into_response(),
            Err(e) => {
                // Map anyhow error to response
                // Assuming CustomToResponse is implemented for anyhow::Error or we construct CommonError
                // Here we can use standard internal server error
                let err = CommonErrors::database_new(&e.to_string());
                error!("{}", err.log());
                err.into_response()
            }
        }
    }
}
