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
use crate::setup::context::AppContext;
use crate::services::transfer_message::views::TransferMessageView;
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRef, Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::get;
use axum::{Json, Router};
use ymir::types::oauth::UserInfo;
use ymir::http_tracing::ExtractedHeaders;
use ymir::query::{Paginated, QuerySpec, Sort};
use ymir::errors::AppResult;
use ymir::utils::{extract_path_urn, extract_payload};
use common::routes::transfer as routes;

#[derive(Clone)]
pub struct TransferMessageRouter {
    ctx: Arc<AppContext>,
}

impl FromRef<TransferMessageRouter> for Arc<dyn TransferMessageServiceTrait> {
    fn from_ref(state: &TransferMessageRouter) -> Self {
        state.ctx.messages().clone()
    }
}

impl TransferMessageRouter {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route(routes::entity::ROOT, get(Self::handle_get_all).post(Self::handle_create))
            .route(
                routes::entity::BY_ID,
                get(Self::handle_get_one).delete(Self::handle_delete),
            )
            .route(routes::entity::BY_PROCESS, get(Self::handle_get_by_process))
            .with_state(self)
    }

    async fn handle_get_all(
        State(state): State<Self>,
        user: UserInfo,
        headers: ExtractedHeaders,
        Query(q): Query<QuerySpec<TransferMessageFilter, Sort>>,
    ) -> AppResult<(HeaderMap, Json<Paginated<TransferMessageView>>)> {
        let (filter, page, sort) = q.into_domain();
        let result = state
            .ctx
            .messages()
            .get_all(&user, &filter, &page, &sort)
            .await?;
        let response_headers = headers.response_headers_paged(result.total);
        Ok((response_headers, Json(result)))
    }

    async fn handle_get_by_process(
        State(state): State<Self>,
        user: UserInfo,
        headers: ExtractedHeaders,
        Path(process_id): Path<String>,
        Query(q): Query<QuerySpec<TransferMessageFilter, Sort>>,
    ) -> AppResult<(HeaderMap, Json<Paginated<TransferMessageView>>)> {
        let process_urn = extract_path_urn(&process_id)?;
        let (filter, page, sort) = q.into_domain();
        let result = state
            .ctx
            .messages()
            .get_all_by_process(&user, &process_urn, &filter, &page, &sort)
            .await?;
        let response_headers = headers.response_headers_paged(result.total);
        Ok((response_headers, Json(result)))
    }

    async fn handle_get_one(
        State(state): State<Self>,
        user: UserInfo,
        headers: ExtractedHeaders,
        Path(id): Path<String>,
    ) -> AppResult<(HeaderMap, Json<TransferMessageView>)> {
        let urn = extract_path_urn(&id)?;
        let view = state.ctx.messages().get_one(&user, &urn).await?;
        Ok((headers.response_headers(), Json(view)))
    }

    async fn handle_create(
        State(state): State<Self>,
        user: UserInfo,
        headers: ExtractedHeaders,
        payload: Result<Json<NewTransferMessageCommand>, JsonRejection>,
    ) -> AppResult<(StatusCode, HeaderMap, Json<TransferMessageView>)> {
        let payload = extract_payload(payload)?;
        let view = state.ctx.messages().create(&user, &payload).await?;
        let response_headers = headers.response_headers();
        Ok((StatusCode::CREATED, response_headers, Json(view)))
    }

    async fn handle_delete(
        State(state): State<Self>,
        user: UserInfo,
        headers: ExtractedHeaders,
        Path(id): Path<String>,
    ) -> AppResult<(StatusCode, HeaderMap)> {
        let urn = extract_path_urn(&id)?;
        state.ctx.messages().delete(&user, &urn).await?;
        Ok((StatusCode::NO_CONTENT, headers.response_headers()))
    }
}
