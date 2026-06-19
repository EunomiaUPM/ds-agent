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

use crate::modules::CoreMateTrait;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use common::batch_requests::BatchRequests;
use common::facades::VerifyTokenRequest;
use serde::Deserialize;
use ymir::data::entities::shared::participant::{Model, Plan};
use ymir::errors::AppResult;
use ymir::types::participants::ParticipantType;
use ymir::utils::extract_payload;

pub struct ParticipantRouter {
    manager: Arc<dyn CoreMateTrait>,
}

impl ParticipantRouter {
    pub fn new(mater: Arc<dyn CoreMateTrait>) -> ParticipantRouter {
        ParticipantRouter { manager: mater }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/all", get(Self::get_all))
            .route("/myself", get(Self::get_myself))
            .route("/{id}", get(Self::get_by_id))
            .route("/batch", post(Self::get_batch))
            .route("/token", post(Self::get_by_token))
            .route("/{id}", put(Self::update_by_id))
            .route("/", post(Self::create))
            .with_state(self.manager)
    }

    async fn get_all(
        State(mater): State<Arc<dyn CoreMateTrait>>,
        query: Query<MateQuery>,
    ) -> AppResult<Json<Vec<Model>>> {
        let filter = query.r#type.unwrap_or(ParticipantType::All);

        Ok(Json(mater.get_all(filter, query.exclude_myself).await?))
    }
    async fn get_by_id(
        State(mater): State<Arc<dyn CoreMateTrait>>,
        Path(id): Path<String>,
    ) -> AppResult<Json<Model>> {
        Ok(Json(mater.get_by_id(id).await?))
    }

    async fn get_myself(State(mater): State<Arc<dyn CoreMateTrait>>) -> AppResult<Json<Model>> {
        Ok(Json(mater.get_me().await?))
    }

    async fn get_batch(
        State(mater): State<Arc<dyn CoreMateTrait>>,
        payload: Result<Json<BatchRequests>, JsonRejection>,
    ) -> AppResult<Json<Vec<Model>>> {
        let payload = extract_payload(payload)?;
        Ok(Json(mater.get_mate_batch(payload).await?))
    }

    async fn get_by_token(
        State(mater): State<Arc<dyn CoreMateTrait>>,
        payload: Result<Json<VerifyTokenRequest>, JsonRejection>,
    ) -> AppResult<Json<Model>> {
        let payload = extract_payload(payload)?;
        Ok(Json(mater.get_by_token(payload).await?))
    }
    async fn update_by_id(
        State(mater): State<Arc<dyn CoreMateTrait>>,
        Path(id): Path<String>,
        payload: Result<Json<serde_json::Value>, JsonRejection>,
    ) -> AppResult<Json<Model>> {
        let payload = extract_payload(payload)?;
        Ok(Json(mater.update_extra_fields_by_id(id, payload).await?))
    }
    async fn create(
        State(mater): State<Arc<dyn CoreMateTrait>>,
        payload: Result<Json<Plan>, JsonRejection>,
    ) -> AppResult<Json<Model>> {
        let payload = extract_payload(payload)?;
        Ok(Json(mater.create_mate(&payload).await?))
    }
}

#[derive(Deserialize)]
struct MateQuery {
    r#type: Option<ParticipantType>,
    #[serde(default)]
    exclude_myself: bool,
}
