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

use crate::entities::dataplane_transfer_logs::DataplaneTransferLogDto;
use crate::services::dataplane_transfer_logs::DataplaneTransferLogServiceTrait;
use axum::extract::{FromRef, Path, State};
use axum::http::HeaderMap;
use axum::routing::get;
use axum::{Json, Router};
use common::auth::access::AccessScope;
use common::auth::http::ExtractedHeaders;
use ymir::errors::AppResult;
use ymir::utils::extract_path_urn;

#[derive(Clone)]
pub struct DataplaneTransferLogsRouter {
    service: Arc<dyn DataplaneTransferLogServiceTrait>,
}

impl FromRef<DataplaneTransferLogsRouter> for Arc<dyn DataplaneTransferLogServiceTrait> {
    fn from_ref(state: &DataplaneTransferLogsRouter) -> Self {
        state.service.clone()
    }
}

impl DataplaneTransferLogsRouter {
    pub fn new(service: Arc<dyn DataplaneTransferLogServiceTrait>) -> Self {
        Self { service }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route(
                "/{dataplane_process_id}/logs",
                get(Self::handle_get_logs_by_dataplane_process_id),
            )
            .with_state(self)
    }

    async fn handle_get_logs_by_dataplane_process_id(
        State(state): State<Self>,
        scope: AccessScope,
        headers: ExtractedHeaders,
        Path(dataplane_process_id): Path<String>,
    ) -> AppResult<(HeaderMap, Json<Vec<DataplaneTransferLogDto>>)> {
        let process_urn = extract_path_urn(&dataplane_process_id)?;
        let logs = state
            .service
            .get_transfer_logs_by_dataplane_process_id(&scope, &process_urn)
            .await?;

        Ok((headers.response_headers(), Json(logs)))
    }
}
