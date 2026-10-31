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

//! HTTP router for negotiation process management endpoints.

use crate::entities::filters::NegotiationProcessFilter;
use crate::entities::negotiation_process::{EditNegotiationProcessDto, NewNegotiationProcessDto};
use crate::setup::context::AppContext;
use crate::services::negotiation_process::NegotiationProcessServiceTrait;
use crate::services::negotiation_process::views::NegotiationProcessView;
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRef, Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use ymir::types::oauth::UserInfo;
use ymir::http_tracing::ExtractedHeaders;
use ymir::batch_requests::BatchRequests;
use ymir::query::{Paginated, QuerySpec, Sort};
use std::sync::Arc;
use ymir::errors::AppResult;
use ymir::utils::{extract_path_urn, extract_payload};
use common::routes::negotiation as routes;

pub type NegotiationProcessQuery = QuerySpec<NegotiationProcessFilter, Sort>;

#[derive(Clone)]
pub struct NegotiationAgentProcessesRouter {
    ctx: Arc<AppContext>,
}

impl FromRef<NegotiationAgentProcessesRouter> for Arc<dyn NegotiationProcessServiceTrait> {
    fn from_ref(state: &NegotiationAgentProcessesRouter) -> Self {
        state.ctx.processes().clone()
    }
}

impl NegotiationAgentProcessesRouter {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route(routes::entity::ROOT, get(Self::handle_get_all).post(Self::handle_create))
            .route(routes::entity::BATCH, post(Self::handle_batch))
            .route(
                routes::entity::BY_ID,
                get(Self::handle_get_one)
                    .put(Self::handle_edit)
                    .delete(Self::handle_delete),
            )
            .route(routes::negotiation_processes::KEY, get(Self::handle_get_by_key_id))
            .with_state(self)
    }

    async fn handle_get_all(
        State(state): State<Self>,
        user: UserInfo,
        headers: ExtractedHeaders,
        Query(q): Query<NegotiationProcessQuery>,
    ) -> AppResult<(HeaderMap, Json<Paginated<NegotiationProcessView>>)> {
        let (filter, page, sort) = q.into_domain();
        let result = state.ctx.processes().get_all(&user, &filter, &page, &sort).await?;
        let response_headers = headers.response_headers_paged(result.total);
        Ok((response_headers, Json(result)))
    }

    async fn handle_batch(
        State(state): State<Self>,
        user: UserInfo,
        headers: ExtractedHeaders,
        payload: Result<Json<BatchRequests>, JsonRejection>,
    ) -> AppResult<(HeaderMap, Json<Vec<NegotiationProcessView>>)> {
        let payload = extract_payload(payload)?;
        let views = state.ctx.processes().batch(&user, &payload).await?;
        let count = views.len() as u64;
        Ok((headers.response_headers_paged(Some(count)), Json(views)))
    }

    async fn handle_get_one(
        State(state): State<Self>,
        user: UserInfo,
        headers: ExtractedHeaders,
        Path(id): Path<String>,
    ) -> AppResult<(HeaderMap, Json<NegotiationProcessView>)> {
        let urn = extract_path_urn(&id)?;
        let view = state.ctx.processes().get_one(&user, &urn).await?;
        Ok((headers.response_headers(), Json(view)))
    }

    async fn handle_create(
        State(state): State<Self>,
        user: UserInfo,
        headers: ExtractedHeaders,
        payload: Result<Json<NewNegotiationProcessDto>, JsonRejection>,
    ) -> AppResult<(StatusCode, HeaderMap, Json<NegotiationProcessView>)> {
        let payload = extract_payload(payload)?;
        let view = state.ctx.processes().create(&user, &payload).await?;
        let response_headers = headers.response_headers();
        Ok((StatusCode::CREATED, response_headers, Json(view)))
    }

    async fn handle_edit(
        State(state): State<Self>,
        user: UserInfo,
        headers: ExtractedHeaders,
        Path(id): Path<String>,
        payload: Result<Json<EditNegotiationProcessDto>, JsonRejection>,
    ) -> AppResult<(HeaderMap, Json<NegotiationProcessView>)> {
        let urn = extract_path_urn(&id)?;
        let payload = extract_payload(payload)?;
        let view = state.ctx.processes().edit(&user, &urn, &payload).await?;
        Ok((headers.response_headers(), Json(view)))
    }

    async fn handle_delete(
        State(state): State<Self>,
        user: UserInfo,
        headers: ExtractedHeaders,
        Path(id): Path<String>,
    ) -> AppResult<(StatusCode, HeaderMap)> {
        let urn = extract_path_urn(&id)?;
        state.ctx.processes().delete(&user, &urn).await?;
        Ok((StatusCode::NO_CONTENT, headers.response_headers()))
    }

    async fn handle_get_by_key_id(
        State(state): State<Self>,
        user: UserInfo,
        headers: ExtractedHeaders,
        Path((id, key_id)): Path<(String, String)>,
    ) -> AppResult<(HeaderMap, Json<NegotiationProcessView>)> {
        let urn = extract_path_urn(&id)?;
        let view = state.ctx.processes().get_one(&user, &urn).await?;
        if view.correlation.identifiers.contains_key(&key_id) {
            Ok((headers.response_headers(), Json(view)))
        } else {
            Err(ymir::errors::Errors::missing_resource(
                format!("{urn}/key/{key_id}"),
                "Key not found for negotiation process",
                None,
            ))
        }
    }
}
