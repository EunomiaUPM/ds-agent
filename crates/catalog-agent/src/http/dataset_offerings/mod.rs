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

//! Publishing a dataset with its distribution and policy in one call.

use std::sync::Arc;

use axum::extract::rejection::JsonRejection;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use common::routes::catalog as routes;
use serde_json::json;
use ymir::types::oauth::UserInfo;
use ymir::utils::extract_payload;

use crate::entities::admin_document::AdminDocument;
use crate::entities::dataset_offering::NewDatasetOfferingCommand;
use crate::http::resources::headers::AdminHeaders;
use crate::services::odrl_policies::views::OdrlPolicyView;
use crate::setup::context::AppContext;

#[derive(Clone)]
pub struct DatasetOfferingRouter {
    ctx: Arc<AppContext>,
}

impl DatasetOfferingRouter {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route(routes::dataset_offerings::ROOT, post(Self::handle_create))
            .with_state(self)
    }

    /// The created dataset and distribution as catalog documents, and the policy.
    async fn handle_create(
        State(state): State<Self>,
        user: UserInfo,
        headers: HeaderMap,
        input: Result<Json<NewDatasetOfferingCommand>, JsonRejection>,
    ) -> Response {
        let input = match extract_payload(input) {
            Ok(input) => input,
            Err(e) => return e.into_response(),
        };
        let visibility = match AdminHeaders::visibility(&headers) {
            Ok(visibility) => visibility,
            Err(e) => return e.into_response(),
        };
        match state.ctx.offerings().create_offering(&user, &input, visibility).await {
            Ok(offering) => (
                StatusCode::CREATED,
                Json(json!({
                    "dataset": AdminDocument::render(&offering.dataset),
                    "distribution": AdminDocument::render(&offering.distribution),
                    "policy": offering.policy.map(OdrlPolicyView::assemble),
                })),
            )
                .into_response(),
            Err(e) => e.into_response(),
        }
    }
}
