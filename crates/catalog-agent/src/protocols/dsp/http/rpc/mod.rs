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

//! The catalog RPC (internal: users): eunomia bodies in, `RpcCatalogResult` out.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use common::rdf::RdfMessage;
use common::routes::catalog as routes;
use serde_json::Value;
use ymir::http::OauthHttpMiddleware;
use ymir::types::oauth::UserInfo;

use crate::protocols::dsp::entities::message_types::CatalogDspMessageType;
use crate::protocols::dsp::entities::rdf_extractor_rpc::EunomiaCatalog;
use crate::protocols::dsp::setup::context::DspContext;

#[derive(Clone)]
pub struct RpcRouter {
    ctx: Arc<DspContext>,
}

impl RpcRouter {
    pub fn new(ctx: Arc<DspContext>) -> Self {
        Self { ctx }
    }

    /// Every route behind the user's OAuth token.
    pub fn router(self) -> Router {
        let validator = self.ctx.app.oauth_validator.clone();
        Router::new()
            .route(routes::rpc::SETUP_CATALOG_REQUEST, post(Self::handle_catalog_request))
            .route(routes::rpc::SETUP_DATASET_REQUEST, post(Self::handle_dataset_request))
            .route_layer(axum::middleware::from_fn_with_state(validator, OauthHttpMiddleware::run))
            .with_state(self)
    }

    async fn handle_catalog_request(State(state): State<Self>, user: UserInfo, Json(body): Json<Value>) -> Response {
        let request =
            match RdfMessage::<EunomiaCatalog>::read_as(&CatalogDspMessageType::CatalogRequestMessage, body).await {
                Ok(request) => request,
                Err(e) => return e.into_response(),
            };
        match state.ctx.rpc.request_catalog(&user, &request).await {
            Ok(result) => (StatusCode::OK, Json(result)).into_response(),
            Err(e) => e.into_response(),
        }
    }

    async fn handle_dataset_request(State(state): State<Self>, user: UserInfo, Json(body): Json<Value>) -> Response {
        let request =
            match RdfMessage::<EunomiaCatalog>::read_as(&CatalogDspMessageType::DatasetRequestMessage, body).await {
                Ok(request) => request,
                Err(e) => return e.into_response(),
            };
        match state.ctx.rpc.request_dataset(&user, &request).await {
            Ok(result) => (StatusCode::OK, Json(result)).into_response(),
            Err(e) => e.into_response(),
        }
    }
}
