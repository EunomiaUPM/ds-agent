/*
 *
 *  * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::protocols::dsp::orchestrator::OrchestratorTrait;
use crate::protocols::dsp::protocol_types::{
    CatalogMessageType, CatalogMessageWrapper, CatalogRequestMessageDto, DatasetRequestMessage,
};
use axum::{
    extract::{rejection::JsonRejection, FromRef, Path, Request, State},
    middleware::{self, Next},
    response::IntoResponse,
    routing::get,
    routing::post,
    Extension, Json, Router,
};
use common::config::services::CatalogConfig;
use common::dsp_common::context_field::ContextField;
use common::dsp_common::normalizer::dsp_namespace_normalizer;
use common::facades::ssi_auth_facade::SSIAuthFacadeTrait;
use common::facades::Mates;
use reqwest::StatusCode;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;

#[derive(Clone)]
pub struct DspRouter {
    orchestrator: Arc<dyn OrchestratorTrait>,
    config: Arc<CatalogConfig>,
    ssi_auth: Arc<dyn SSIAuthFacadeTrait>,
}

impl FromRef<DspRouter> for Arc<dyn OrchestratorTrait> {
    fn from_ref(state: &DspRouter) -> Self {
        state.orchestrator.clone()
    }
}

impl DspRouter {
    pub fn new(
        service: Arc<dyn OrchestratorTrait>,
        config: Arc<CatalogConfig>,
        ssi_auth: Arc<dyn SSIAuthFacadeTrait>,
    ) -> Self {
        Self {
            orchestrator: service,
            config,
            ssi_auth,
        }
    }

    async fn auth_middleware(
        State(state): State<DspRouter>,
        mut request: Request,
        next: Next,
    ) -> Result<impl IntoResponse, StatusCode> {
        let headers = request.headers();
        let auth_header = headers.get("Authorization");
        let token = match auth_header {
            Some(header) => header.to_str().unwrap_or("").to_string(),
            None => return Err(StatusCode::UNAUTHORIZED),
        };
        let token = token.replace("Bearer ", "");
        match state.ssi_auth.verify_token(token).await {
            Ok(mate) => {
                request.extensions_mut().insert(mate);
                Ok(next.run(request).await)
            }
            Err(_) => Err(StatusCode::UNAUTHORIZED),
        }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/request", post(Self::handle_catalog_request))
            .route("/datasets/{id}", get(Self::handle_dataset_request))
            .layer(middleware::from_fn_with_state(
                self.clone(),
                Self::auth_middleware,
            ))
            .layer(middleware::from_fn(dsp_namespace_normalizer))
            .with_state(self)
    }

    async fn handle_catalog_request(
        State(state): State<DspRouter>,
        Extension(_mate): Extension<Mates>,
        input: Result<Json<CatalogMessageWrapper<CatalogRequestMessageDto>>, JsonRejection>,
    ) -> impl IntoResponse {
        let input = match input {
            Ok(input) => input.0,
            Err(e) => return (StatusCode::BAD_REQUEST, e.body_text()).into_response(),
        };
        match state
            .orchestrator
            .get_protocol_service()
            .on_catalog_request(&input)
            .await
        {
            Ok(catalog) => (StatusCode::OK, Json(catalog)).into_response(),
            Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
        }
    }

    async fn handle_dataset_request(
        State(state): State<DspRouter>,
        Path(id): Path<String>,
        Extension(_mate): Extension<Mates>,
    ) -> impl IntoResponse {
        let dataset_id = match Urn::from_str(&id) {
            Ok(urn) => urn,
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    format!("Invalid dataset ID: {}", id),
                )
                    .into_response()
            }
        };
        let request_msg = CatalogMessageWrapper {
            context: ContextField::default(),
            _type: CatalogMessageType::DatasetRequestMessage,
            dto: DatasetRequestMessage {
                dataset: dataset_id,
            },
        };
        match state
            .orchestrator
            .get_protocol_service()
            .on_dataset_request(&request_msg)
            .await
        {
            Ok(dataset) => (StatusCode::OK, Json(dataset)).into_response(),
            Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
        }
    }
}
