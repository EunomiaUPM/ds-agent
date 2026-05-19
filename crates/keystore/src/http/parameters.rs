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

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use serde::Deserialize;
use ymir::errors::AppResult;

use crate::entities::commands::{EditParameterCommand, NewParameterCommand};
use crate::entities::key::{Key, KeyPrefix};
use crate::services::parameters::ParameterStore;
use crate::services::parameters::views::{ParameterMetadataView, ParameterView, VersionResponse};

#[derive(Clone)]
pub struct ParameterRouter {
    service: Arc<dyn ParameterStore<serde_json::Value>>,
}

#[derive(Deserialize)]
pub struct PrefixQuery {
    pub prefix: Option<String>,
}

impl ParameterRouter {
    pub fn new(service: Arc<dyn ParameterStore<serde_json::Value>>) -> Self {
        Self { service }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/", get(Self::list))
            .route("/", post(Self::create))
            .route("/{key}", get(Self::read))
            .route("/{key}", put(Self::update))
            .route("/{key}", delete(Self::delete))
            .with_state(self)
    }

    async fn list(
        State(state): State<ParameterRouter>,
        Query(params): Query<PrefixQuery>,
    ) -> AppResult<Json<Vec<ParameterMetadataView>>> {
        let prefix = KeyPrefix::new(params.prefix.unwrap_or_default());
        let items = state.service.list(&prefix).await?;
        Ok(Json(
            items.into_iter().map(ParameterMetadataView::from).collect(),
        ))
    }

    async fn create(
        State(state): State<ParameterRouter>,
        Json(cmd): Json<NewParameterCommand<serde_json::Value>>,
    ) -> AppResult<(StatusCode, Json<VersionResponse>)> {
        let version = state.service.create(&cmd).await?;
        Ok((StatusCode::CREATED, Json(VersionResponse::from(version))))
    }

    async fn read(
        State(state): State<ParameterRouter>,
        Path(key): Path<String>,
    ) -> AppResult<Json<ParameterView>> {
        let key = Key::new(key)?;
        let entry = state.service.read(&key).await?;
        Ok(Json(ParameterView::from(entry)))
    }

    async fn update(
        State(state): State<ParameterRouter>,
        Path(key): Path<String>,
        Json(cmd): Json<EditParameterCommand<serde_json::Value>>,
    ) -> AppResult<Json<VersionResponse>> {
        let key = Key::new(key)?;
        let version = state.service.update(&key, &cmd, "").await?;
        Ok(Json(VersionResponse::from(version)))
    }

    async fn delete(
        State(state): State<ParameterRouter>,
        Path(key): Path<String>,
    ) -> AppResult<StatusCode> {
        let key = Key::new(key)?;
        state.service.delete(&key).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
