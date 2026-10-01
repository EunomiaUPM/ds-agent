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

//! HTTP router for agreement management endpoints.

use crate::entities::agreement::{EditAgreementDto, NewAgreementDto};
use crate::entities::filters::AgreementFilter;
use crate::services::agreement::AgreementServiceTrait;
use crate::services::agreement::views::AgreementView;
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRef, Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use common::auth::access::AccessScope;
use common::auth::http::ExtractedHeaders;
use common::batch_requests::BatchRequests;
use common::query::{Paginated, QuerySpec, Sort};
use std::sync::Arc;
use ymir::errors::AppResult;
use ymir::utils::{extract_path_urn, extract_payload};

pub type AgreementQuery = QuerySpec<AgreementFilter, Sort>;

#[derive(Clone)]
pub struct NegotiationAgentAgreementsRouter {
    service: Arc<dyn AgreementServiceTrait>,
}

impl FromRef<NegotiationAgentAgreementsRouter> for Arc<dyn AgreementServiceTrait> {
    fn from_ref(state: &NegotiationAgentAgreementsRouter) -> Self {
        state.service.clone()
    }
}

impl NegotiationAgentAgreementsRouter {
    pub fn new(service: Arc<dyn AgreementServiceTrait>) -> Self {
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
            .route("/process/{process_id}", get(Self::handle_get_by_process))
            .route("/assignee/{assignee}", get(Self::handle_get_by_assignee))
            .route("/assigner/{assigner}", get(Self::handle_get_by_assigner))
            .with_state(self)
    }

    async fn handle_get_all(
        State(state): State<Self>,
        scope: AccessScope,
        headers: ExtractedHeaders,
        Query(q): Query<AgreementQuery>,
    ) -> AppResult<(HeaderMap, Json<Paginated<AgreementView>>)> {
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
        Query(q): Query<AgreementQuery>,
    ) -> AppResult<(HeaderMap, Json<Paginated<AgreementView>>)> {
        let process_urn = extract_path_urn(&process_id)?;
        let (mut filter, page, sort) = q.into_domain();
        filter.process_id = Some(process_urn.to_string());
        let result = state.service.get_all(&scope, &filter, &page, &sort).await?;
        let response_headers = headers.response_headers_paged(result.total);
        Ok((response_headers, Json(result)))
    }

    async fn handle_get_by_assignee(
        State(state): State<Self>,
        scope: AccessScope,
        headers: ExtractedHeaders,
        Path(assignee): Path<String>,
        Query(q): Query<AgreementQuery>,
    ) -> AppResult<(HeaderMap, Json<Paginated<AgreementView>>)> {
        let (mut filter, page, sort) = q.into_domain();
        filter.consumer_id = Some(assignee);
        let result = state.service.get_all(&scope, &filter, &page, &sort).await?;
        let response_headers = headers.response_headers_paged(result.total);
        Ok((response_headers, Json(result)))
    }

    async fn handle_get_by_assigner(
        State(state): State<Self>,
        scope: AccessScope,
        headers: ExtractedHeaders,
        Path(assigner): Path<String>,
        Query(q): Query<AgreementQuery>,
    ) -> AppResult<(HeaderMap, Json<Paginated<AgreementView>>)> {
        let (mut filter, page, sort) = q.into_domain();
        filter.provider_id = Some(assigner);
        let result = state.service.get_all(&scope, &filter, &page, &sort).await?;
        let response_headers = headers.response_headers_paged(result.total);
        Ok((response_headers, Json(result)))
    }

    async fn handle_batch(
        State(state): State<Self>,
        scope: AccessScope,
        headers: ExtractedHeaders,
        payload: Result<Json<BatchRequests>, JsonRejection>,
    ) -> AppResult<(HeaderMap, Json<Vec<AgreementView>>)> {
        let payload = extract_payload(payload)?;
        let views = state.service.batch(&scope, &payload).await?;
        let count = views.len() as u64;
        Ok((headers.response_headers_paged(Some(count)), Json(views)))
    }

    async fn handle_get_one(
        State(state): State<Self>,
        scope: AccessScope,
        headers: ExtractedHeaders,
        Path(id): Path<String>,
    ) -> AppResult<(HeaderMap, Json<AgreementView>)> {
        let urn = extract_path_urn(&id)?;
        let view = state.service.get_one(&scope, &urn).await?;
        Ok((headers.response_headers(), Json(view)))
    }

    async fn handle_create(
        State(state): State<Self>,
        scope: AccessScope,
        headers: ExtractedHeaders,
        payload: Result<Json<NewAgreementDto>, JsonRejection>,
    ) -> AppResult<(StatusCode, HeaderMap, Json<AgreementView>)> {
        let payload = extract_payload(payload)?;
        let view = state.service.create(&scope, &payload).await?;
        let response_headers = headers.response_headers();
        Ok((StatusCode::CREATED, response_headers, Json(view)))
    }

    async fn handle_edit(
        State(state): State<Self>,
        scope: AccessScope,
        headers: ExtractedHeaders,
        Path(id): Path<String>,
        payload: Result<Json<EditAgreementDto>, JsonRejection>,
    ) -> AppResult<(HeaderMap, Json<AgreementView>)> {
        let urn = extract_path_urn(&id)?;
        let payload = extract_payload(payload)?;
        let view = state.service.edit(&scope, &urn, &payload).await?;
        Ok((headers.response_headers(), Json(view)))
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
