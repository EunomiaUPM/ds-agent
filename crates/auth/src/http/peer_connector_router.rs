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
use ymir::data::entities::sent::grant;
use ymir::errors::AppResult;
use ymir::types::gnap::CallbackBody;
use ymir::types::oauth::UserInfo;
use ymir::types::wallet::OidcUri;
use ymir::utils::extract_payload;

use crate::entities::filters::SentGrantFilter;
use crate::http::path_id::decode_path_id;
use crate::modules::PeerConnectorModule;
use crate::types::entities::ReachProvider;
use common::paginated_spec::Paginated;
use common::query::{QueryFilter, QuerySpec};
use common::routes::auth::peer_connection;

pub type PeerConnectorQuery = QuerySpec<SentGrantFilter>;

/// Routes to start and follow onboarding with a peer.
pub struct OnboarderRouter {
    peer_connector: Arc<dyn PeerConnectorModule>,
}

impl OnboarderRouter {
    // ==========================================================================================
    // Sub-routers
    // ==========================================================================================

    pub fn new(peer_connector: Arc<dyn PeerConnectorModule>) -> Self {
        Self { peer_connector }
    }

    /// GNAP interaction callbacks pushed by the peer's authorization server.
    pub fn external(&self) -> Router {
        Router::new()
            .route(
                peer_connection::CALLBACK,
                get(Self::get_callback).post(Self::post_callback),
            )
            .with_state(self.peer_connector.clone())
    }

    /// User routes to start and follow onboarding; mounted behind the OAuth guard.
    pub fn internal(&self) -> Router {
        Router::new()
            .route(peer_connection::CONNECT, post(Self::connect))
            .route(peer_connection::REQUEST_ALL, get(Self::get_all))
            .route(peer_connection::REQUEST, get(Self::get_one))
            .route(peer_connection::REQUEST_DETAILS, get(Self::get_one_with_details))
            .route(peer_connection::OID4VP, post(Self::manage_oid4vp))
            .route(peer_connection::TOKEN, get(Self::peer_token))
            .with_state(self.peer_connector.clone())
    }

    // ==========================================================================================
    // External requests: peers, authorities and wallets, authenticated by the protocol (no user)
    // ==========================================================================================

    async fn get_callback(
        State(peer_connector): State<Arc<dyn PeerConnectorModule>>,
        Path(id): Path<String>,
        Query(params): Query<CallbackBody>,
    ) -> AppResult<()> {
        peer_connector.manage_interaction_finish(&id, params).await
    }

    async fn post_callback(
        State(peer_connector): State<Arc<dyn PeerConnectorModule>>,
        Path(id): Path<String>,
        payload: Result<Json<CallbackBody>, JsonRejection>,
    ) -> AppResult<()> {
        let payload = extract_payload(payload)?;
        peer_connector.manage_interaction_finish(&id, payload).await
    }

    // ==========================================================================================
    // Internal requests: users, behind the OAuth guard (queries)
    // ==========================================================================================

    async fn get_all(
        State(peer_connector): State<Arc<dyn PeerConnectorModule>>,
        user: UserInfo,
        Query(query): Query<PeerConnectorQuery>,
    ) -> AppResult<Json<Paginated<grant::Model>>> {
        query.filter.validate()?;
        Ok(Json(
            peer_connector
                .get_all(&user, &query.filter, &query.page, &query.sort)
                .await?,
        ))
    }

    async fn get_one(
        State(peer_connector): State<Arc<dyn PeerConnectorModule>>,
        user: UserInfo,
        Path(id): Path<String>,
    ) -> AppResult<Json<grant::Model>> {
        Ok(Json(peer_connector.get_by_id(&user, &id).await?))
    }

    async fn get_one_with_details(
        State(peer_connector): State<Arc<dyn PeerConnectorModule>>,
        user: UserInfo,
        Path(id): Path<String>,
    ) -> AppResult<Json<Value>> {
        Ok(Json(
            peer_connector.get_by_id_with_details(&user, &id).await?,
        ))
    }

    /// The caller's token towards peer `id` (in base64url, see `path_id`), for the grants
    /// facade; `null` if it has none.
    async fn peer_token(
        State(peer_connector): State<Arc<dyn PeerConnectorModule>>,
        user: UserInfo,
        Path(id): Path<String>,
    ) -> AppResult<Json<Option<String>>> {
        let id = decode_path_id(&id)?;
        Ok(Json(peer_connector.peer_token(&user, &id).await?))
    }

    // ==========================================================================================
    // Internal requests: users, behind the OAuth guard (actions)
    // ==========================================================================================

    async fn connect(
        State(peer_connector): State<Arc<dyn PeerConnectorModule>>,
        user: UserInfo,
        payload: Result<Json<ReachProvider>, JsonRejection>,
    ) -> AppResult<()> {
        let payload = extract_payload(payload)?;
        peer_connector.req_peer_connection(&user, payload).await
    }

    async fn manage_oid4vp(
        State(peer_connector): State<Arc<dyn PeerConnectorModule>>,
        user: UserInfo,
        Path(id): Path<String>,
        payload: Result<Json<OidcUri>, JsonRejection>,
    ) -> AppResult<()> {
        let payload = extract_payload(payload)?;
        peer_connector.process_oid4vp(&user, &id, payload).await
    }
}
