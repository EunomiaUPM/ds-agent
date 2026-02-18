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
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use rainbow_common::batch_requests::BatchRequests;
use rainbow_common::mates::mates::VerifyTokenRequest;
use ymir::errors::CustomToResponse;

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

    async fn get_all(State(mater): State<Arc<dyn CoreMateTrait>>) -> impl IntoResponse {
        match mater.get_all().await {
            Ok(data) => (StatusCode::OK, Json(data)).into_response(),
            Err(e) => e.to_response()
        }
    }
    async fn get_by_id(
        State(mater): State<Arc<dyn CoreMateTrait>>,
        Path(id): Path<String>
    ) -> impl IntoResponse {
        match mater.get_by_id(id).await {
            Ok(data) => (StatusCode::OK, Json(data)).into_response(),
            Err(e) => e.to_response()
        }
    }

    async fn get_me(State(mater): State<Arc<dyn CoreMateTrait>>) -> impl IntoResponse {
        match mater.get_me().await {
            Ok(data) => (StatusCode::OK, Json(data)).into_response(),
            Err(e) => e.to_response()
        }
    }

    async fn get_batch(
        State(mater): State<Arc<dyn CoreMateTrait>>,
        payload: Result<Json<BatchRequests>, JsonRejection>
    ) -> impl IntoResponse {
        let payload = match payload {
            Ok(Json(data)) => data,
            Err(e) => return e.to_response()
        };

        match mater.get_mate_batch(payload).await {
            Ok(data) => (StatusCode::OK, Json(data)).into_response(),
            Err(e) => e.to_response()
        }
    }

    async fn get_by_token(
        State(mater): State<Arc<dyn CoreMateTrait>>,
        payload: Result<Json<VerifyTokenRequest>, JsonRejection>
    ) -> impl IntoResponse {
        let payload = match payload {
            Ok(Json(data)) => data,
            Err(e) => return e.to_response()
        };

        match mater.get_by_token(payload).await {
            Ok(data) => (StatusCode::OK, Json(data)).into_response(),
            Err(e) => e.to_response()
        }
    }
}
