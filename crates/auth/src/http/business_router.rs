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
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use common::auth::business::RainbowBusinessLoginRequest;
use ymir::errors::AppResult;
use ymir::utils::extract_payload;

use crate::core::traits::CoreBusinessTrait;
use crate::types::business::BusinessResponse;

pub struct BusinessRouter {
    pub business: Arc<dyn CoreBusinessTrait>
}

impl BusinessRouter {
    pub fn new(business: Arc<dyn CoreBusinessTrait>) -> Self { BusinessRouter { business } }

    pub fn router(self) -> Router {
        Router::new()
            .route("/login", post(Self::login))
            .route("/token", post(Self::token))
            .with_state(self.business)
    }

    async fn login(
        State(business): State<Arc<dyn CoreBusinessTrait>>,
        payload: Result<Json<RainbowBusinessLoginRequest>, JsonRejection>
    ) -> AppResult<String> {
        let payload = extract_payload(payload)?;
        business.login(payload).await
    }
    async fn token(
        State(business): State<Arc<dyn CoreBusinessTrait>>,
        payload: Result<Json<RainbowBusinessLoginRequest>, JsonRejection>
    ) -> AppResult<Json<BusinessResponse>> {
        let payload = extract_payload(payload)?;
        Ok(Json(business.token(payload).await?))
    }
}
