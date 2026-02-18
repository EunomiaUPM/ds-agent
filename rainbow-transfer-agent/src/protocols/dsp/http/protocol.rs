/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use axum::{
    extract::{rejection::JsonRejection, FromRef, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use std::future::Future;
use std::sync::Arc;

use crate::protocols::dsp::errors::extract_payload_error;
use crate::protocols::dsp::orchestrator::OrchestratorTrait;
use crate::protocols::dsp::protocol_types::{
    TransferCompletionMessageDto, TransferErrorDto, TransferProcessMessageType,
    TransferProcessMessageWrapper, TransferRequestMessageDto, TransferStartMessageDto,
    TransferSuspensionMessageDto, TransferTerminationMessageDto,
};
use rainbow_common::dsp_common::context_field::ContextField;
use rainbow_common::errors::CommonErrors;

use axum::{
    extract::Request,
    http::HeaderMap,
    middleware::{self, Next},
    Extension,
};
use rainbow_common::config::services::TransferConfig;
use rainbow_common::facades::ssi_auth_facade::SSIAuthFacadeTrait;
use rainbow_common::mates::mates::Mates;

#[derive(Clone)]
pub struct DspRouter {
    orchestrator: Arc<dyn OrchestratorTrait>,
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
        Self { orchestrator: service, config, ssi_auth }
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
            .layer(middleware::from_fn_with_state(self.clone(), Self::auth_middleware))
            .with_state(self)
    }

    async fn process_request<T, R, F, Fut>(
        input: Result<Json<T>, JsonRejection>,
        success_code: StatusCode,
        action: F,
    ) -> impl IntoResponse
    where
        T: Send,
        R: Serialize,
        F: FnOnce(T) -> Fut,
        Fut: Future<Output = anyhow::Result<R>> + Send,
    {
        let payload = match extract_payload_error(input) {
            Ok(v) => v,
            Err(e) => {
                let error_dto: TransferProcessMessageWrapper<TransferErrorDto> = e.into();
                return (StatusCode::BAD_REQUEST, Json(error_dto)).into_response();
            }
        };
        Self::map_service_result(action(payload).await, success_code).into_response()
    }

    fn map_service_result<R>(
        result: anyhow::Result<R>,
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

    fn map_service_error(err: anyhow::Error) -> impl IntoResponse {
        match err.downcast::<CommonErrors>() {
            Ok(common_errors) => {
                let error_dto: TransferProcessMessageWrapper<TransferErrorDto> =
                    common_errors.into();
                (StatusCode::BAD_REQUEST, Json(error_dto)).into_response()
            }
            Err(original_err) => {
                let error_dto: TransferProcessMessageWrapper<TransferErrorDto> =
                    TransferProcessMessageWrapper {
                        context: ContextField::default(),
                        _type: TransferProcessMessageType::TransferError,
                        dto: TransferErrorDto {
                            consumer_pid: None,
                            provider_pid: None,
                            code: Some("5000".to_string()),
                            reason: Some(vec![original_err.to_string()]),
                        },
                    };
                (StatusCode::BAD_REQUEST, Json(error_dto)).into_response()
            }
        }
    }

    async fn handle_get_transfer_process(
        State(state): State<DspRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        Self::map_service_result(
            state.orchestrator.get_protocol_service().on_get_transfer_process(&id).await,
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
        let payload = match extract_payload_error(input) {
            Ok(v) => v,
            Err(e) => {
                let error_dto: TransferProcessMessageWrapper<TransferErrorDto> = e.into();
                return (StatusCode::BAD_REQUEST, Json(error_dto)).into_response();
            }
        };

        let result = state
            .orchestrator
            .get_protocol_service()
            .on_transfer_request(&payload, &mate.participant_id)
            .await;

        match result {
            Ok((data, already_exists)) => {
                let status = if already_exists { StatusCode::OK } else { StatusCode::CREATED };
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
        Self::process_request(input, StatusCode::OK, |data| async move {
            state.orchestrator.get_protocol_service().on_transfer_start(&id, &data).await
        })
        .await
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
        Self::process_request(input, StatusCode::OK, |data| async move {
            state.orchestrator.get_protocol_service().on_transfer_completion(&id, &data).await
        })
        .await
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
        Self::process_request(input, StatusCode::OK, |data| async move {
            state.orchestrator.get_protocol_service().on_transfer_termination(&id, &data).await
        })
        .await
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
        Self::process_request(input, StatusCode::OK, |data| async move {
            state.orchestrator.get_protocol_service().on_transfer_suspension(&id, &data).await
        })
        .await
    }
}
