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

use crate::services::config::ConfigStore;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use ymir::types::oauth::UserInfo;
use common::config::ApplicationConfig;
use common::eunomia::document::JsonLd;
use ymir::errors::AppResult;
use common::routes::keystore as routes;

#[derive(Clone)]
pub struct ConfigRouter {
    service: Arc<dyn ConfigStore>,
}

impl ConfigRouter {
    pub fn new(service: Arc<dyn ConfigStore>) -> Self {
        Self { service }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route(routes::config::ROOT, get(Self::get_application_config))
            .with_state(self)
    }

    async fn get_application_config(
        State(state): State<ConfigRouter>,
        user: UserInfo,
    ) -> AppResult<Json<JsonLd<ApplicationConfig>>> {
        let items = state.service.get_application_config(&user).await?;
        Ok(Json(JsonLd(items)))
    }
}
