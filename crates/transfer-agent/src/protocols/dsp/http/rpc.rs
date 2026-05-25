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

use axum::{
    extract::{rejection::JsonRejection, FromRef, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde::Serialize;
use std::sync::Arc;

use crate::protocols::dsp::context::DspTransferContext;
use crate::protocols::dsp::orchestrator::rpc::types::{
    RpcTransferCompletionMessageDto, RpcTransferErrorDto, RpcTransferRequestMessageDto,
    RpcTransferStartMessageDto, RpcTransferSuspensionMessageDto, RpcTransferTerminationMessageDto,
};
use crate::protocols::dsp::orchestrator::OrchestratorTrait;
use crate::protocols::dsp::protocol_types::{
    TransferErrorDto, TransferProcessMessageType, TransferProcessMessageWrapper,
};
use common::dsp_common::context_field::ContextField;
use serde::Deserialize;
use std::str::FromStr;
use urn::Urn;
use ymir::errors::Outcome;
use ymir::utils::extract_payload;

#[derive(Clone)]
pub struct RpcRouter {
    orchestrator: Arc<dyn OrchestratorTrait>,
}

impl FromRef<RpcRouter> for Arc<dyn OrchestratorTrait> {
    fn from_ref(state: &RpcRouter) -> Self {
        state.orchestrator.clone()
    }
}

impl RpcRouter {
    pub fn new(service: Arc<dyn OrchestratorTrait>) -> Self {
        Self {
            orchestrator: service,
        }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route(
                "/rpc/setup-request",
                post(Self::handle_transfer_request_rpc),
            )
            .route("/rpc/setup-start", post(Self::handle_transfer_start_rpc))
            .route(
                "/rpc/setup-completion",
                post(Self::handle_transfer_completion_rpc),
            )
            .route(
                "/rpc/setup-termination",
                post(Self::handle_transfer_termination_rpc),
            )
            .route(
                "/rpc/setup-suspension",
                post(Self::handle_transfer_suspension_rpc),
            )
            .route("/tck/transfers/requests", post(Self::tck_initiate_transfer))
            .with_state(self)
    }

    fn map_service_result<R, T>(
        result: Outcome<R>,
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
                let error_wrapper: TransferProcessMessageWrapper<TransferErrorDto> =
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
                let rpc_error_dto: RpcTransferErrorDto<T> = RpcTransferErrorDto {
                    request: original_request,
                    error: error_wrapper,
                };
                (StatusCode::BAD_REQUEST, Json(rpc_error_dto)).into_response()
            }
        }
    }

    async fn handle_transfer_request_rpc(
        State(state): State<RpcRouter>,
        input: Result<Json<RpcTransferRequestMessageDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let data = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        let ctx = DspTransferContext::outbound_request(
            data.associated_agent_peer.clone(),
            data.provider_address.clone(),
            data.data_address.clone(),
        );
        Self::map_service_result(
            state
                .orchestrator
                .get_rpc_service()
                .setup_transfer_request(ctx, &data)
                .await,
            StatusCode::CREATED,
            data,
        )
        .into_response()
    }

    async fn handle_transfer_start_rpc(
        State(state): State<RpcRouter>,
        input: Result<Json<RpcTransferStartMessageDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let data = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        let ctx = DspTransferContext::outbound_continuation();
        Self::map_service_result(
            state
                .orchestrator
                .get_rpc_service()
                .setup_transfer_start(ctx, &data)
                .await,
            StatusCode::ACCEPTED,
            data,
        )
        .into_response()
    }

    async fn handle_transfer_completion_rpc(
        State(state): State<RpcRouter>,
        input: Result<Json<RpcTransferCompletionMessageDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let data = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        let ctx = DspTransferContext::outbound_continuation();
        Self::map_service_result(
            state
                .orchestrator
                .get_rpc_service()
                .setup_transfer_completion(ctx, &data)
                .await,
            StatusCode::ACCEPTED,
            data,
        )
        .into_response()
    }

    async fn handle_transfer_termination_rpc(
        State(state): State<RpcRouter>,
        input: Result<Json<RpcTransferTerminationMessageDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let data = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        let ctx = DspTransferContext::outbound_continuation();
        Self::map_service_result(
            state
                .orchestrator
                .get_rpc_service()
                .setup_transfer_termination(ctx, &data)
                .await,
            StatusCode::ACCEPTED,
            data,
        )
        .into_response()
    }

    async fn handle_transfer_suspension_rpc(
        State(state): State<RpcRouter>,
        input: Result<Json<RpcTransferSuspensionMessageDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let data = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        let ctx = DspTransferContext::outbound_continuation();
        Self::map_service_result(
            state
                .orchestrator
                .get_rpc_service()
                .setup_transfer_suspension(ctx, &data)
                .await,
            StatusCode::ACCEPTED,
            data,
        )
        .into_response()
    }

    async fn tck_initiate_transfer(
        State(state): State<RpcRouter>,
        input: Result<Json<TckTransferInitiateRequest>, JsonRejection>,
    ) -> Response {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        let agreement_id_urn = match Urn::from_str(&input.agreement_id) {
            Ok(u) => u,
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    "invalid agreementId: must be a URN",
                )
                    .into_response();
            }
        };
        let callback_base = "http://localhost:5000/"; // TODO change here
        let callback_address = format!("{}/dsp/current/transfers", callback_base);
        let rpc_dto = RpcTransferRequestMessageDto {
            associated_agent_peer: input.provider_id,
            agreement_id: agreement_id_urn,
            format: input.format,
            data_address: None,
            provider_address: input.connector_address,
            callback_address,
            auto_start: None,
        };
        let ctx = DspTransferContext::outbound_request(
            rpc_dto.associated_agent_peer.clone(),
            rpc_dto.provider_address.clone(),
            rpc_dto.data_address.clone(),
        );
        Self::map_service_result(
            state
                .orchestrator
                .get_rpc_service()
                .setup_transfer_request(ctx, &rpc_dto)
                .await,
            StatusCode::CREATED,
            rpc_dto,
        )
        .into_response()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TckTransferInitiateRequest {
    pub agreement_id: String,
    pub format: String,
    pub provider_id: String,
    pub connector_address: String,
}
