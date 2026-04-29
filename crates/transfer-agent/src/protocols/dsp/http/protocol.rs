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

use crate::protocols::dsp::context::DspTransferContext;
use crate::protocols::dsp::orchestrator::OrchestratorTrait;
use crate::protocols::dsp::protocol_types::{
    TransferCompletionMessageDto, TransferErrorDto, TransferProcessMessageType,
    TransferProcessMessageWrapper, TransferRequestMessageDto, TransferStartMessageDto,
    TransferSuspensionMessageDto, TransferTerminationMessageDto,
};
use axum::{
    extract::{rejection::JsonRejection, FromRef, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use common::dsp_common::context_field::ContextField;
use common::dsp_common::normalizer::dsp_namespace_normalizer;
use serde::Serialize;
use std::sync::Arc;

use axum::{
    extract::Request,
    middleware::{self, Next},
    Extension,
};
use common::config::services::TransferConfig;
use common::facades::ssi_auth_facade::SSIAuthFacadeTrait;
use common::facades::Mates;
use ymir::errors::Errors;
use ymir::utils::{extract_path_urn, extract_payload};

#[derive(Clone)]
pub struct DspRouter {
    orchestrator: Arc<dyn OrchestratorTrait>,
    #[allow(dead_code)]
    config: Arc<TransferConfig>,
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
        config: Arc<TransferConfig>,
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
            .route("/request", post(Self::handle_transfer_request))
            .route("/{id}", get(Self::handle_get_transfer_process))
            .route("/{id}/start", post(Self::handle_transfer_start))
            .route("/{id}/completion", post(Self::handle_transfer_completion))
            .route("/{id}/termination", post(Self::handle_transfer_termination))
            .route("/{id}/suspension", post(Self::handle_transfer_suspension))
            .layer(middleware::from_fn_with_state(
                self.clone(),
                Self::auth_middleware,
            ))
            .layer(middleware::from_fn(dsp_namespace_normalizer))
            .with_state(self)
    }

    fn map_service_result<R>(
        result: ymir::errors::Outcome<R>,
        success_code: StatusCode,
    ) -> impl IntoResponse
    where
        R: Serialize,
    {
        match result {
            Ok(data) => (success_code, Json(data)).into_response(),
            Err(err) => Self::map_service_error(err).into_response(),
        }
    }

    fn map_service_error(err: Errors) -> impl IntoResponse {
        let error_dto: TransferProcessMessageWrapper<TransferErrorDto> =
            TransferProcessMessageWrapper {
                context: ContextField::default(),
                _type: TransferProcessMessageType::TransferError,
                dto: TransferErrorDto {
                    consumer_pid: None,
                    provider_pid: None,
                    code: Some("5000".to_string()),
                    reason: Some(vec![
                        err.to_string(),
                        err.reason().to_string(),
                        err.path().to_string(),
                        err.context(),
                    ]),
                },
            };
        (StatusCode::BAD_REQUEST, Json(error_dto)).into_response()
    }

    async fn handle_get_transfer_process(
        State(state): State<DspRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        Self::map_service_result(
            state
                .orchestrator
                .get_protocol_service()
                .on_get_transfer_process(&id)
                .await,
            StatusCode::OK,
        )
    }

    async fn handle_transfer_request(
        State(state): State<DspRouter>,
        Extension(mate): Extension<Mates>,
        input: Result<
            Json<TransferProcessMessageWrapper<TransferRequestMessageDto>>,
            JsonRejection,
        >,
    ) -> impl IntoResponse {
        let data = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        let ctx = DspTransferContext::inbound(None, mate.participant_id.clone(), mate);
        let result = state
            .orchestrator
            .get_protocol_service()
            .on_transfer_request(ctx, &data)
            .await;

        match result {
            Ok((data, already_exists)) => {
                let status = if already_exists {
                    StatusCode::OK
                } else {
                    StatusCode::CREATED
                };
                (status, Json(data)).into_response()
            }
            Err(err) => Self::map_service_error(err).into_response(),
        }
    }

    async fn handle_transfer_start(
        State(state): State<DspRouter>,
        Path(id): Path<String>,
        Extension(mate): Extension<Mates>,
        input: Result<Json<TransferProcessMessageWrapper<TransferStartMessageDto>>, JsonRejection>,
    ) -> impl IntoResponse {
        let data = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        let process_id = match extract_path_urn(&id) {
            Ok(u) => u,
            Err(e) => return e.into_response(),
        };
        let ctx = DspTransferContext::inbound(Some(process_id), mate.participant_id.clone(), mate);
        Self::map_service_result(
            state
                .orchestrator
                .get_protocol_service()
                .on_transfer_start(ctx, &data)
                .await,
            StatusCode::OK,
        )
        .into_response()
    }

    async fn handle_transfer_completion(
        State(state): State<DspRouter>,
        Path(id): Path<String>,
        Extension(mate): Extension<Mates>,
        input: Result<
            Json<TransferProcessMessageWrapper<TransferCompletionMessageDto>>,
            JsonRejection,
        >,
    ) -> impl IntoResponse {
        let data = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        let process_id = match extract_path_urn(&id) {
            Ok(u) => u,
            Err(e) => return e.into_response(),
        };
        let ctx = DspTransferContext::inbound(Some(process_id), mate.participant_id.clone(), mate);
        Self::map_service_result(
            state
                .orchestrator
                .get_protocol_service()
                .on_transfer_completion(ctx, &data)
                .await,
            StatusCode::OK,
        )
        .into_response()
    }

    async fn handle_transfer_termination(
        State(state): State<DspRouter>,
        Path(id): Path<String>,
        Extension(mate): Extension<Mates>,
        input: Result<
            Json<TransferProcessMessageWrapper<TransferTerminationMessageDto>>,
            JsonRejection,
        >,
    ) -> impl IntoResponse {
        let data = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        let process_id = match extract_path_urn(&id) {
            Ok(u) => u,
            Err(e) => return e.into_response(),
        };
        let ctx = DspTransferContext::inbound(Some(process_id), mate.participant_id.clone(), mate);
        Self::map_service_result(
            state
                .orchestrator
                .get_protocol_service()
                .on_transfer_termination(ctx, &data)
                .await,
            StatusCode::OK,
        )
        .into_response()
    }

    async fn handle_transfer_suspension(
        State(state): State<DspRouter>,
        Path(id): Path<String>,
        Extension(mate): Extension<Mates>,
        input: Result<
            Json<TransferProcessMessageWrapper<TransferSuspensionMessageDto>>,
            JsonRejection,
        >,
    ) -> impl IntoResponse {
        let data = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        let process_id = match extract_path_urn(&id) {
            Ok(u) => u,
            Err(e) => return e.into_response(),
        };
        let ctx = DspTransferContext::inbound(Some(process_id), mate.participant_id.clone(), mate);
        Self::map_service_result(
            state
                .orchestrator
                .get_protocol_service()
                .on_transfer_suspension(ctx, &data)
                .await,
            StatusCode::OK,
        )
        .into_response()
    }
}
