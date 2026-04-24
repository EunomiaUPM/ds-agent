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
use std::future::Future;
use std::sync::Arc;

use crate::http::common::extract_payload;
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

    async fn process_request<T, R, F, Fut>(
        input: Result<Json<T>, JsonRejection>,
        success_code: StatusCode,
        action: F,
    ) -> impl IntoResponse
    where
        T: Send + Serialize + Clone + 'static,
        R: Serialize,
        F: FnOnce(T) -> Fut,
        Fut: Future<Output = Outcome<R>> + Send,
    {
        let payload = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e,
        };
        Self::map_service_result(action(payload.clone()).await, success_code, payload)
            .into_response()
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
                            reason: Some(vec![err.to_string()]),
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
        Self::process_request(input, StatusCode::CREATED, |data| async move {
            state
                .orchestrator
                .get_rpc_service()
                .setup_transfer_request(&data)
                .await
        })
        .await
    }

    async fn handle_transfer_start_rpc(
        State(state): State<RpcRouter>,
        input: Result<Json<RpcTransferStartMessageDto>, JsonRejection>,
    ) -> impl IntoResponse {
        Self::process_request(input, StatusCode::ACCEPTED, |data| async move {
            state
                .orchestrator
                .get_rpc_service()
                .setup_transfer_start(&data)
                .await
        })
        .await
    }

    async fn handle_transfer_completion_rpc(
        State(state): State<RpcRouter>,
        input: Result<Json<RpcTransferCompletionMessageDto>, JsonRejection>,
    ) -> impl IntoResponse {
        Self::process_request(input, StatusCode::ACCEPTED, |data| async move {
            state
                .orchestrator
                .get_rpc_service()
                .setup_transfer_completion(&data)
                .await
        })
        .await
    }

    async fn handle_transfer_termination_rpc(
        State(state): State<RpcRouter>,
        input: Result<Json<RpcTransferTerminationMessageDto>, JsonRejection>,
    ) -> impl IntoResponse {
        Self::process_request(input, StatusCode::ACCEPTED, |data| async move {
            state
                .orchestrator
                .get_rpc_service()
                .setup_transfer_termination(&data)
                .await
        })
        .await
    }

    async fn handle_transfer_suspension_rpc(
        State(state): State<RpcRouter>,
        input: Result<Json<RpcTransferSuspensionMessageDto>, JsonRejection>,
    ) -> impl IntoResponse {
        Self::process_request(input, StatusCode::ACCEPTED, |data| async move {
            state
                .orchestrator
                .get_rpc_service()
                .setup_transfer_suspension(&data)
                .await
        })
        .await
    }

    // ── TCK bridge ────────────────────────────────────────────────────────────
    // Accepts the format the Eclipse DSP TCK POSTs to initiate a consumer-role
    // transfer and translates it to the internal RPC call.
    async fn tck_initiate_transfer(
        State(state): State<RpcRouter>,
        input: Result<Json<TckTransferInitiateRequest>, JsonRejection>,
    ) -> Response {
        let input = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let agreement_id_urn = match Urn::from_str(&input.agreement_id) {
            Ok(u) => u,
            Err(_) => {
                return (StatusCode::BAD_REQUEST, "invalid agreementId: must be a URN")
                    .into_response();
            }
        };
        //let callback_base = state.config.ssi_auth().hosts.get_host(HostType::Http);
        let callback_base = "http://localhost:5000/"; // TODO change here
        let callback_address = format!("{}/dsp/current/transfers", callback_base);
        let rpc_dto = RpcTransferRequestMessageDto {
            associated_agent_peer: input.provider_id,
            agreement_id: agreement_id_urn,
            format: input.format,
            data_address: None,
            provider_address: input.connector_address,
            callback_address,
        };
        Self::process_request(Ok(Json(rpc_dto)), StatusCode::CREATED, |data| async move {
            state
                .orchestrator
                .get_rpc_service()
                .setup_transfer_request(&data)
                .await
        })
        .await
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
