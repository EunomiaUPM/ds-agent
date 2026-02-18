/*
 *
 * * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 * *
 * * This program is free software: you can redistribute it and/or modify
 * * it under the terms of the GNU General Public License as published by
 * * the Free Software Foundation, either version 3 of the License, or
 * * (at your option) any later version.
 * *
 * * This program is distributed in the hope that it will be useful,
 * * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * * GNU General Public License for more details.
 * *
 * * You should have received a copy of the GNU General Public License
 * * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::protocols::dsp::errors::extract_payload_error;
use crate::protocols::dsp::orchestrator::OrchestratorTrait;
use crate::protocols::dsp::protocol_types::{
    NegotiationAgreementMessageDto, NegotiationErrorMessageDto, NegotiationEventMessageDto,
    NegotiationOfferInitMessageDto, NegotiationOfferMessageDto, NegotiationProcessMessageType,
    NegotiationProcessMessageWrapper, NegotiationRequestInitMessageDto,
    NegotiationRequestMessageDto, NegotiationTerminationMessageDto,
    NegotiationVerificationMessageDto,
};
use axum::{
    Extension, Json, Router,
    extract::{FromRef, Path, Request, State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use rainbow_common::config::services::ContractsConfig;
use rainbow_common::dsp_common::context_field::ContextField;
use rainbow_common::errors::CommonErrors;
use rainbow_common::facades::ssi_auth_facade::SSIAuthFacadeTrait;
use rainbow_common::mates::mates::Mates;
use serde::Serialize;
use std::future::Future;
use std::sync::Arc;

#[derive(Clone)]
pub struct DspRouter {
    orchestrator: Arc<dyn OrchestratorTrait>,
    config: Arc<ContractsConfig>,
    ssi_auth: Arc<dyn SSIAuthFacadeTrait>,
}

impl FromRef<DspRouter> for Arc<dyn OrchestratorTrait> {
    fn from_ref(state: &DspRouter) -> Self {
        state.orchestrator.clone()
    }
}

impl FromRef<DspRouter> for Arc<ContractsConfig> {
    fn from_ref(state: &DspRouter) -> Self {
        state.config.clone()
    }
}

impl DspRouter {
    pub fn new(
        service: Arc<dyn OrchestratorTrait>,
        config: Arc<ContractsConfig>,
        ssi_auth: Arc<dyn SSIAuthFacadeTrait>,
    ) -> Self {
        Self { orchestrator: service, config, ssi_auth }
    }

    pub fn router(self) -> Router {
        Router::new()
            // =========================================================
            //  COMMON & PROVIDER ROUTES (DSP 8.2)
            // =========================================================
            // 8.2.1 & 8.3.2: Get Negotiation (Unified)
            .route("/{id}", get(Self::handle_get_negotiation))
            // 8.2.2: Contract Request Endpoint (Consumer initiates) -> ROLE: PROVIDER
            .route("/request", post(Self::handle_initial_request))
            // 8.2.3: Contract Request Endpoint (Consumer counters) -> ROLE: PROVIDER
            .route("/{id}/request", post(Self::handle_consumer_request))
            // 8.2.5: Agreement Verification (Consumer verifies) -> ROLE: PROVIDER
            .route(
                "/{id}/agreement/verification",
                post(Self::handle_agreement_verification),
            )
            // =========================================================
            //  CONSUMER CALLBACK ROUTES (DSP 8.3)
            // =========================================================
            // 8.3.3: Contract Offer Endpoint (Provider initiates) -> ROLE: CONSUMER
            .route("/offers", post(Self::handle_initial_offer))
            // 8.3.4: Contract Offer Endpoint (Provider counters) -> ROLE: CONSUMER
            .route("/{id}/offers", post(Self::handle_provider_offer))
            // 8.3.5: Contract Agreement Endpoint (Provider sends Agreement) -> ROLE: CONSUMER
            .route("/{id}/agreement", post(Self::handle_agreement_reception))
            // =========================================================
            //  SHARED / AMBIGUOUS ROUTES
            // =========================================================
            // 8.2.4 (Provider Endpoint) & 8.3.6 (Consumer Endpoint) -> Events
            // Both allow POST /events. The logic inside must discriminate based on state/role.
            .route("/{id}/events", post(Self::handle_negotiation_events))
            // 8.2.6 (Provider Endpoint) & 8.3.7 (Consumer Endpoint) -> Termination
            .route("/{id}/termination", post(Self::handle_negotiation_termination))
            .layer(middleware::from_fn_with_state(self.clone(), Self::auth_middleware))
            .with_state(self)
    }

    async fn auth_middleware(
        State(state): State<DspRouter>,
        headers: HeaderMap,
        mut request: Request,
        next: Next,
    ) -> Result<Response, StatusCode> {
        let auth_header = headers
            .get("Authorization")
            .and_then(|header| header.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        if !auth_header.starts_with("Bearer ") {
            return Err(StatusCode::UNAUTHORIZED);
        }

        let token = auth_header.trim_start_matches("Bearer ").to_string();

        match state.ssi_auth.verify_token(token).await {
            Ok(mate) => {
                request.extensions_mut().insert(mate);
                Ok(next.run(request).await)
            }
            Err(_) => Err(StatusCode::UNAUTHORIZED),
        }
    }

    // --- Helpers ---

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
                let error_dto: NegotiationProcessMessageWrapper<NegotiationErrorMessageDto> =
                    e.into();
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
                let error_dto: NegotiationProcessMessageWrapper<NegotiationErrorMessageDto> =
                    common_errors.into();
                (StatusCode::BAD_REQUEST, Json(error_dto)).into_response()
            }
            Err(original_err) => {
                let error_dto = NegotiationProcessMessageWrapper {
                    context: ContextField::default(),
                    _type: NegotiationProcessMessageType::NegotiationError,
                    dto: NegotiationErrorMessageDto {
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

    // --- Handlers ---

    // 1. GET (Unified)
    async fn handle_get_negotiation(
        State(state): State<DspRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        Self::map_service_result(
            state.orchestrator.get_protocol_service().on_get_negotiation(&id).await,
            StatusCode::OK,
        )
    }

    // 2. PROVIDER ROLE Handlers (Incoming requests from a Consumer)

    async fn handle_initial_request(
        State(state): State<DspRouter>,
        Extension(mate): Extension<Mates>,
        input: Result<
            Json<NegotiationProcessMessageWrapper<NegotiationRequestInitMessageDto>>,
            JsonRejection,
        >,
    ) -> impl IntoResponse {
        let payload = match extract_payload_error(input) {
            Ok(v) => v,
            Err(e) => {
                let error_dto: NegotiationProcessMessageWrapper<NegotiationErrorMessageDto> =
                    e.into();
                return (StatusCode::BAD_REQUEST, Json(error_dto)).into_response();
            }
        };

        let result = state
            .orchestrator
            .get_protocol_service()
            .on_initial_contract_request(&payload, &mate)
            .await;
        match result {
            Ok((data, exists)) => {
                let status = if exists { StatusCode::OK } else { StatusCode::CREATED };
                (status, Json(data)).into_response()
            }
            Err(err) => Self::map_service_error(err).into_response(),
        }
    }

    async fn handle_consumer_request(
        State(state): State<DspRouter>,
        Path(id): Path<String>,
        Extension(mate): Extension<Mates>,
        input: Result<
            Json<NegotiationProcessMessageWrapper<NegotiationRequestMessageDto>>,
            JsonRejection,
        >,
    ) -> impl IntoResponse {
        Self::process_request(input, StatusCode::OK, |data| async move {
            state.orchestrator.get_protocol_service().on_consumer_request(&id, &data, &mate).await
        })
        .await
    }

    async fn handle_agreement_verification(
        State(state): State<DspRouter>,
        Path(id): Path<String>,
        Extension(mate): Extension<Mates>,
        input: Result<
            Json<NegotiationProcessMessageWrapper<NegotiationVerificationMessageDto>>,
            JsonRejection,
        >,
    ) -> impl IntoResponse {
        Self::process_request(input, StatusCode::OK, |data| async move {
            state
                .orchestrator
                .get_protocol_service()
                .on_agreement_verification(&id, &data, &mate)
                .await
        })
        .await
    }

    // 3. CONSUMER ROLE Handlers (Incoming callbacks from a Provider)

    async fn handle_initial_offer(
        State(state): State<DspRouter>,
        Extension(mate): Extension<Mates>,
        input: Result<
            Json<NegotiationProcessMessageWrapper<NegotiationOfferInitMessageDto>>,
            JsonRejection,
        >,
    ) -> impl IntoResponse {
        let payload = match extract_payload_error(input) {
            Ok(v) => v,
            Err(e) => {
                let error_dto: NegotiationProcessMessageWrapper<NegotiationErrorMessageDto> =
                    e.into();
                return (StatusCode::BAD_REQUEST, Json(error_dto)).into_response();
            }
        };

        let result = state
            .orchestrator
            .get_protocol_service()
            .on_initial_provider_offer(&payload, &mate)
            .await;
        match result {
            Ok((data, exists)) => {
                let status = if exists { StatusCode::OK } else { StatusCode::CREATED };
                (status, Json(data)).into_response()
            }
            Err(err) => Self::map_service_error(err).into_response(),
        }
    }

    async fn handle_provider_offer(
        State(state): State<DspRouter>,
        Path(id): Path<String>,
        Extension(mate): Extension<Mates>,
        input: Result<
            Json<NegotiationProcessMessageWrapper<NegotiationOfferMessageDto>>,
            JsonRejection,
        >,
    ) -> impl IntoResponse {
        Self::process_request(input, StatusCode::OK, |data| async move {
            state.orchestrator.get_protocol_service().on_provider_offer(&id, &data, &mate).await
        })
        .await
    }

    async fn handle_agreement_reception(
        State(state): State<DspRouter>,
        Path(id): Path<String>,
        Extension(mate): Extension<Mates>,
        input: Result<
            Json<NegotiationProcessMessageWrapper<NegotiationAgreementMessageDto>>,
            JsonRejection,
        >,
    ) -> impl IntoResponse {
        Self::process_request(input, StatusCode::OK, |data| async move {
            state
                .orchestrator
                .get_protocol_service()
                .on_agreement_reception(&id, &data, &mate)
                .await
        })
        .await
    }

    // 4. SHARED Handlers

    async fn handle_negotiation_events(
        State(state): State<DspRouter>,
        Path(id): Path<String>,
        Extension(mate): Extension<Mates>,
        input: Result<
            Json<NegotiationProcessMessageWrapper<NegotiationEventMessageDto>>,
            JsonRejection,
        >,
    ) -> impl IntoResponse {
        Self::process_request(input, StatusCode::OK, |data| async move {
            state.orchestrator.get_protocol_service().on_negotiation_event(&id, &data, &mate).await
        })
        .await
    }

    async fn handle_negotiation_termination(
        State(state): State<DspRouter>,
        Path(id): Path<String>,
        Extension(mate): Extension<Mates>,
        input: Result<
            Json<NegotiationProcessMessageWrapper<NegotiationTerminationMessageDto>>,
            JsonRejection,
        >,
    ) -> impl IntoResponse {
        Self::process_request(input, StatusCode::OK, |data| async move {
            state
                .orchestrator
                .get_protocol_service()
                .on_negotiation_termination(&id, &data, &mate)
                .await
        })
        .await
    }
}
