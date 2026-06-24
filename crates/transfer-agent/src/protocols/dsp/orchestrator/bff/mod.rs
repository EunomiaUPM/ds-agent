pub(crate) mod bff;

use crate::protocols::dsp::context::DspTransferContext;
use crate::protocols::dsp::orchestrator::rpc::types::{
    RpcTransferMessageDto, RpcTransferRequestMessageDto,
};
use crate::protocols::dsp::protocol_types::{
    TransferProcessAckDto, TransferProcessMessageWrapper, TransferRequestMessageDto,
};
use ymir::errors::Outcome;

#[async_trait::async_trait]
pub trait BFFRPCOrchestratorTrait: Send + Sync + 'static {
    /// Consumer-side BFF: sends a TransferRequest with `autoStart = true` so the
    /// Provider automatically responds with a TransferStart message.
    async fn setup_transfer_request_bff_rpc(
        &self,
        input: &RpcTransferRequestMessageDto,
    ) -> Outcome<RpcTransferMessageDto<RpcTransferRequestMessageDto>>;

    /// Provider-side BFF: processes an inbound TransferRequest and immediately
    /// chains a TransferStart to the Consumer's callbackAddress.
    async fn on_transfer_request_auto_start(
        &self,
        ctx: DspTransferContext,
        input: &TransferProcessMessageWrapper<TransferRequestMessageDto>,
    ) -> Outcome<(TransferProcessMessageWrapper<TransferProcessAckDto>, bool)>;
}
