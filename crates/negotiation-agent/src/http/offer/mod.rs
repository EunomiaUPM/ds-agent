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

//! HTTP router for offer management endpoints.

use crate::entities::filters::OfferFilter;
use crate::entities::offer::NewOfferDto;
use crate::http::common::ExtractedHeaders;
use crate::services::offer::OfferServiceTrait;
use crate::services::offer::views::OfferView;
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRef, Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use common::auth::access::AccessScope;
use common::batch_requests::BatchRequests;
use common::query::{Paginated, QuerySpec, Sort};
use std::sync::Arc;
use ymir::errors::AppResult;
use ymir::utils::{extract_path_urn, extract_payload};

pub type OfferQuery = QuerySpec<OfferFilter, Sort>;

#[derive(Clone)]
pub struct NegotiationAgentOffersRouter {
    service: Arc<dyn OfferServiceTrait>,
}

impl FromRef<NegotiationAgentOffersRouter> for Arc<dyn OfferServiceTrait> {
    fn from_ref(state: &NegotiationAgentOffersRouter) -> Self {
        state.service.clone()
    }
}

impl NegotiationAgentOffersRouter {
    pub fn new(service: Arc<dyn OfferServiceTrait>) -> Self {
        Self { service }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/", get(Self::handle_get_all).post(Self::handle_create))
            .route("/batch", post(Self::handle_batch))
            .route(
                "/{id}",
                get(Self::handle_get_one).delete(Self::handle_delete),
            )
            .route("/process/{process_id}", get(Self::handle_get_by_process))
            .route("/offer-id/{offer_id}", get(Self::handle_get_by_offer_id))
            .with_state(self)
    }

    async fn handle_get_all(
        State(state): State<Self>,
        scope: AccessScope,
        headers: ExtractedHeaders,
        Query(q): Query<OfferQuery>,
    ) -> AppResult<(HeaderMap, Json<Paginated<OfferView>>)> {
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
        Query(q): Query<OfferQuery>,
    ) -> AppResult<(HeaderMap, Json<Paginated<OfferView>>)> {
        let process_urn = extract_path_urn(&process_id)?;
        let (mut filter, page, sort) = q.into_domain();
        filter.process_id = Some(process_urn.to_string());
        let result = state.service.get_all(&scope, &filter, &page, &sort).await?;
        let response_headers = headers.response_headers_paged(result.total);
        Ok((response_headers, Json(result)))
    }

    async fn handle_get_by_offer_id(
        State(state): State<Self>,
        scope: AccessScope,
        headers: ExtractedHeaders,
        Path(offer_id): Path<String>,
        Query(q): Query<OfferQuery>,
    ) -> AppResult<(HeaderMap, Json<Paginated<OfferView>>)> {
        let (mut filter, page, sort) = q.into_domain();
        filter.offer_id = Some(offer_id);
        let result = state.service.get_all(&scope, &filter, &page, &sort).await?;
        let response_headers = headers.response_headers_paged(result.total);
        Ok((response_headers, Json(result)))
    }

    async fn handle_batch(
        State(state): State<Self>,
        scope: AccessScope,
        headers: ExtractedHeaders,
        payload: Result<Json<BatchRequests>, JsonRejection>,
    ) -> AppResult<(HeaderMap, Json<Vec<OfferView>>)> {
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
    ) -> AppResult<(HeaderMap, Json<OfferView>)> {
        let urn = extract_path_urn(&id)?;
        let view = state.service.get_one(&scope, &urn).await?;
        Ok((headers.response_headers(), Json(view)))
    }

    async fn handle_create(
        State(state): State<Self>,
        scope: AccessScope,
        headers: ExtractedHeaders,
        payload: Result<Json<NewOfferDto>, JsonRejection>,
    ) -> AppResult<(StatusCode, HeaderMap, Json<OfferView>)> {
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
