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

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use tracing::error;
use ymir::errors::{CustomToResponse, ErrorLogTrait, Errors};
use ymir::types::errors::BadFormat;
use ymir::types::gnap::{ApprovedCallbackBody, CallbackBody};

use crate::ssi::core::traits::CoreOnboarderTrait;
use crate::ssi::types::entities::ReachProvider;

pub struct OnboarderRouter {
    onboarder: Arc<dyn CoreOnboarderTrait>,
}

impl OnboarderRouter {
    pub fn new(onboarder: Arc<dyn CoreOnboarderTrait>) -> Self {
        Self { onboarder }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/provider", post(Self::onboard))
            .route("/callback/{id}", get(Self::get_callback))
            .route("/callback/{id}", post(Self::post_callback))
            .with_state(self.onboarder)
    }

    async fn onboard(
        State(onboarder): State<Arc<dyn CoreOnboarderTrait>>,
        payload: Result<Json<ReachProvider>, JsonRejection>,
    ) -> impl IntoResponse {
        let payload = match payload {
            Ok(Json(data)) => data,
            Err(e) => {
                return e.to_response();
            }
        };

        match onboarder.onboard_req(payload).await {
            Ok(uri) => uri.into_response(),
            Err(e) => e.to_response(),
        }
    }

    async fn get_callback(
        State(onboarder): State<Arc<dyn CoreOnboarderTrait>>,
        Path(id): Path<String>,
        Query(params): Query<HashMap<String, String>>,
    ) -> impl IntoResponse {
        let hash = match params.get("hash") {
            Some(hash) => hash.clone(),
            None => {
                let error = Errors::format_new(
                    BadFormat::Received,
                    "Unable to retrieve hash from callback",
                );
                error!("{}", error.log());
                return error.into_response();
            }
        };

        let interact_ref = match params.get("interact_ref") {
            Some(interact_ref) => interact_ref.clone(),
            None => {
                let error = Errors::format_new(
                    BadFormat::Received,
                    "Unable to retrieve interact reference",
                );
                error!("{}", error.log());
                return error.into_response();
            }
        };

        let payload = ApprovedCallbackBody { interact_ref, hash };
        match onboarder.continue_req(&id, payload).await {
            Ok(data) => (StatusCode::OK, Json(data)).into_response(),
            Err(e) => e.to_response(),
        }
    }

    async fn post_callback(
        State(onboarder): State<Arc<dyn CoreOnboarderTrait>>,
        Path(id): Path<String>,
        payload: Result<Json<CallbackBody>, JsonRejection>,
    ) -> impl IntoResponse {
        let payload = match payload {
            Ok(Json(data)) => data,
            Err(e) => return e.to_response(),
        };

        match payload {
            CallbackBody::Approved(data) => match onboarder.continue_req(&id, data).await {
                Ok(data) => (StatusCode::OK, Json(data)).into_response(),
                Err(e) => e.to_response(),
            },
            CallbackBody::Rejected(_) => match onboarder.manage_rejection(id).await {
                Ok(_) => (StatusCode::OK,).into_response(),
                Err(e) => e.to_response(),
            },
        }
    }
}
