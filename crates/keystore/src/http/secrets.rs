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

use crate::entities::commands::{EditSecretCommand, NewSecretCommand};
use crate::entities::key::{Key, KeyPrefix};
use crate::services::secrets::SecretStore;
use crate::services::secrets::views::{SecretMetadataView, SecretView, VersionResponse};

#[derive(Clone)]
pub struct SecretRouter {
    service: Arc<dyn SecretStore>,
}

#[derive(Deserialize)]
pub struct PrefixQuery {
    pub prefix: Option<String>,
}

impl SecretRouter {
    pub fn new(service: Arc<dyn SecretStore>) -> Self {
        Self { service }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/", get(Self::list))
            .route("/", post(Self::create))
            .route("/{*key}", get(Self::read))
            .route("/{*key}", put(Self::update))
            .route("/{*key}", delete(Self::delete))
            .with_state(self)
    }

    async fn list(
        State(state): State<SecretRouter>,
        Query(params): Query<PrefixQuery>,
    ) -> AppResult<Json<Vec<SecretMetadataView>>> {
        let prefix = KeyPrefix::new(params.prefix.unwrap_or_default());
        let items = state.service.list(&prefix).await?;
        Ok(Json(
            items.into_iter().map(SecretMetadataView::from).collect(),
        ))
    }

    async fn create(
        State(state): State<SecretRouter>,
        Json(cmd): Json<NewSecretCommand>,
    ) -> AppResult<(StatusCode, Json<VersionResponse>)> {
        let version = state.service.create(&cmd).await?;
        Ok((StatusCode::CREATED, Json(VersionResponse::from(version))))
    }

    async fn read(
        State(state): State<SecretRouter>,
        Path(key): Path<String>,
    ) -> AppResult<Json<SecretView>> {
        let key = Key::new(format!("/{}", key))?;
        let entry = state.service.read(&key).await?;
        Ok(Json(SecretView::from(entry)))
    }

    async fn update(
        State(state): State<SecretRouter>,
        Path(key): Path<String>,
        Json(cmd): Json<EditSecretCommand>,
    ) -> AppResult<Json<VersionResponse>> {
        let key = Key::new(format!("/{}", key))?;
        let version = state.service.update(&key, &cmd).await?;
        Ok(Json(VersionResponse::from(version)))
    }

    async fn delete(
        State(state): State<SecretRouter>,
        Path(key): Path<String>,
    ) -> AppResult<StatusCode> {
        let key = Key::new(format!("/{}", key))?;
        state.service.delete(&key).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
