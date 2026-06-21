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

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::routing::post;
use axum::{Json, Router};
use ymir::errors::AppResult;
use ymir::types::gnap::grant_response::GrantResponse;

use crate::modules::GateKeeperModule;

pub struct GateKeeperRouter {
    gatekeeper: Arc<dyn GateKeeperModule>,
}

impl GateKeeperRouter {
    pub fn new(gatekeeper: Arc<dyn GateKeeperModule>) -> Self {
        GateKeeperRouter { gatekeeper }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/access", post(Self::manage_req))
            .route("/continue/{id}", post(Self::continue_req))
            .with_state(self.gatekeeper)
    }

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
            gatekeeper.manage_continue_req(id, payload, headers).await,
        ))
    }
}
