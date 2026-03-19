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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::sync::Arc;

use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use rainbow_common::batch_requests::BatchRequests;
use rainbow_common::mates::mates::VerifyTokenRequest;
use ymir::data::entities::mates::Model;
use ymir::errors::AppResult;
use ymir::utils::extract_payload;

use crate::ssi::core::traits::CoreMateTrait;

pub struct MateRouter {
    mater: Arc<dyn CoreMateTrait>
}

impl MateRouter {
    pub fn new(mater: Arc<dyn CoreMateTrait>) -> MateRouter { MateRouter { mater } }

    pub fn router(self) -> Router {
        Router::new()
            .route("/all", get(Self::get_all))
            .route("/{id}", get(Self::get_by_id))
            .route("/myself", get(Self::get_me))
            .route("/batch", post(Self::get_batch))
            .route("/token", post(Self::get_by_token))
            .with_state(self.mater)
    }

    async fn get_all(State(mater): State<Arc<dyn CoreMateTrait>>) -> AppResult<Json<Vec<Model>>> {
        Ok(Json(mater.get_all().await?))
    }
    async fn get_by_id(
        State(mater): State<Arc<dyn CoreMateTrait>>,
        Path(id): Path<String>
    ) -> AppResult<Json<Model>> {
        Ok(Json(mater.get_by_id(id).await?))
    }

    async fn get_me(State(mater): State<Arc<dyn CoreMateTrait>>) -> AppResult<Json<Model>> {
        Ok(Json(mater.get_me().await?))
    }

    async fn get_batch(
        State(mater): State<Arc<dyn CoreMateTrait>>,
        payload: Result<Json<BatchRequests>, JsonRejection>
    ) -> AppResult<Json<Vec<Model>>> {
        let payload = extract_payload(payload)?;
        Ok(Json(mater.get_mate_batch(payload).await?))
    }

    async fn get_by_token(
        State(mater): State<Arc<dyn CoreMateTrait>>,
        payload: Result<Json<VerifyTokenRequest>, JsonRejection>
    ) -> AppResult<Json<Model>> {
        let payload = extract_payload(payload)?;
        Ok(Json(mater.get_by_token(payload).await?))
    }
}
