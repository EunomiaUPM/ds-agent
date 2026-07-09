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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
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
use ymir::types::wallet::OidcUri;
use ymir::utils::extract_payload;

use crate::modules::PeerConnectorModule;
use crate::types::entities::ReachProvider;

pub struct OnboarderRouter {
    peer_connector: Arc<dyn PeerConnectorModule>,
}

impl OnboarderRouter {
    pub fn new(peer_connector: Arc<dyn PeerConnectorModule>) -> Self {
        Self { peer_connector }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/connect", post(Self::connect))
            .route("/request/all", get(Self::get_all))
            .route("/request/{id}", get(Self::get_one))
            .route("/request/{id}/details", get(Self::get_one_with_details))
            .route(
                "/callback/{id}",
                get(Self::get_callback).post(Self::post_callback),
            )
            .route("/oid4vp/{id}", post(Self::manage_oid4vp))
            .with_state(self.peer_connector)
    }

    async fn connect(
        State(peer_connector): State<Arc<dyn PeerConnectorModule>>,
        payload: Result<Json<ReachProvider>, JsonRejection>,
    ) -> AppResult<()> {
        let payload = extract_payload(payload)?;
        peer_connector.req_peer_connection(payload).await
    }

    async fn get_callback(
        State(peer_connector): State<Arc<dyn PeerConnectorModule>>,
        Path(id): Path<String>,
        Query(params): Query<CallbackBody>,
    ) -> AppResult<()> {
        peer_connector.manage_interaction_finish(id, params).await
    }

    async fn post_callback(
        State(peer_connector): State<Arc<dyn PeerConnectorModule>>,
        Path(id): Path<String>,
        payload: Result<Json<CallbackBody>, JsonRejection>,
    ) -> AppResult<()> {
        let payload = extract_payload(payload)?;
        peer_connector.manage_interaction_finish(id, payload).await
    }

    async fn get_all(
        State(peer_connector): State<Arc<dyn PeerConnectorModule>>,
    ) -> AppResult<Json<Vec<grant::Model>>> {
        Ok(Json(peer_connector.get_all().await?))
    }

    async fn get_one(
        State(peer_connector): State<Arc<dyn PeerConnectorModule>>,
        Path(id): Path<String>,
    ) -> AppResult<Json<grant::Model>> {
        Ok(Json(peer_connector.get_by_id(id).await?))
    }

    async fn get_one_with_details(
        State(peer_connector): State<Arc<dyn PeerConnectorModule>>,
        Path(id): Path<String>,
    ) -> AppResult<Json<Value>> {
        Ok(Json(peer_connector.get_by_id_with_details(id).await?))
    }
    
    async fn manage_oid4vp(
        State(peer_connector): State<Arc<dyn PeerConnectorModule>>,
        Path(id): Path<String>,
        payload: Result<Json<OidcUri>, JsonRejection>,
    ) -> AppResult<()> {
        let payload = extract_payload(payload)?;
        peer_connector.process_oid4vp(id, payload).await
    }
}
