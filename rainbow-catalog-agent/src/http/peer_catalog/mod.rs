/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use crate::entities::catalogs::{CatalogEntityTrait, EditCatalogDto, NewCatalogDto};
use crate::entities::peer_catalogs::PeerCatalogTrait;
use crate::http::common::to_camel_case::ToCamelCase;
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRef, Path, Query, State};
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use rainbow_common::batch_requests::BatchRequests;
use rainbow_common::config::services::CatalogConfig;
use rainbow_common::errors::CommonErrors;
use reqwest::StatusCode;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone)]
pub struct PeerCatalogEntityRouter {
    service: Arc<dyn PeerCatalogTrait>,
}

impl FromRef<PeerCatalogEntityRouter> for Arc<dyn PeerCatalogTrait> {
    fn from_ref(state: &PeerCatalogEntityRouter) -> Self {
        state.service.clone()
    }
}

impl PeerCatalogEntityRouter {
    pub fn new(service: Arc<dyn PeerCatalogTrait>) -> Self {
        Self { service }
    }

    pub fn router(self) -> Router {
        Router::new().route("/{peer_id}", get(Self::handle_get_catalog_by_peer_id)).with_state(self)
    }

    async fn handle_get_catalog_by_peer_id(
        State(state): State<PeerCatalogEntityRouter>,
        Path(peer_id): Path<String>,
    ) -> impl IntoResponse {
        match state.service.get_peer_catalog(&peer_id).await {
            Ok(Some(catalog)) => (StatusCode::OK, Json(catalog)).into_response(),
            Ok(None) => {
                let err =
                    CommonErrors::missing_resource_new("peer catalog", "Peer Catalog not found");
                err.into_response()
            }
            Err(e) => return e.into_response(),
        }
    }
}
