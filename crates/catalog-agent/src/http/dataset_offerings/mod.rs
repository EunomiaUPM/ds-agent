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

use axum::extract::rejection::JsonRejection;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};
use common::auth::AccessScope;
use ymir::utils::extract_payload;

use crate::entities::dataset_offerings::NewDatasetOfferingDto;
use crate::http::common::to_camel_case::ToCamelCase;
use crate::services::dataset_offerings::DatasetOfferingServiceTrait;

#[derive(Clone)]
pub(crate) struct DatasetOfferingRouter {
    service: Arc<dyn DatasetOfferingServiceTrait>,
}

impl DatasetOfferingRouter {
    pub(crate) fn new(service: Arc<dyn DatasetOfferingServiceTrait>) -> Self {
        Self { service }
    }

    pub(crate) fn router(self) -> Router {
        Router::new()
            .route("/", post(Self::handle_create))
            .with_state(self)
    }

    async fn handle_create(
        State(state): State<Self>,
        scope: AccessScope,
        input: Result<Json<NewDatasetOfferingDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let input = match extract_payload(input) {
            Ok(input) => input,
            Err(e) => return e.into_response(),
        };
        match state.service.create_offering(&scope, &input).await {
            Ok(offering) => (StatusCode::CREATED, Json(ToCamelCase(offering))).into_response(),
            Err(e) => e.into_response(),
        }
    }
}
