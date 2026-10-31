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
use axum::routing::get;
use axum::{Json, Router};
use ymir::types::oauth::UserInfo;
use ymir::errors::AppResult;

use crate::entities::commands::{EditParameterCommand, NewParameterCommand};
use crate::entities::filters::PrefixFilter;
use crate::entities::key::Key;
use crate::services::parameters::ParameterStore;
use crate::services::parameters::views::{ParameterView, VersionResponse};
use common::eunomia::document::JsonLd;
use ymir::query::QuerySpec;
use common::routes::keystore as routes;

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
            .route(routes::parameters::ROOT, get(Self::list).post(Self::create))
            .route(
                routes::parameters::BY_KEY,
                get(Self::read).put(Self::update).delete(Self::delete),
            )
            .with_state(self)
    }

    async fn list(
        State(state): State<ParameterRouter>,
        user: UserInfo,
        Query(params): Query<PrefixQuery>,
    ) -> AppResult<Json<Vec<JsonLd<ParameterView>>>> {
        let items = state.service.list(&user, &params.filter).await?;
        Ok(Json(items.into_iter().map(|item| JsonLd(ParameterView::from(item))).collect()))
    }

    async fn create(
        State(state): State<ParameterRouter>,
        user: UserInfo,
        Json(JsonLd(cmd)): Json<JsonLd<NewParameterCommand<serde_json::Value>>>,
    ) -> AppResult<(StatusCode, Json<JsonLd<ParameterView>>)> {
        let entry = state.service.create(&user, &cmd).await?;
        Ok((StatusCode::CREATED, Json(JsonLd(ParameterView::from(entry)))))
    }

    async fn read(
        State(state): State<ParameterRouter>,
        user: UserInfo,
        Path(key): Path<String>,
    ) -> AppResult<Json<JsonLd<ParameterView>>> {
        let key = Key::new(format!("/{key}"))?;
        let entry = state.service.read(&user, &key).await?;
        Ok(Json(JsonLd(ParameterView::from(entry))))
    }

    async fn update(
        State(state): State<ParameterRouter>,
        user: UserInfo,
        Path(key): Path<String>,
        Json(JsonLd(cmd)): Json<JsonLd<EditParameterCommand<serde_json::Value>>>,
    ) -> AppResult<Json<JsonLd<VersionResponse>>> {
        let key = Key::new(format!("/{key}"))?;
        let version = state.service.update(&user, &key, &cmd, "").await?;
        Ok(Json(JsonLd(VersionResponse::from(version))))
    }

    async fn delete(
        State(state): State<ParameterRouter>,
        user: UserInfo,
        Path(key): Path<String>,
    ) -> AppResult<StatusCode> {
        let key = Key::new(format!("/{key}"))?;
        state.service.delete(&user, &key).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
