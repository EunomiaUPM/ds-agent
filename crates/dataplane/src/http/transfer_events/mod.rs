/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
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

use std::sync::Arc;

use crate::entities::transfer_events::TransferEventDto;
use crate::services::transfer_events::TransferEventServiceTrait;
use axum::extract::{FromRef, Path, State};
use axum::http::HeaderMap;
use axum::routing::get;
use axum::{Json, Router};
use common::auth::access::AccessScope;
use common::auth::http::ExtractedHeaders;
use ymir::errors::AppResult;
use ymir::utils::extract_path_urn;

#[derive(Clone)]
pub struct TransferEventsRouter {
    service: Arc<dyn TransferEventServiceTrait>,
}

impl FromRef<TransferEventsRouter> for Arc<dyn TransferEventServiceTrait> {
    fn from_ref(state: &TransferEventsRouter) -> Self {
        state.service.clone()
    }
}

impl TransferEventsRouter {
    pub fn new(service: Arc<dyn TransferEventServiceTrait>) -> Self {
        Self { service }
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
        Router::new()
            .route("/{event_id}", get(Self::handle_get_event_by_id))
            .with_state(self)
    }

    async fn handle_get_events_by_transfer_id(
        State(state): State<Self>,
        scope: AccessScope,
        headers: ExtractedHeaders,
        Path(dataplane_process_id): Path<String>,
    ) -> AppResult<(HeaderMap, Json<Vec<TransferEventDto>>)> {
        let process_urn = extract_path_urn(&dataplane_process_id)?;
        let events = state
            .service
            .get_by_process_id(&scope, &process_urn)
            .await?;

        Ok((headers.response_headers(), Json(events)))
    }

    async fn handle_get_event_by_id(
        State(state): State<Self>,
        scope: AccessScope,
        headers: ExtractedHeaders,
        Path(event_id): Path<String>,
    ) -> AppResult<(HeaderMap, Json<TransferEventDto>)> {
        let event_urn = extract_path_urn(&event_id)?;
        let event = state.service.get_one(&scope, &event_urn).await?;

        Ok((headers.response_headers(), Json(event)))
    }
}
