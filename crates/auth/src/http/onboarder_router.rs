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
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use ymir::data::entities::mates::Model;
use ymir::errors::AppResult;
use ymir::types::gnap::{ApprovedCallbackBody, CallbackBody};
use ymir::utils::{extract_payload, extract_query_param};

use crate::core::traits::CoreOnboarderTrait;
use crate::types::entities::ReachProvider;

pub struct OnboarderRouter {
    onboarder: Arc<dyn CoreOnboarderTrait>
}

impl OnboarderRouter {
    pub fn new(onboarder: Arc<dyn CoreOnboarderTrait>) -> Self { Self { onboarder } }

    pub fn router(self) -> Router {
        Router::new()
            .route("/provider", post(Self::onboard))
            .route("/callback/{id}", get(Self::get_callback))
            .route("/callback/{id}", post(Self::post_callback))
            .with_state(self.onboarder)
    }

    async fn onboard(
        State(onboarder): State<Arc<dyn CoreOnboarderTrait>>,
        payload: Result<Json<ReachProvider>, JsonRejection>
    ) -> AppResult {
        let payload = extract_payload(payload)?;
        Ok(match onboarder.onboard_req(payload).await {
            Ok(Some(data)) => data.into_response(),
            Ok(None) => ().into_response(),
            Err(e) => e.into_response()
        })
    }

    async fn get_callback(
        State(onboarder): State<Arc<dyn CoreOnboarderTrait>>,
        Path(id): Path<String>,
        Query(params): Query<HashMap<String, String>>
    ) -> AppResult<Json<Model>> {
        let hash = extract_query_param(&params, "hash")?;
        let interact_ref = extract_query_param(&params, "interact_ref")?;
        let payload = ApprovedCallbackBody { interact_ref, hash };
        Ok(Json(onboarder.continue_req(&id, payload).await?))
    }

    async fn post_callback(
        State(onboarder): State<Arc<dyn CoreOnboarderTrait>>,
        Path(id): Path<String>,
        payload: Result<Json<CallbackBody>, JsonRejection>
    ) -> AppResult {
        let payload = extract_payload(payload)?;
        Ok(match payload {
            CallbackBody::Approved(data) => {
                onboarder.continue_req(&id, data).await.map(Json).into_response()
            }
            CallbackBody::Rejected(_) => onboarder.manage_rejection(id).await.into_response()
        })
    }
}
