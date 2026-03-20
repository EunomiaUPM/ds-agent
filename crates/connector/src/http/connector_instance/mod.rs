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

use crate::entities::connector_instance::{ConnectorInstanceTrait, ConnectorInstantiationDto};
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRef, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use std::sync::Arc;
use ymir::errors::Errors;
use ymir::utils::{extract_path_urn, extract_payload};

#[derive(Clone)]
pub struct ConnectorInstanceRouter {
    service: Arc<dyn ConnectorInstanceTrait>,
}

impl FromRef<ConnectorInstanceRouter> for Arc<dyn ConnectorInstanceTrait> {
    fn from_ref(state: &ConnectorInstanceRouter) -> Self {
        state.service.clone()
    }
}

impl ConnectorInstanceRouter {
    pub fn new(service: Arc<dyn ConnectorInstanceTrait>) -> Self {
        Self { service }
    }
    pub fn router(self) -> Router {
        Router::new()
            .route("/", post(Self::handle_upsert_instance))
            .route("/{id}", get(Self::handle_get_instance_by_id))
            .route(
                "/distribution/{did}",
                get(Self::get_instance_by_distribution),
            )
            .route("/{id}", delete(Self::handle_delete_instance_by_id))
            .with_state(self)
    }

    async fn handle_upsert_instance(
        State(state): State<ConnectorInstanceRouter>,
        input: Result<Json<ConnectorInstantiationDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let mut input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        match state.service.upsert_instance(&mut input).await {
            Ok(instance) => (StatusCode::OK, Json(instance)).into_response(),
            Err(err) => err.into_response(),
        }
    }
    async fn handle_get_instance_by_id(
        State(state): State<ConnectorInstanceRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        let id = match extract_path_urn(&id) {
            Ok(urn) => urn,
            Err(err) => return err.into_response(),
        };
        match state.service.get_instance_by_id(&id).await {
            Ok(Some(instance)) => (StatusCode::OK, Json(instance)).into_response(),
            Ok(None) => {
                let err = Errors::missing_resource("instance", "Instance not found", None);
                err.into_response()
            }
            Err(err) => err.into_response(),
        }
    }
    async fn get_instance_by_distribution(
        State(state): State<ConnectorInstanceRouter>,
        Path(did): Path<String>,
    ) -> impl IntoResponse {
        let did = match extract_path_urn(&did) {
            Ok(urn) => urn,
            Err(err) => return err.into_response(),
        };
        match state.service.get_instance_by_distribution(&did).await {
            Ok(Some(instance)) => (StatusCode::OK, Json(instance)).into_response(),
            Ok(None) => {
                let err = Errors::missing_resource("instance", "Instance not found", None);
                err.into_response()
            }
            Err(err) => err.into_response(),
        }
    }
    async fn handle_delete_instance_by_id(
        State(state): State<ConnectorInstanceRouter>,
        Path(did): Path<String>,
    ) -> impl IntoResponse {
        let did = match extract_path_urn(&did) {
            Ok(urn) => urn,
            Err(err) => return err.into_response(),
        };
        match state.service.delete_instance_by_id(&did).await {
            Ok(_) => StatusCode::ACCEPTED.into_response(),
            Err(err) => err.into_response(),
        }
    }
}
