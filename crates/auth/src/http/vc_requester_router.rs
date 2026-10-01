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
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::Value;
use ymir::data::entities::sent::grant::Model;
use ymir::errors::AppResult;
use ymir::types::gnap::CallbackBody;
use ymir::types::wallet::OidcUri;
use ymir::utils::extract_payload;

use crate::entities::filters::SentGrantFilter;
use crate::modules::VcRequesterModule;
use crate::types::entities::ReachAuthority;
use common::auth::AccessScope;
use common::paginated_spec::Paginated;
use common::query::{QueryFilter, QuerySpec};

pub type VcRequesterQuery = QuerySpec<SentGrantFilter>;

pub struct VcRequesterRouter {
    requester: Arc<dyn VcRequesterModule>,
}

impl VcRequesterRouter {
    pub fn new(requester: Arc<dyn VcRequesterModule>) -> Self {
        VcRequesterRouter { requester }
    }

    /// GNAP interaction callbacks pushed by the authority.
    pub fn protocol_router(&self) -> Router {
        Router::new()
            .route(
                "/callback/{id}",
                get(Self::get_callback).post(Self::post_callback),
            )
            .with_state(self.requester.clone())
    }

    pub fn router(&self) -> Router {
        Router::new()
            .route("/beg", post(Self::beg))
            .route("/all", get(Self::get_all))
            .route("/{id}", get(Self::get_one))
            .route("/{id}/details", get(Self::get_one_with_details))
            .route("/oid4vci/{id}", post(Self::manage_oid4vci))
            .route("/oid4vp/{id}", post(Self::manage_oid4vp))
            .with_state(self.requester.clone())
    }

    async fn beg(
        State(requester): State<Arc<dyn VcRequesterModule>>,
        scope: AccessScope,
        payload: Result<Json<ReachAuthority>, JsonRejection>,
    ) -> AppResult<()> {
        let payload = extract_payload(payload)?;
        requester.beg_vc(&scope, payload).await
    }

    async fn get_all(
        State(requester): State<Arc<dyn VcRequesterModule>>,
        scope: AccessScope,
        Query(query): Query<VcRequesterQuery>,
    ) -> AppResult<Json<Paginated<Model>>> {
        query.filter.validate()?;
        Ok(Json(
            requester
                .get_all(&scope, &query.filter, &query.page, &query.sort)
                .await?,
        ))
    }

    async fn get_one(
        State(requester): State<Arc<dyn VcRequesterModule>>,
        scope: AccessScope,
        Path(id): Path<String>,
    ) -> AppResult<Json<Model>> {
        Ok(Json(requester.get_by_id(&scope, id).await?))
    }

    async fn get_one_with_details(
        State(requester): State<Arc<dyn VcRequesterModule>>,
        scope: AccessScope,
        Path(id): Path<String>,
    ) -> AppResult<Json<Value>> {
        Ok(Json(requester.get_by_id_with_details(&scope, id).await?))
    }

    async fn get_callback(
        State(requester): State<Arc<dyn VcRequesterModule>>,
        Path(id): Path<String>,
        Query(payload): Query<CallbackBody>,
    ) -> AppResult<()> {
        requester.manage_interaction_finish(id, payload).await
    }

    async fn post_callback(
        State(requester): State<Arc<dyn VcRequesterModule>>,
        Path(id): Path<String>,
        payload: Result<Json<CallbackBody>, JsonRejection>,
    ) -> AppResult<()> {
        let payload = extract_payload(payload)?;
        requester.manage_interaction_finish(id, payload).await
    }

    async fn manage_oid4vci(
        State(requester): State<Arc<dyn VcRequesterModule>>,
        scope: AccessScope,
        Path(id): Path<String>,
        payload: Result<Json<OidcUri>, JsonRejection>,
    ) -> AppResult<()> {
        let payload = extract_payload(payload)?;
        requester.process_oid4vci(&scope, id, payload).await
    }

    async fn manage_oid4vp(
        State(requester): State<Arc<dyn VcRequesterModule>>,
        scope: AccessScope,
        Path(id): Path<String>,
        payload: Result<Json<OidcUri>, JsonRejection>,
    ) -> AppResult<()> {
        let payload = extract_payload(payload)?;
        requester.process_oid4vp(&scope, id, payload).await
    }
}
