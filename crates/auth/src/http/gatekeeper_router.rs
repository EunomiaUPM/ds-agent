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

use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::Value;
use ymir::data::entities::received::grant;
use ymir::errors::AppResult;
use ymir::types::gnap::grant_response::GrantResponse;

use crate::entities::filters::RecvGrantFilter;
use crate::modules::GateKeeperModule;
use common::auth::AccessScope;
use common::paginated_spec::Paginated;
use common::query::{QueryFilter, QuerySpec};

pub type GateKeeperQuery = QuerySpec<RecvGrantFilter>;

pub struct GateKeeperRouter {
    gatekeeper: Arc<dyn GateKeeperModule>,
}

impl GateKeeperRouter {
    pub fn new(gatekeeper: Arc<dyn GateKeeperModule>) -> Self {
        GateKeeperRouter { gatekeeper }
    }

    /// GNAP endpoints called by peers; the path names the tenant being onboarded into.
    pub fn protocol_router(&self) -> Router {
        Router::new()
            .route("/{tenant}/access", post(Self::manage_req))
            .route("/{tenant}/continue/{id}", post(Self::continue_req))
            .with_state(self.gatekeeper.clone())
    }

    pub fn router(&self) -> Router {
        Router::new()
            .route("/request/all", get(Self::get_all))
            .route("/request/{id}", get(Self::get_one))
            .route("/request/{id}/details", get(Self::get_one_with_details))
            .with_state(self.gatekeeper.clone())
    }

    async fn manage_req(
        State(gatekeeper): State<Arc<dyn GateKeeperModule>>,
        Path(tenant): Path<String>,
        headers: HeaderMap,
        payload: Bytes,
    ) -> AppResult<Json<GrantResponse>> {
        Ok(Json(
            gatekeeper.manage_grant_req(tenant, payload, headers).await,
        ))
    }

    async fn continue_req(
        State(gatekeeper): State<Arc<dyn GateKeeperModule>>,
        headers: HeaderMap,
        Path((tenant, id)): Path<(String, String)>,
        payload: Bytes,
    ) -> AppResult<Json<GrantResponse>> {
        Ok(Json(
            gatekeeper
                .manage_continue_req(tenant, id, payload, headers)
                .await,
        ))
    }

    async fn get_all(
        State(gatekeeper): State<Arc<dyn GateKeeperModule>>,
        scope: AccessScope,
        Query(query): Query<GateKeeperQuery>,
    ) -> AppResult<Json<Paginated<grant::Model>>> {
        query.filter.validate()?;
        Ok(Json(
            gatekeeper
                .get_all(&scope, &query.filter, &query.page, &query.sort)
                .await?,
        ))
    }

    async fn get_one(
        State(gatekeeper): State<Arc<dyn GateKeeperModule>>,
        scope: AccessScope,
        Path(id): Path<String>,
    ) -> AppResult<Json<grant::Model>> {
        Ok(Json(gatekeeper.get_by_id(&scope, id).await?))
    }

    async fn get_one_with_details(
        State(gatekeeper): State<Arc<dyn GateKeeperModule>>,
        scope: AccessScope,
        Path(id): Path<String>,
    ) -> AppResult<Json<Value>> {
        Ok(Json(gatekeeper.get_by_id_with_details(&scope, id).await?))
    }
}
