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

use crate::entities::commands::NewTransferMessageCommand;
use crate::entities::filters::TransferMessageFilter;
use crate::services::transfer_message::TransferMessageServiceTrait;
use crate::services::transfer_message::views::TransferMessageView;
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRef, Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::get;
use axum::{Json, Router};
use common::auth::access::AccessScope;
use common::auth::http::ExtractedHeaders;
use common::query::{Paginated, QuerySpec, Sort};
use ymir::errors::AppResult;
use ymir::utils::{extract_path_urn, extract_payload};

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
        scope: AccessScope,
        headers: ExtractedHeaders,
        Query(q): Query<QuerySpec<TransferMessageFilter, Sort>>,
    ) -> AppResult<(HeaderMap, Json<Paginated<TransferMessageView>>)> {
        let (filter, page, sort) = q.into_domain();
        let result = state.service.get_all(&scope, &filter, &page, &sort).await?;
        let response_headers = headers.response_headers_paged(result.total);
        Ok((response_headers, Json(result)))
    }

    async fn handle_get_by_process(
        State(state): State<Self>,
        scope: AccessScope,
        headers: ExtractedHeaders,
        Path(process_id): Path<String>,
        Query(q): Query<QuerySpec<TransferMessageFilter, Sort>>,
    ) -> AppResult<(HeaderMap, Json<Paginated<TransferMessageView>>)> {
        let process_urn = extract_path_urn(&process_id)?;
        let (filter, page, sort) = q.into_domain();
        let result = state
            .service
            .get_all_by_process(&scope, &process_urn, &filter, &page, &sort)
            .await?;
        let response_headers = headers.response_headers_paged(result.total);
        Ok((response_headers, Json(result)))
    }

    async fn handle_get_one(
        State(state): State<Self>,
        scope: AccessScope,
        headers: ExtractedHeaders,
        Path(id): Path<String>,
    ) -> AppResult<(HeaderMap, Json<TransferMessageView>)> {
        let urn = extract_path_urn(&id)?;
        let view = state.service.get_one(&scope, &urn).await?;
        Ok((headers.response_headers(), Json(view)))
    }

    async fn handle_create(
        State(state): State<Self>,
        scope: AccessScope,
        headers: ExtractedHeaders,
        payload: Result<Json<NewTransferMessageCommand>, JsonRejection>,
    ) -> AppResult<(StatusCode, HeaderMap, Json<TransferMessageView>)> {
        let payload = extract_payload(payload)?;
        let view = state.service.create(&scope, &payload).await?;
        let response_headers = headers.response_headers();
        Ok((StatusCode::CREATED, response_headers, Json(view)))
    }

    async fn handle_delete(
        State(state): State<Self>,
        scope: AccessScope,
        headers: ExtractedHeaders,
        Path(id): Path<String>,
    ) -> AppResult<(StatusCode, HeaderMap)> {
        let urn = extract_path_urn(&id)?;
        state.service.delete(&scope, &urn).await?;
        Ok((StatusCode::NO_CONTENT, headers.response_headers()))
    }
}
