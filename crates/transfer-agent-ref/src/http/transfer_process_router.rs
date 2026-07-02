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
use axum::extract::{FromRef, OriginalUri, Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use common::batch_requests::BatchRequests;
use serde::Deserialize;
use ymir::errors::AppResult;
use ymir::utils::{extract_path_urn, extract_payload};

use crate::entities::commands::{EditTransferProcessCommand, NewTransferProcessCommand};
use crate::entities::query::{Page, Paginated, Sort, TransferProcessFilter};
use crate::http::extractors::{AuthClaims, ExtractedHeaders};
use crate::services::access::AccessScope;
use crate::services::transfer_process::TransferProcessServiceTrait;
use crate::services::transfer_process::views::TransferProcessView;

// Router ────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub(crate) struct TransferProcessRouter {
    service: Arc<dyn TransferProcessServiceTrait>,
}

impl FromRef<TransferProcessRouter> for Arc<dyn TransferProcessServiceTrait> {
    fn from_ref(state: &TransferProcessRouter) -> Self {
        state.service.clone()
    }
}

impl TransferProcessRouter {
    pub(crate) fn new(service: Arc<dyn TransferProcessServiceTrait>) -> Self {
        Self { service }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/", get(Self::handle_get_all).post(Self::handle_create))
            .route("/batch", post(Self::handle_batch))
            .route(
                "/{id}",
                get(Self::handle_get_one)
                    .put(Self::handle_edit)
                    .delete(Self::handle_delete),
            )
            .with_state(self)
    }

    async fn handle_get_all(
        State(state): State<Self>,
        auth: AuthClaims,
        headers: ExtractedHeaders,
        Query(q): Query<TransferProcessQuery>,
    ) -> AppResult<(HeaderMap, Json<Paginated<TransferProcessView>>)> {
        let scope = AccessScope::for_read(&auth, &headers.tenant_id)?;
        let (filter, page, sort) = q.into_domain();
        let result = state.service.get_all(&scope, &filter, &page, &sort).await?;
        let response_headers = headers.response_headers_paged(result.total);
        Ok((response_headers, Json(result)))
    }

    async fn handle_batch(
        State(state): State<Self>,
        auth: AuthClaims,
        headers: ExtractedHeaders,
        payload: Result<Json<BatchRequests>, JsonRejection>,
    ) -> AppResult<(HeaderMap, Json<Vec<TransferProcessView>>)> {
        let scope = AccessScope::for_read(&auth, &headers.tenant_id)?;
        let payload = extract_payload(payload)?;
        let views = state.service.batch(&scope, &payload).await?;
        let count = views.len() as u64;
        Ok((headers.response_headers_paged(Some(count)), Json(views)))
    }

    async fn handle_get_one(
        State(state): State<Self>,
        auth: AuthClaims,
        headers: ExtractedHeaders,
        Path(id): Path<String>,
    ) -> AppResult<(HeaderMap, Json<TransferProcessView>)> {
        let scope = AccessScope::for_read(&auth, &headers.tenant_id)?;
        let urn = extract_path_urn(&id)?;
        let view = state.service.get_one(&scope, &urn).await?;
        Ok((headers.response_headers(), Json(view)))
    }

    async fn handle_create(
        State(state): State<Self>,
        auth: AuthClaims,
        headers: ExtractedHeaders,
        OriginalUri(uri): OriginalUri,
        payload: Result<Json<NewTransferProcessCommand>, JsonRejection>,
    ) -> AppResult<(StatusCode, HeaderMap, Json<TransferProcessView>)> {
        let scope = AccessScope::for_write(&auth, &headers.tenant_id)?;
        let payload = extract_payload(payload)?;
        let view = state.service.create(&scope, &payload).await?;
        let mut response_headers = headers.response_headers();
        insert_location(&mut response_headers, uri.path(), &view.id.to_string());
        Ok((StatusCode::CREATED, response_headers, Json(view)))
    }

    async fn handle_edit(
        State(state): State<Self>,
        auth: AuthClaims,
        headers: ExtractedHeaders,
        Path(id): Path<String>,
        payload: Result<Json<EditTransferProcessCommand>, JsonRejection>,
    ) -> AppResult<(HeaderMap, Json<TransferProcessView>)> {
        let scope = AccessScope::for_write(&auth, &headers.tenant_id)?;
        let urn = extract_path_urn(&id)?;
        let payload = extract_payload(payload)?;
        let view = state.service.edit(&scope, &urn, &payload).await?;
        Ok((headers.response_headers(), Json(view)))
    }

    async fn handle_delete(
        State(state): State<Self>,
        auth: AuthClaims,
        headers: ExtractedHeaders,
        Path(id): Path<String>,
    ) -> AppResult<(StatusCode, HeaderMap)> {
        let scope = AccessScope::for_write(&auth, &headers.tenant_id)?;
        let urn = extract_path_urn(&id)?;
        state.service.delete(&scope, &urn).await?;
        Ok((StatusCode::NO_CONTENT, headers.response_headers()))
    }
}

/// Sets `Location: <collection-path>/<id>` on a 201 response.
fn insert_location(headers: &mut HeaderMap, collection_path: &str, id: &str) {
    let path = format!("{}/{}", collection_path.trim_end_matches('/'), id);
    if let Ok(value) = axum::http::HeaderValue::from_str(&path) {
        headers.insert(axum::http::header::LOCATION, value);
    }
}

// Query params ──────────────────────────────────────────────────────────────

/// Tenant is intentionally absent from the query string — the service injects it
/// from the caller's [`AccessScope`]. `limit` and `cursor` are NOT flattened from
/// `Page` to avoid serde_urlencoded's string→u32 coercion failure when using
/// `#[serde(flatten)]`. Date-range validation and limit clamping also live in the
/// service so both transports share them.
#[derive(Deserialize)]
pub struct TransferProcessQuery {
    #[serde(flatten)]
    filter: TransferProcessFilter,
    #[serde(default = "default_limit")]
    limit: u32,
    cursor: Option<String>,
    #[serde(default)]
    sort: Sort,
}

fn default_limit() -> u32 {
    crate::entities::query::DEFAULT_PAGE_LIMIT
}

impl TransferProcessQuery {
    fn into_domain(self) -> (TransferProcessFilter, Page, Sort) {
        let page = Page {
            limit: self.limit,
            cursor: self.cursor,
        };
        (self.filter, page, self.sort)
    }
}
