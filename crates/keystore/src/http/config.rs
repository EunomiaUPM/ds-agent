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

use crate::entities::commands::{EditParameterCommand, NewParameterCommand};
use crate::entities::key::{Key, KeyPrefix};
use crate::services::config::ConfigStore;
use crate::services::parameters::ParameterStore;
use crate::services::parameters::views::{ParameterView, VersionResponse};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use common::config::ApplicationConfig;
use serde::Deserialize;
use ymir::errors::AppResult;

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
            .route("/", get(Self::get_application_config))
            .with_state(self)
    }

    async fn get_application_config(
        State(state): State<ConfigRouter>,
    ) -> AppResult<Json<ApplicationConfig>> {
        let items = state.service.get_application_config().await?;
        Ok(Json(items))
    }
}
