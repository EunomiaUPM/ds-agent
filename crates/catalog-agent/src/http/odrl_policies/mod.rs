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

//! The ODRL offer API; offers stay ODRL JSON.

use std::sync::Arc;

use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use common::batch_requests::BatchRequests;
use common::query::QuerySpec;
use common::routes::catalog as routes;
use ymir::types::oauth::UserInfo;
use ymir::utils::{extract_path_urn, extract_payload};

use crate::entities::commands::NewOdrlOfferCommand;
use crate::entities::filters::OdrlPolicyFilter;
use crate::entities::odrl_offer::OdrlOffer;
use crate::services::odrl_policies::views::OdrlPolicyView;
use crate::setup::context::AppContext;

pub type OdrlPolicyQuery = QuerySpec<OdrlPolicyFilter>;

#[derive(Clone)]
pub struct OdrlOfferEntityRouter {
    ctx: Arc<AppContext>,
}

impl OdrlOfferEntityRouter {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route(routes::entity::ROOT, get(Self::handle_list).post(Self::handle_create))
            .route(routes::entity::BATCH, post(Self::handle_batch))
            .route(routes::entity::BY_ID, get(Self::handle_get).delete(Self::handle_delete))
            .route(
                routes::odrl_policies::BY_ENTITY,
                get(Self::handle_by_entity).delete(Self::handle_delete_by_entity),
            )
            .with_state(self)
    }

    fn views(offers: Vec<OdrlOffer>) -> Vec<OdrlPolicyView> {
        offers.into_iter().map(OdrlPolicyView::assemble).collect()
    }

    async fn handle_list(State(state): State<Self>, user: UserInfo, Query(query): Query<OdrlPolicyQuery>) -> Response {
        let (filter, page, sort) = query.into_domain();
        match state.ctx.offers().list(&user, &filter, &page, &sort).await {
            Ok(page) => Json(page.map(OdrlPolicyView::assemble)).into_response(),
            Err(e) => e.into_response(),
        }
    }

    async fn handle_batch(
        State(state): State<Self>,
        user: UserInfo,
        input: Result<Json<BatchRequests>, JsonRejection>,
    ) -> Response {
        let input = match extract_payload(input) {
            Ok(input) => input,
            Err(e) => return e.into_response(),
        };
        match state.ctx.offers().batch(&user, &input.ids).await {
            Ok(offers) => Json(Self::views(offers)).into_response(),
            Err(e) => e.into_response(),
        }
    }

    async fn handle_by_entity(State(state): State<Self>, user: UserInfo, Path(id): Path<String>) -> Response {
        let entity = match extract_path_urn(&id) {
            Ok(entity) => entity,
            Err(resp) => return resp.into_response(),
        };
        match state.ctx.offers().by_entities(&user, std::slice::from_ref(&entity)).await {
            Ok(offers) => Json(Self::views(offers)).into_response(),
            Err(e) => e.into_response(),
        }
    }

    async fn handle_get(State(state): State<Self>, user: UserInfo, Path(id): Path<String>) -> Response {
        let id = match extract_path_urn(&id) {
            Ok(id) => id,
            Err(resp) => return resp.into_response(),
        };
        match state.ctx.offers().get(&user, &id).await {
            Ok(offer) => Json(OdrlPolicyView::assemble(offer)).into_response(),
            Err(e) => e.into_response(),
        }
    }

    async fn handle_create(
        State(state): State<Self>,
        user: UserInfo,
        input: Result<Json<NewOdrlOfferCommand>, JsonRejection>,
    ) -> Response {
        let input = match extract_payload(input) {
            Ok(input) => input,
            Err(e) => return e.into_response(),
        };
        match state.ctx.offers().create(&user, &input).await {
            Ok(offer) => (StatusCode::CREATED, Json(OdrlPolicyView::assemble(offer))).into_response(),
            Err(e) => e.into_response(),
        }
    }

    async fn handle_delete(State(state): State<Self>, user: UserInfo, Path(id): Path<String>) -> Response {
        let id = match extract_path_urn(&id) {
            Ok(id) => id,
            Err(resp) => return resp.into_response(),
        };
        match state.ctx.offers().delete(&user, &id).await {
            Ok(()) => StatusCode::NO_CONTENT.into_response(),
            Err(e) => e.into_response(),
        }
    }

    async fn handle_delete_by_entity(State(state): State<Self>, user: UserInfo, Path(id): Path<String>) -> Response {
        let entity = match extract_path_urn(&id) {
            Ok(entity) => entity,
            Err(resp) => return resp.into_response(),
        };
        match state.ctx.offers().delete_by_entity(&user, &entity).await {
            Ok(()) => StatusCode::NO_CONTENT.into_response(),
            Err(e) => e.into_response(),
        }
    }
}
