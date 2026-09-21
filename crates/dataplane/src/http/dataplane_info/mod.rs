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

use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRef, Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use common::auth::access::AccessScope;
use common::batch_requests::BatchRequests;
use common::query::{default_limit, Page, Paginated, Sort};
use serde::Deserialize;
use ymir::errors::AppResult;
use ymir::utils::{extract_path_urn, extract_payload};

use crate::entities::dataplane_transfers::{
    DataplaneTransferDto, EditDataplaneTransferDto, InteractionMode, NewDataplaneTransferDto,
    TransferRole, TransferState,
};
use crate::entities::filters::DataplaneTransferFilter;
use crate::http::extractors::ExtractedHeaders;
use crate::services::dataplane_transfers::DataplaneTransferServiceTrait;

#[derive(Debug, Deserialize, Default)]
pub struct DataplaneTransferQuery {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
    pub sort: Option<Sort>,
    pub tenant_id: Option<String>,
    pub transfer_process_id: Option<String>,
    pub role: Option<TransferRole>,
    pub interaction_mode: Option<InteractionMode>,
    pub state: Option<TransferState>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

impl DataplaneTransferQuery {
    pub fn into_domain(self) -> (DataplaneTransferFilter, Page, Sort) {
        let filter = DataplaneTransferFilter {
            tenant_id: self.tenant_id,
            transfer_process_id: self.transfer_process_id,
            role: self.role,
            interaction_mode: self.interaction_mode,
            state: self.state,
            created_after: self.created_after,
            created_before: self.created_before,
        };
        let page = Page::new(self.limit.unwrap_or_else(default_limit), self.cursor);
        let sort = self.sort.unwrap_or_default();
        (filter, page, sort)
    }
}

#[derive(Clone)]
pub struct DataPlaneProcessesRouter {
    service: Arc<dyn DataplaneTransferServiceTrait>,
}

impl FromRef<DataPlaneProcessesRouter> for Arc<dyn DataplaneTransferServiceTrait> {
    fn from_ref(state: &DataPlaneProcessesRouter) -> Self {
        state.service.clone()
    }
}

impl DataPlaneProcessesRouter {
    pub fn new(service: Arc<dyn DataplaneTransferServiceTrait>) -> Self {
        Self { service }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/", get(Self::handle_get_all_dataplane_transfers))
            .route("/", post(Self::handle_create_dataplane_transfer))
            .route("/batch", post(Self::handle_get_batch_dataplane_transfers))
            .route("/{dataplane_id}", get(Self::handle_get_data_plane_by_id))
            .route(
                "/{dataplane_id}",
                put(Self::handle_put_dataplane_transfer_by_id),
            )
            .route(
                "/{dataplane_id}",
                delete(Self::handle_delete_dataplane_transfer),
            )
            .route("/{dataplane_id}/info", get(Self::handle_get_dataplane_info))
            .route(
                "/transfer-process/{transfer_process_id}",
                get(Self::handle_get_dataplane_transfer_by_process_id),
            )
            .with_state(self)
    }

    async fn handle_get_all_dataplane_transfers(
        State(state): State<Self>,
        scope: AccessScope,
        headers: ExtractedHeaders,
        Query(q): Query<DataplaneTransferQuery>,
    ) -> AppResult<(HeaderMap, Json<Paginated<DataplaneTransferDto>>)> {
        let (filter, page, sort) = q.into_domain();
        let result = state.service.get_all(&scope, &filter, &page, &sort).await?;
        let response_headers = headers.response_headers_paged(result.total);
        Ok((response_headers, Json(result)))
    }

    async fn handle_get_data_plane_by_id(
        State(state): State<Self>,
        scope: AccessScope,
        headers: ExtractedHeaders,
        Path(dataplane_id): Path<String>,
    ) -> AppResult<(HeaderMap, Json<DataplaneTransferDto>)> {
        let data_plane_id = extract_path_urn(&dataplane_id)?;
        let transfer = state.service.get_one(&scope, &data_plane_id).await?;
        Ok((headers.response_headers(), Json(transfer)))
    }

