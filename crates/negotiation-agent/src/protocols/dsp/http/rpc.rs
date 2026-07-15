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

use crate::protocols::dsp::errors::extract_payload_error;
use crate::protocols::dsp::orchestrator::OrchestratorTrait;
use crate::protocols::dsp::orchestrator::rpc::types::{
    RpcNegotiationAgreementMessageDto, RpcNegotiationErrorDto,
    RpcNegotiationEventAcceptedMessageDto, RpcNegotiationEventFinalizedMessageDto,
    RpcNegotiationOfferInitMessageDto, RpcNegotiationOfferMessageDto,
    RpcNegotiationRequestInitMessageDto, RpcNegotiationRequestMessageDto,
    RpcNegotiationTerminationMessageDto, RpcNegotiationVerificationMessageDto,
};
use crate::protocols::dsp::protocol_types::{
    NegotiationErrorMessageDto, NegotiationProcessMessageType, NegotiationProcessMessageWrapper,
};
use axum::{
    Json, Router,
    extract::{FromRef, State, rejection::JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
};
use common::config::services::ContractsConfig;
use common::config::services::traits::ContractsConfigTrait;
use common::dsp_common::context_field::ContextField;
use common::dsp_common::odrl::{
    ContractRequestMessageOfferOfferId, ContractRequestMessageOfferTypes,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;
use ymir::errors::Errors;

#[derive(Clone)]
pub struct RpcRouter {
    orchestrator: Arc<dyn OrchestratorTrait>,
    config: Arc<ContractsConfig>,
}

impl FromRef<RpcRouter> for Arc<dyn OrchestratorTrait> {
    fn from_ref(state: &RpcRouter) -> Self {
        state.orchestrator.clone()
    }
}

impl FromRef<RpcRouter> for Arc<ContractsConfig> {
    fn from_ref(state: &RpcRouter) -> Self {
        state.config.clone()
    }
}

impl RpcRouter {
    pub fn new(service: Arc<dyn OrchestratorTrait>, config: Arc<ContractsConfig>) -> Self {
        Self {
            orchestrator: service,
            config,
        }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route(
                "/rpc/setup-request-init",
                post(Self::negotiation_request_init_rpc),
            )
            .route("/rpc/setup-request", post(Self::negotiation_request_rpc))
            .route(
                "/rpc/setup-offer-init",
                post(Self::negotiation_offer_init_rpc),
            )
            .route("/rpc/setup-offer", post(Self::negotiation_offer_rpc))
            .route(
                "/rpc/setup-acceptance",
                post(Self::negotiation_event_accepted_rpc),
            )
            .route(
                "/rpc/setup-agreement",
                post(Self::negotiation_agreement_rpc),
            )
            .route(
                "/rpc/setup-verification",
                post(Self::negotiation_agreement_verification_rpc),
            )
            .route(
                "/rpc/setup-finalization",
                post(Self::negotiation_event_finalized_rpc),
            )
            .route(
                "/rpc/setup-termination",
                post(Self::negotiation_termination_rpc),
            )
            .route(
                "/tck/negotiations/requests",
                post(Self::tck_initiate_negotiation),
            )
            .with_state(self)
    }

    async fn process_request<T, R, F, Fut>(
        input: Result<Json<T>, JsonRejection>,
        success_code: StatusCode,
        action: F,
    ) -> impl IntoResponse
    where
        T: Send + Serialize + Clone + 'static,
        R: Serialize,
        F: FnOnce(T) -> Fut,
        Fut: Future<Output = Result<R, Errors>> + Send,
    {
        let payload = match extract_payload_error(input) {
            Ok(v) => v,
            Err(e) => {
                let error_dto: NegotiationProcessMessageWrapper<NegotiationErrorMessageDto> =
                    e.into();
                return (StatusCode::BAD_REQUEST, Json(error_dto)).into_response();
            }
        };
        Self::map_service_result(action(payload.clone()).await, success_code, payload)
            .into_response()
    }

    fn map_service_result<R, T>(
        result: Result<R, Errors>,
        success_code: StatusCode,
        original_request: T,
    ) -> impl IntoResponse
    where
        R: Serialize,
        T: Serialize + Clone,
    {
        match result {
            Ok(data) => (success_code, Json(data)).into_response(),
            Err(err) => {
                let error_wrapper: NegotiationProcessMessageWrapper<NegotiationErrorMessageDto> =
                    err.into();
                let rpc_error_dto: RpcNegotiationErrorDto<T> = RpcNegotiationErrorDto {
                    request: original_request,
                    error: error_wrapper,
                };

                (StatusCode::BAD_REQUEST, Json(rpc_error_dto)).into_response()
            }
        }
    }

    async fn negotiation_request_init_rpc(
        State(state): State<RpcRouter>,
        input: Result<Json<RpcNegotiationRequestInitMessageDto>, JsonRejection>,
    ) -> impl IntoResponse {
        Self::process_request(input, StatusCode::CREATED, |data| async move {
            state
                .orchestrator
                .get_rpc_service()
                .setup_negotiation_request_init_rpc(&data)
                .await
        })
        .await
    }
    async fn negotiation_request_rpc(
        State(state): State<RpcRouter>,
        input: Result<Json<RpcNegotiationRequestMessageDto>, JsonRejection>,
    ) -> impl IntoResponse {
        Self::process_request(input, StatusCode::CREATED, |data| async move {
            state
                .orchestrator
                .get_rpc_service()
                .setup_negotiation_request_rpc(&data)
                .await
        })
        .await
    }
    async fn negotiation_offer_init_rpc(
        State(state): State<RpcRouter>,
        input: Result<Json<RpcNegotiationOfferInitMessageDto>, JsonRejection>,
    ) -> impl IntoResponse {
        Self::process_request(input, StatusCode::CREATED, |data| async move {
            state
                .orchestrator
                .get_rpc_service()
                .setup_negotiation_offer_init_rpc(&data)
                .await
        })
        .await
    }
    async fn negotiation_offer_rpc(
        State(state): State<RpcRouter>,
        input: Result<Json<RpcNegotiationOfferMessageDto>, JsonRejection>,
    ) -> impl IntoResponse {
        Self::process_request(input, StatusCode::CREATED, |data| async move {
            state
                .orchestrator
                .get_rpc_service()
                .setup_negotiation_offer_rpc(&data)
                .await
        })
        .await
    }
    async fn negotiation_event_accepted_rpc(
        State(state): State<RpcRouter>,
        input: Result<Json<RpcNegotiationEventAcceptedMessageDto>, JsonRejection>,
    ) -> impl IntoResponse {
        Self::process_request(input, StatusCode::CREATED, |data| async move {
            state
                .orchestrator
                .get_rpc_service()
                .setup_negotiation_event_accepted_rpc(&data)
                .await
        })
        .await
    }
    async fn negotiation_agreement_rpc(
        State(state): State<RpcRouter>,
        input: Result<Json<RpcNegotiationAgreementMessageDto>, JsonRejection>,
    ) -> impl IntoResponse {
        Self::process_request(input, StatusCode::CREATED, |data| async move {
            state
                .orchestrator
                .get_rpc_service()
                .setup_negotiation_agreement_rpc(&data)
                .await
        })
        .await
    }
    async fn negotiation_agreement_verification_rpc(
        State(state): State<RpcRouter>,
        input: Result<Json<RpcNegotiationVerificationMessageDto>, JsonRejection>,
    ) -> impl IntoResponse {
        Self::process_request(input, StatusCode::CREATED, |data| async move {
            state
                .orchestrator
                .get_rpc_service()
                .setup_negotiation_agreement_verification_rpc(&data)
                .await
        })
        .await
    }
    async fn negotiation_event_finalized_rpc(
        State(state): State<RpcRouter>,
        input: Result<Json<RpcNegotiationEventFinalizedMessageDto>, JsonRejection>,
    ) -> impl IntoResponse {
        Self::process_request(input, StatusCode::CREATED, |data| async move {
            state
                .orchestrator
                .get_rpc_service()
                .setup_negotiation_event_finalized_rpc(&data)
                .await
        })
        .await
    }
    async fn negotiation_termination_rpc(
        State(state): State<RpcRouter>,
        input: Result<Json<RpcNegotiationTerminationMessageDto>, JsonRejection>,
    ) -> impl IntoResponse {
        Self::process_request(input, StatusCode::CREATED, |data| async move {
            state
                .orchestrator
                .get_rpc_service()
                .setup_negotiation_termination_rpc(&data)
                .await
        })
        .await
    }

    // TCK bridge ────────────────────────────────────────────────────────────
    // Accepts the format the Eclipse DSP TCK POSTs to initiate a consumer-role
    // negotiation and translates it to the internal RPC call.
    async fn tck_initiate_negotiation(
        State(state): State<RpcRouter>,
        input: Result<Json<TckNegotiationInitiateRequest>, JsonRejection>,
    ) -> Response {
        let input = match extract_payload_error(input) {
            Ok(v) => v,
            Err(e) => {
                let error_dto: NegotiationProcessMessageWrapper<NegotiationErrorMessageDto> =
                    e.into();
                return (StatusCode::BAD_REQUEST, Json(error_dto)).into_response();
            }
        };
        let offer_id_urn = match Urn::from_str(&input.offer_id) {
            Ok(u) => u,
            Err(_) => {
                return (StatusCode::BAD_REQUEST, "invalid offerId: must be a URN").into_response();
            }
        };
        let callback_base = state.config.ssi_auth().hosts.get_host(HostType::Http);
        let callback_address = format!("{}/dsp/current/negotiations", callback_base);
        let rpc_dto = RpcNegotiationRequestInitMessageDto::new(
            input.provider_id,
            input.connector_address,
            callback_address,
            ContractRequestMessageOfferTypes::OfferId(ContractRequestMessageOfferOfferId {
                id: offer_id_urn,
            }),
        );
        Self::process_request(Ok(Json(rpc_dto)), StatusCode::CREATED, |data| async move {
            state
                .orchestrator
                .get_rpc_service()
                .setup_negotiation_request_init_rpc(&data)
                .await
        })
        .await
        .into_response()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TckNegotiationInitiateRequest {
    pub dataset_id: String,
    pub offer_id: String,
    pub provider_id: String,
    pub connector_address: String,
}
