/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use crate::entities::transfer_events::TransferEventEntitiesTrait;
use crate::http::common::parse_urn;
use axum::extract::{FromRef, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use common::errors::{CommonErrors, ErrorLog};
use std::sync::Arc;
use ymir::errors::Errors;

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

    pub fn dataplane_processes_sub_router(self) -> Router {
        Router::new()
            .route(
                "/{dataplane_process_id}/events",
                get(Self::handle_get_events_by_transfer_id),
            )
            .with_state(self)
    }

    pub fn events_sub_router(self) -> Router {
        Router::new().route("/{event_id}", get(Self::handle_get_event_by_id)).with_state(self)
    }

    async fn handle_get_events_by_transfer_id(
        State(state): State<TransferEventsRouter>,
        Path(dataplane_process_id): Path<String>,
    ) -> impl IntoResponse {
        let dataplane_process_id = match parse_urn(&dataplane_process_id) {
            Ok(urn) => urn,
            Err(resp) => return resp,
        };

        match state
            .transfer_event_entity
            .get_transfer_events_by_process_id(&dataplane_process_id)
            .await
        {
            Ok(events) => (StatusCode::OK, Json(events)).into_response(),
            Err(e) => e.into_response(),
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
                    let err = Errors::missing_resource(
                        event_id.to_string().as_str(),
                        "Transfer event not found",
                        None,
                    );
                    err.into_response()
                }
            },
            Err(e) => e.into_response(),
        }
    }
}
