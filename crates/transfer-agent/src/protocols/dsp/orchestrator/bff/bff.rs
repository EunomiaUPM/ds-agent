use crate::protocols::dsp::context::DspTransferContext;
use crate::protocols::dsp::orchestrator::bff::BFFRPCOrchestratorTrait;
use crate::protocols::dsp::orchestrator::protocol::ProtocolOrchestratorTrait;
use crate::protocols::dsp::orchestrator::rpc::RPCOrchestratorTrait;
use crate::protocols::dsp::orchestrator::rpc::types::{
    RpcTransferMessageDto, RpcTransferRequestMessageDto, RpcTransferStartMessageDto,
};
use crate::protocols::dsp::protocol_types::{
    TransferProcessAckDto, TransferProcessMessageWrapper, TransferRequestMessageDto,
};
use std::sync::Arc;
use std::time::Duration;
use ymir::errors::Outcome;

pub struct BFFRPCOrchestratorService {
    protocol_service: Arc<dyn ProtocolOrchestratorTrait>,
    rpc_service: Arc<dyn RPCOrchestratorTrait>,
}

impl BFFRPCOrchestratorService {
    pub fn new(
        protocol_service: Arc<dyn ProtocolOrchestratorTrait>,
        rpc_service: Arc<dyn RPCOrchestratorTrait>,
    ) -> Self {
        Self {
            protocol_service,
            rpc_service,
        }
    }
}

#[async_trait::async_trait]
impl BFFRPCOrchestratorTrait for BFFRPCOrchestratorService {
    // Consumer → sends REQUEST with autoStart=true baked in; Provider auto-starts.
    async fn setup_transfer_request_bff_rpc(
        &self,
        input: &RpcTransferRequestMessageDto,
    ) -> Outcome<RpcTransferMessageDto<RpcTransferRequestMessageDto>> {
        let mut bff_input = input.clone();
        bff_input.auto_start = Some(true);
        let ctx = DspTransferContext::outbound_request(
            bff_input.associated_agent_peer.clone(),
            bff_input.provider_address.clone(),
            bff_input.data_address.clone(),
        );
        self.rpc_service.setup_transfer_request(ctx, &bff_input).await
    }

    // Provider → processes REQUEST then schedules a TransferStart once the ack
    // has reached the Consumer (i.e. the Consumer has persisted its own record).
    async fn on_transfer_request_auto_start(
        &self,
        ctx: DspTransferContext,
        input: &TransferProcessMessageWrapper<TransferRequestMessageDto>,
    ) -> Outcome<(TransferProcessMessageWrapper<TransferProcessAckDto>, bool)> {
        let (ack, already_exists) = self
            .protocol_service
            .on_transfer_request(ctx, input)
            .await?;

        if !already_exists {
            let start_input = RpcTransferStartMessageDto {
                consumer_pid: input.dto.consumer_pid.clone(),
                provider_pid: ack.dto.provider_pid.clone(),
            };
            let rpc_service = self.rpc_service.clone();

            // polls consumer to persist own transfer process in different thread
            tokio::spawn(async move {
                const MAX_ATTEMPTS: u32 = 8;
                // incremental delaying
                let delays_ms: [u64; 8] = [400, 400, 800, 1600, 1600, 1600, 1600, 1600];

                for attempt in 0..MAX_ATTEMPTS {
                    tokio::time::sleep(Duration::from_millis(delays_ms[attempt as usize])).await;
                    let start_ctx = DspTransferContext::outbound_continuation();
                    match rpc_service.setup_transfer_start(start_ctx, &start_input).await {
                        Ok(_) => {
                            tracing::info!(
                                "BFF auto-start: TransferStart sent (attempt {})",
                                attempt + 1
                            );
                            return;
                        }
                        Err(e) => {
                            tracing::warn!(
                                "BFF auto-start: attempt {} failed: {}",
                                attempt + 1,
                                e
                            );
                        }
                    }
                }
                tracing::error!(
                    "BFF auto-start: giving up after {} attempts",
                    MAX_ATTEMPTS
                );
            });
        }

        Ok((ack, already_exists))
    }
}
