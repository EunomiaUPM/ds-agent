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
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::Value;
use ymir::data::entities::received::grant;
use ymir::errors::AppResult;
use ymir::types::gnap::grant_response::GrantResponse;
use ymir::types::oauth::UserInfo;
use ymir::utils::extract_payload;
use crate::entities::filters::RecvGrantFilter;
use crate::modules::GateKeeperModule;
use common::facades::grants_facade::VerifiedPeer;
use common::facades::VerifyTokenRequest;
use common::paginated_spec::Paginated;
use common::query::{QueryFilter, QuerySpec};
use common::routes::auth::gate;

pub type GateKeeperQuery = QuerySpec<RecvGrantFilter>;

/// GNAP routes peers call to request access to this agent.
pub struct GateKeeperRouter {
    gatekeeper: Arc<dyn GateKeeperModule>,
}

impl GateKeeperRouter {
    // ==========================================================================================
    // Sub-routers
    // ==========================================================================================

    pub fn new(gatekeeper: Arc<dyn GateKeeperModule>) -> Self {
        GateKeeperRouter { gatekeeper }
    }

    /// GNAP endpoints called by peers: grant request and its continuation.
    pub fn external(&self) -> Router {
        Router::new()
            .route(gate::ACCESS, post(Self::manage_req))
            .route(gate::CONTINUE, post(Self::continue_req))
            .route(
                gate::TOKEN,
                post(Self::rotate_token).delete(Self::revoke_token),
            )
            .with_state(self.gatekeeper.clone())
    }

    /// Received grants as users see them; mounted behind the OAuth guard.
    pub fn internal(&self) -> Router {
        Router::new()
            .route(gate::REQUEST_ALL, get(Self::get_all))
            .route(gate::REQUEST, get(Self::get_one).delete(Self::disconnect))
            .route(gate::REQUEST_DETAILS, get(Self::get_one_with_details))
            .route(gate::TOKEN_VERIFY, post(Self::verify_token))
            .with_state(self.gatekeeper.clone())
    }

    // ==========================================================================================
    // External requests: peers, authorities and wallets, authenticated by the protocol (no user)
    // ==========================================================================================

    async fn manage_req(
        State(gatekeeper): State<Arc<dyn GateKeeperModule>>,
        headers: HeaderMap,
        payload: Bytes,
    ) -> AppResult<Json<GrantResponse>> {
        Ok(Json(gatekeeper.manage_grant_req(payload, headers).await))
    }

    async fn continue_req(
        State(gatekeeper): State<Arc<dyn GateKeeperModule>>,
        headers: HeaderMap,
        Path(id): Path<String>,
        payload: Bytes,
    ) -> AppResult<Json<GrantResponse>> {
        Ok(Json(
            gatekeeper.manage_continue_req(&id, payload, headers).await,
        ))
    }

    async fn rotate_token(
        State(gatekeeper): State<Arc<dyn GateKeeperModule>>,
        headers: HeaderMap,
        Path(id): Path<String>,
        payload: Bytes,
    ) -> AppResult<Json<GrantResponse>> {
        Ok(Json(
            gatekeeper.manage_rotation(&id, payload, headers).await,
        ))
    }

    async fn revoke_token(
        State(gatekeeper): State<Arc<dyn GateKeeperModule>>,
        headers: HeaderMap,
        Path(id): Path<String>,
        payload: Bytes,
    ) -> AppResult<StatusCode> {
        gatekeeper.manage_revocation(&id, payload, headers).await?;
        Ok(StatusCode::NO_CONTENT)
    }

    // ==========================================================================================
    // Internal requests: users, behind the OAuth guard (queries)
    // ==========================================================================================

    async fn get_all(
        State(gatekeeper): State<Arc<dyn GateKeeperModule>>,
        user: UserInfo,
        Query(query): Query<GateKeeperQuery>,
    ) -> AppResult<Json<Paginated<grant::Model>>> {
        query.filter.validate()?;
        Ok(Json(
            gatekeeper
                .get_all(&user, &query.filter, &query.page, &query.sort)
                .await?,
        ))
    }

    async fn get_one(
        State(gatekeeper): State<Arc<dyn GateKeeperModule>>,
        user: UserInfo,
        Path(id): Path<String>,
    ) -> AppResult<Json<grant::Model>> {
        Ok(Json(gatekeeper.get_by_id(&user, &id).await?))
    }

    async fn disconnect(
        State(gatekeeper): State<Arc<dyn GateKeeperModule>>,
        user: UserInfo,
        Path(id): Path<String>,
    ) -> AppResult<StatusCode> {
        gatekeeper.disconnect(&user, &id).await?;
        Ok(StatusCode::NO_CONTENT)
    }

    async fn get_one_with_details(
        State(gatekeeper): State<Arc<dyn GateKeeperModule>>,
        user: UserInfo,
        Path(id): Path<String>,
    ) -> AppResult<Json<Value>> {
        Ok(Json(gatekeeper.get_by_id_with_details(&user, &id).await?))
    }

    // ==========================================================================================
    // Internal requests: other agents, through the grants facade
    // ==========================================================================================

    async fn verify_token(
        State(gatekeeper): State<Arc<dyn GateKeeperModule>>,
        payload: Result<Json<VerifyTokenRequest>, JsonRejection>,
    ) -> AppResult<Json<VerifiedPeer>> {
        let payload = extract_payload(payload)?;
        Ok(Json(gatekeeper.verify_token(&payload.token).await?))
    }
}
