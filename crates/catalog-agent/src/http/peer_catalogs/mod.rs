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

//! The catalogs fetched from peers, as they sent them.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use common::routes::catalog as routes;
use ymir::errors::Errors;
use ymir::types::oauth::UserInfo;

use crate::setup::context::AppContext;

#[derive(Clone)]
pub struct PeerCatalogRouter {
    ctx: Arc<AppContext>,
}

impl PeerCatalogRouter {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route(routes::peer_catalogs::ROOT, get(Self::handle_get_all))
            .route(routes::peer_catalogs::BY_PEER, get(Self::handle_get))
            .with_state(self)
    }

    /// Pairs of the peer and its catalog, as before: `[[peer, catalog], …]`.
    async fn handle_get_all(State(state): State<Self>, user: UserInfo) -> Response {
        match state.ctx.peer_catalogs().get_all(&user).await {
            Ok(found) => (StatusCode::OK, Json(found)).into_response(),
            Err(e) => e.into_response(),
        }
    }

    async fn handle_get(State(state): State<Self>, user: UserInfo, Path(peer_id): Path<String>) -> Response {
        match state.ctx.peer_catalogs().get(&user, &peer_id).await {
            Ok(Some(catalog)) => (StatusCode::OK, Json(catalog)).into_response(),
            Ok(None) => Errors::missing_resource(peer_id, "peer catalog not found", None).into_response(),
            Err(e) => e.into_response(),
        }
    }
}
