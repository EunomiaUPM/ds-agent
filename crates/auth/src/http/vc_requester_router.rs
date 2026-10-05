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
use common::oauth::UserInfo;
use common::paginated_spec::Paginated;
use common::query::{QueryFilter, QuerySpec};
use common::routes::auth::vc_request;

pub type VcRequesterQuery = QuerySpec<SentGrantFilter>;

/// Routes to request credentials from an authority and follow the request.
pub struct VcRequesterRouter {
    requester: Arc<dyn VcRequesterModule>,
}

impl VcRequesterRouter {
    // ==========================================================================================
    // Sub-routers
    // ==========================================================================================

    pub fn new(requester: Arc<dyn VcRequesterModule>) -> Self {
        VcRequesterRouter { requester }
    }

    /// GNAP interaction callbacks pushed by the authority.
    pub fn external(&self) -> Router {
        Router::new()
            .route(
                vc_request::CALLBACK,
                get(Self::get_callback).post(Self::post_callback),
            )
            .with_state(self.requester.clone())
    }

    /// User routes to request credentials and follow the requests; mounted behind the OAuth
    /// guard.
    pub fn internal(&self) -> Router {
        Router::new()
            .route(vc_request::BEG, post(Self::beg))
            .route(vc_request::ALL, get(Self::get_all))
            .route(vc_request::BY_ID, get(Self::get_one))
            .route(vc_request::DETAILS, get(Self::get_one_with_details))
            .route(vc_request::OID4VCI, post(Self::manage_oid4vci))
            .route(vc_request::OID4VP, post(Self::manage_oid4vp))
            .with_state(self.requester.clone())
    }

    // ==========================================================================================
    // External requests: peers, authorities and wallets, authenticated by the protocol (no user)
    // ==========================================================================================

    async fn get_callback(
        State(requester): State<Arc<dyn VcRequesterModule>>,
        Path(id): Path<String>,
        Query(payload): Query<CallbackBody>,
    ) -> AppResult<()> {
        requester.manage_interaction_finish(&id, payload).await
    }

    async fn post_callback(
        State(requester): State<Arc<dyn VcRequesterModule>>,
        Path(id): Path<String>,
        payload: Result<Json<CallbackBody>, JsonRejection>,
    ) -> AppResult<()> {
        let payload = extract_payload(payload)?;
        requester.manage_interaction_finish(&id, payload).await
    }

    // ==========================================================================================
    // Internal requests: users, behind the OAuth guard (queries)
    // ==========================================================================================

    async fn get_all(
        State(requester): State<Arc<dyn VcRequesterModule>>,
        user: UserInfo,
        Query(query): Query<VcRequesterQuery>,
    ) -> AppResult<Json<Paginated<Model>>> {
        query.filter.validate()?;
        Ok(Json(
            requester
                .get_all(&user, &query.filter, &query.page, &query.sort)
                .await?,
        ))
    }

    async fn get_one(
        State(requester): State<Arc<dyn VcRequesterModule>>,
        user: UserInfo,
        Path(id): Path<String>,
    ) -> AppResult<Json<Model>> {
        Ok(Json(requester.get_by_id(&user, &id).await?))
    }

    async fn get_one_with_details(
        State(requester): State<Arc<dyn VcRequesterModule>>,
        user: UserInfo,
        Path(id): Path<String>,
    ) -> AppResult<Json<Value>> {
        Ok(Json(requester.get_by_id_with_details(&user, &id).await?))
    }

    // ==========================================================================================
    // Internal requests: users, behind the OAuth guard (actions)
    // ==========================================================================================

    async fn beg(
        State(requester): State<Arc<dyn VcRequesterModule>>,
        user: UserInfo,
        payload: Result<Json<ReachAuthority>, JsonRejection>,
    ) -> AppResult<()> {
        let payload = extract_payload(payload)?;
        requester.beg_vc(&user, payload).await
    }

    async fn manage_oid4vci(
        State(requester): State<Arc<dyn VcRequesterModule>>,
        user: UserInfo,
        Path(id): Path<String>,
        payload: Result<Json<OidcUri>, JsonRejection>,
    ) -> AppResult<()> {
        let payload = extract_payload(payload)?;
        requester.process_oid4vci(&user, &id, payload).await
    }

    async fn manage_oid4vp(
        State(requester): State<Arc<dyn VcRequesterModule>>,
        user: UserInfo,
        Path(id): Path<String>,
        payload: Result<Json<OidcUri>, JsonRejection>,
    ) -> AppResult<()> {
        let payload = extract_payload(payload)?;
        requester.process_oid4vp(&user, &id, payload).await
    }
}
