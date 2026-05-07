pub(crate) mod bff;

use ymir::errors::Outcome;
use crate::protocols::dsp::orchestrator::rpc::types::{RpcNegotiationAgreementMessageDto, RpcNegotiationEventAcceptedMessageDto, RpcNegotiationEventFinalizedMessageDto, RpcNegotiationMessageDto, RpcNegotiationOfferInitMessageDto, RpcNegotiationOfferMessageDto, RpcNegotiationRequestInitMessageDto, RpcNegotiationRequestMessageDto, RpcNegotiationTerminationMessageDto, RpcNegotiationVerificationMessageDto};

#[async_trait::async_trait]
pub trait BFFRPCOrchestratorTrait: Send + Sync + 'static {
    async fn setup_negotiation_request_init_bff_rpc(
        &self,
        input: &RpcNegotiationRequestInitMessageDto,
    ) -> Outcome<RpcNegotiationMessageDto<RpcNegotiationRequestInitMessageDto>>;
    async fn setup_negotiation_offer_init_bff_rpc(
        &self,
        input: &RpcNegotiationOfferInitMessageDto,
    ) -> Outcome<RpcNegotiationMessageDto<RpcNegotiationOfferInitMessageDto>>;
    async fn setup_negotiation_agreement_bff_rpc(
        &self,
        input: &RpcNegotiationAgreementMessageDto,
    ) -> Outcome<RpcNegotiationMessageDto<RpcNegotiationEventFinalizedMessageDto>>;
    async fn setup_negotiation_event_accepted_bff_rpc(
        &self,
        input: &RpcNegotiationEventAcceptedMessageDto,
    ) -> Outcome<RpcNegotiationMessageDto<RpcNegotiationEventAcceptedMessageDto>>;
    async fn setup_negotiation_termination_bff_rpc(
        &self,
        input: &RpcNegotiationTerminationMessageDto,
    ) -> Outcome<RpcNegotiationMessageDto<RpcNegotiationTerminationMessageDto>>;
}
