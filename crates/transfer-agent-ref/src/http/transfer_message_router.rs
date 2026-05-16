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
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use ymir::errors::AppResult;
use ymir::utils::{extract_path_urn, extract_payload};

use common::auth::claims::Role;
use common::auth::rbac::Rbac;

use crate::entities::commands::NewTransferMessageCommand;
use crate::entities::query::{Page, Paginated, Sort, TransferMessageFilter};
use crate::http::extractors::{AuthClaims, ExtractedHeaders};
use crate::services::transfer_message::TransferMessageServiceTrait;
use crate::services::transfer_message::views::TransferMessageView;

// Router ────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub(crate) struct TransferMessageRouter {
    service: Arc<dyn TransferMessageServiceTrait>,
}

impl FromRef<TransferMessageRouter> for Arc<dyn TransferMessageServiceTrait> {
    fn from_ref(state: &TransferMessageRouter) -> Self {
        state.service.clone()
    }
}

impl TransferMessageRouter {
    pub(crate) fn new(service: Arc<dyn TransferMessageServiceTrait>) -> Self {
        Self { service }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/", get(Self::handle_get_all).post(Self::handle_create))
            .route(
                "/{id}",
                get(Self::handle_get_one).delete(Self::handle_delete),
            )
            .route("/process/{process_id}", get(Self::handle_get_by_process))
            .with_state(self)
    }

    async fn handle_get_all(
        State(state): State<Self>,
        auth: AuthClaims,
        headers: ExtractedHeaders,
        Query(q): Query<TransferMessageQuery>,
    ) -> AppResult<(HeaderMap, Json<Paginated<TransferMessageView>>)> {
        Rbac::require_read(&auth, headers.tenant_id.as_str())?;
        let tenant_filter = (auth.role != Role::Admin).then(|| headers.tenant_id.clone());
        let (filter, page, sort) = q.into_domain(tenant_filter);
        let result = state.service.get_all(&filter, &page, &sort).await?;
        let response_headers = headers.response_headers_paged(result.total);
        Ok((response_headers, Json(result)))
    }

    async fn handle_get_by_process(
        State(state): State<Self>,
        auth: AuthClaims,
        headers: ExtractedHeaders,
        Path(process_id): Path<String>,
        Query(q): Query<TransferMessageQuery>,
    ) -> AppResult<(HeaderMap, Json<Paginated<TransferMessageView>>)> {
        Rbac::require_read(&auth, headers.tenant_id.as_str())?;
        let process_urn = extract_path_urn(&process_id)?;
        let tenant_filter = (auth.role != Role::Admin).then(|| headers.tenant_id.clone());
        let (filter, page, sort) = q.into_domain(tenant_filter);
        let result = state
            .service
            .get_all_by_process(&process_urn, &filter, &page, &sort)
            .await?;
        let response_headers = headers.response_headers_paged(result.total);
        Ok((response_headers, Json(result)))
    }

    async fn handle_get_one(
        State(state): State<Self>,
        auth: AuthClaims,
        headers: ExtractedHeaders,
        Path(id): Path<String>,
    ) -> AppResult<(HeaderMap, Json<TransferMessageView>)> {
        Rbac::require_read(&auth, headers.tenant_id.as_str())?;
        let urn = extract_path_urn(&id)?;
        let view = state.service.get_one(&urn).await?;
        Ok((headers.response_headers(), Json(view)))
    }

    async fn handle_create(
        State(state): State<Self>,
        auth: AuthClaims,
        headers: ExtractedHeaders,
        payload: Result<Json<NewTransferMessageCommand>, JsonRejection>,
    ) -> AppResult<(StatusCode, HeaderMap, Json<TransferMessageView>)> {
        Rbac::require_write(&auth, headers.tenant_id.as_str())?;
        let mut payload = extract_payload(payload)?;
        if payload.tenant_id.is_none() {
            payload.tenant_id = Some(headers.tenant_id.clone());
        }
        let view = state.service.create(&payload).await?;
        Ok((StatusCode::CREATED, headers.response_headers(), Json(view)))
    }

    async fn handle_delete(
        State(state): State<Self>,
        auth: AuthClaims,
        headers: ExtractedHeaders,
        Path(id): Path<String>,
    ) -> AppResult<(StatusCode, HeaderMap)> {
        Rbac::require_write(&auth, headers.tenant_id.as_str())?;
        let urn = extract_path_urn(&id)?;
        state.service.delete(&urn).await?;
        Ok((StatusCode::NO_CONTENT, headers.response_headers()))
    }
}

// Query params ──────────────────────────────────────────────────────────────

/// Tenant is intentionally absent from the query string — it comes from X-Tenant-ID header.
#[derive(Deserialize)]
pub struct TransferMessageQuery {
    #[serde(flatten)]
    filter: TransferMessageFilter,
    #[serde(default = "default_limit")]
    limit: u32,
    cursor: Option<String>,
    #[serde(default)]
    sort: Sort,
}

fn default_limit() -> u32 {
    20
}

impl TransferMessageQuery {
    fn into_domain(
        self,
        force_tenant_id: Option<crate::entities::ids::TenantId>,
    ) -> (TransferMessageFilter, Page, Sort) {
        let mut filter = self.filter;
        if let Some(tid) = force_tenant_id {
            filter.tenant_id = Some(tid);
        }
        let page = Page { limit: self.limit, cursor: self.cursor };
        (filter, page, self.sort)
    }
}
