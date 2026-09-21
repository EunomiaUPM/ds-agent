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
use common::auth::AccessScope;
use serde::Deserialize;
use ymir::errors::AppResult;

use crate::entities::commands::{EditParameterCommand, NewParameterCommand};
use crate::entities::filters::PrefixFilter;
use crate::entities::key::Key;
use crate::services::parameters::ParameterStore;
use crate::services::parameters::views::{ParameterView, VersionResponse};
use common::query::QuerySpec;

#[derive(Clone)]
pub struct ParameterRouter {
    service: Arc<dyn ParameterStore<serde_json::Value>>,
}

pub type PrefixQuery = QuerySpec<PrefixFilter>;

impl ParameterRouter {
    pub fn new(service: Arc<dyn ParameterStore<serde_json::Value>>) -> Self {
        Self { service }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/", get(Self::list).post(Self::create))
            .route(
                "/{*key}",
                get(Self::read).put(Self::update).delete(Self::delete),
            )
            .with_state(self)
    }

    async fn list(
        State(state): State<ParameterRouter>,
        scope: AccessScope,
        Query(params): Query<PrefixQuery>,
    ) -> AppResult<Json<Vec<ParameterView>>> {
        let items = state.service.list(&scope, &params.filter).await?;
        Ok(Json(items.into_iter().map(ParameterView::from).collect()))
    }

    async fn create(
        State(state): State<ParameterRouter>,
        scope: AccessScope,
        Json(cmd): Json<NewParameterCommand<serde_json::Value>>,
    ) -> AppResult<(StatusCode, Json<ParameterView>)> {
        let entry = state.service.create(&scope, &cmd).await?;
        Ok((StatusCode::CREATED, Json(ParameterView::from(entry))))
    }

    async fn read(
        State(state): State<ParameterRouter>,
        scope: AccessScope,
        Path(key): Path<String>,
    ) -> AppResult<Json<ParameterView>> {
        let key = Key::new(format!("/{}", key))?;
        let entry = state.service.read(&scope, &key).await?;
        Ok(Json(ParameterView::from(entry)))
    }

    async fn update(
        State(state): State<ParameterRouter>,
        scope: AccessScope,
        Path(key): Path<String>,
        Json(cmd): Json<EditParameterCommand<serde_json::Value>>,
    ) -> AppResult<Json<VersionResponse>> {
        let key = Key::new(format!("/{}", key))?;
        let version = state.service.update(&scope, &key, &cmd, "").await?;
        Ok(Json(VersionResponse::from(version)))
    }

    async fn delete(
        State(state): State<ParameterRouter>,
        scope: AccessScope,
        Path(key): Path<String>,
    ) -> AppResult<StatusCode> {
        let key = Key::new(format!("/{}", key))?;
        state.service.delete(&scope, &key).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