    async fn handle_get_batch_dataplane_transfers(
        State(state): State<Self>,
        scope: AccessScope,
        headers: ExtractedHeaders,
        payload: Result<Json<BatchRequests>, JsonRejection>,
    ) -> AppResult<(HeaderMap, Json<Vec<DataplaneTransferDto>>)> {
        let input = extract_payload(payload)?;
        let transfers = state.service.batch(&scope, &input).await?;
        let count = transfers.len() as u64;
        Ok((headers.response_headers_paged(Some(count)), Json(transfers)))
    }

    async fn handle_get_dataplane_transfer_by_process_id(
        State(state): State<Self>,
        scope: AccessScope,
        headers: ExtractedHeaders,
        Path(transfer_process_id): Path<String>,
    ) -> AppResult<(HeaderMap, Json<DataplaneTransferDto>)> {
        let process_urn = extract_path_urn(&transfer_process_id)?;
        let transfer = state
            .service
            .get_by_process_id(&scope, &process_urn)
            .await?;
        Ok((headers.response_headers(), Json(transfer)))
    }

    async fn handle_create_dataplane_transfer(
        State(state): State<Self>,
        scope: AccessScope,
        headers: ExtractedHeaders,
        payload: Result<Json<NewDataplaneTransferDto>, JsonRejection>,
    ) -> AppResult<(StatusCode, HeaderMap, Json<DataplaneTransferDto>)> {
        let new_dataplane_transfer = extract_payload(payload)?;
        let transfer = state
            .service
            .create(&scope, &new_dataplane_transfer)
            .await?;
        Ok((
            StatusCode::CREATED,
            headers.response_headers(),
            Json(transfer),
        ))
    }

    async fn handle_put_dataplane_transfer_by_id(
        State(state): State<Self>,
        scope: AccessScope,
        headers: ExtractedHeaders,
        Path(dataplane_id): Path<String>,
        payload: Result<Json<EditDataplaneTransferDto>, JsonRejection>,
    ) -> AppResult<(HeaderMap, Json<DataplaneTransferDto>)> {
        let data_plane_id = extract_path_urn(&dataplane_id)?;
        let edit_dataplane_transfer = extract_payload(payload)?;
        let transfer = state
            .service
            .edit(&scope, &data_plane_id, &edit_dataplane_transfer)
            .await?;
        Ok((headers.response_headers(), Json(transfer)))
    }

    async fn handle_delete_dataplane_transfer(
        State(state): State<Self>,
        scope: AccessScope,
        headers: ExtractedHeaders,
        Path(dataplane_id): Path<String>,
    ) -> AppResult<(StatusCode, HeaderMap)> {
        let data_plane_id = extract_path_urn(&dataplane_id)?;
        state.service.delete(&scope, &data_plane_id).await?;
        Ok((StatusCode::NO_CONTENT, headers.response_headers()))
    }

    async fn handle_get_dataplane_info(
        State(state): State<Self>,
        scope: AccessScope,
        headers: ExtractedHeaders,
        Path(dataplane_id): Path<String>,
        req_headers: HeaderMap,
    ) -> AppResult<(HeaderMap, Json<DataplaneInfoResponse>)> {
        let data_plane_id = extract_path_urn(&dataplane_id)?;
        let transfer = state.service.get_one(&scope, &data_plane_id).await?;

        let mut ingress_url = None;
        if transfer.inner.interaction_mode == InteractionMode::Pull {
            if let Some(host) = req_headers.get("host").and_then(|h| h.to_str().ok()) {
                ingress_url = Some(format!("{}/dataplane/proxy/{}", host, data_plane_id));
            }
        }

        let response = DataplaneInfoResponse {
            id: transfer.inner.id,
            interaction_mode: transfer.inner.interaction_mode.to_string(),
            ingress_url,
        };
        Ok((headers.response_headers(), Json(response)))
    }
}

#[derive(serde::Serialize)]
pub struct DataplaneInfoResponse {
    pub id: String,
    pub interaction_mode: String,
    pub ingress_url: Option<String>,
}
