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
pub struct TransferEventsRouter {
    transfer_event_entity: Arc<dyn TransferEventEntitiesTrait>,
}

impl FromRef<TransferEventsRouter> for Arc<dyn TransferEventEntitiesTrait> {
    fn from_ref(state: &TransferEventsRouter) -> Self {
        state.transfer_event_entity.clone()
    }
}

impl TransferEventsRouter {
    pub fn new(transfer_event_entity: Arc<dyn TransferEventEntitiesTrait>) -> Self {
        Self { transfer_event_entity }
    }

    pub fn transfers_sub_router(self) -> Router {
        Router::new()
            .route("/{transfer_id}/events", get(Self::handle_get_events_by_transfer_id))
            .with_state(self)
    }

    pub fn events_sub_router(self) -> Router {
        Router::new().route("/{event_id}", get(Self::handle_get_event_by_id)).with_state(self)
    }

    async fn handle_get_events_by_transfer_id(
        State(state): State<TransferEventsRouter>,
        Path(transfer_id): Path<String>,
    ) -> impl IntoResponse {
        let transfer_id = match parse_urn(&transfer_id) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };

        match state.transfer_event_entity.get_transfer_events_by_process_id(&transfer_id).await {
            Ok(events) => (StatusCode::OK, Json(events)).into_response(),
            Err(e) => e.to_response(),
        }
    }

    async fn handle_get_event_by_id(
        State(state): State<TransferEventsRouter>,
        Path(event_id): Path<String>,
    ) -> impl IntoResponse {
        let event_id = match parse_urn(&event_id) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };
        match state.transfer_event_entity.get_transfer_event_by_id(&event_id).await {
            Ok(transfer_event) => match transfer_event {
                Some(transfer_event) => (StatusCode::OK, Json(transfer_event)).into_response(),
                None => {
                    let err = CommonErrors::missing_resource_new(
                        event_id.to_string().as_str(),
                        "Transfer event not found",
                    );
                    tracing::error!("{}", err.log());
                    err.into_response()
                }
            },
            Err(e) => e.to_response(),
        }
    }
}
