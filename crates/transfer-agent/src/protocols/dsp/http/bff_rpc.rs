use crate::protocols::dsp::orchestrator::OrchestratorTrait;
use crate::protocols::dsp::orchestrator::rpc::types::{
    RpcTransferErrorDto, RpcTransferRequestMessageDto,
};
use crate::protocols::dsp::protocol_types::{
    TransferErrorDto, TransferProcessMessageType, TransferProcessMessageWrapper,
};
use axum::{
    Json, Router,
    extract::{FromRef, State, rejection::JsonRejection},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
};
use common::dsp_common::context_field::ContextField;
use serde::Serialize;
use std::sync::Arc;
use ymir::errors::Outcome;
use ymir::utils::extract_payload;

#[derive(Clone)]
pub struct BffRpcRouter {
    orchestrator: Arc<dyn OrchestratorTrait>,
}

impl FromRef<BffRpcRouter> for Arc<dyn OrchestratorTrait> {
    fn from_ref(state: &BffRpcRouter) -> Self {
        state.orchestrator.clone()
    }
}

impl BffRpcRouter {
    pub fn new(orchestrator: Arc<dyn OrchestratorTrait>) -> Self {
        Self { orchestrator }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route(
                "/bff-rpc/setup-request",
                post(Self::handle_transfer_request_bff_rpc),
            )
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

    async fn handle_transfer_request_bff_rpc(
        State(state): State<BffRpcRouter>,
        input: Result<Json<RpcTransferRequestMessageDto>, JsonRejection>,
    ) -> impl IntoResponse {
        let data = match extract_payload(input) {
            Ok(v) => v,
            Err(e) => return e.into_response(),
        };
        Self::map_service_result(
            state
                .orchestrator
                .get_bff_rpc_service()
                .setup_transfer_request_bff_rpc(&data)
                .await,
            StatusCode::CREATED,
            data,
        )
        .into_response()
    }
}
