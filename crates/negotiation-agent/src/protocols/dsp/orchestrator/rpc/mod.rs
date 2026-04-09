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
use crate::protocols::dsp::orchestrator::rpc::types::{
    RpcNegotiationAgreementMessageDto, RpcNegotiationEventAcceptedMessageDto,
    RpcNegotiationEventFinalizedMessageDto, RpcNegotiationMessageDto,
    RpcNegotiationOfferInitMessageDto, RpcNegotiationOfferMessageDto,
    RpcNegotiationRequestInitMessageDto, RpcNegotiationRequestMessageDto,
    RpcNegotiationTerminationMessageDto, RpcNegotiationVerificationMessageDto,
};
use ymir::errors::Outcome;

pub(crate) mod rpc;
pub(crate) mod types;

// ─── Step modules (Template Method pattern) ───────────────────────────────────
// Each step encodes one DSP negotiation lifecycle operation.  The orchestrator
// in `rpc.rs` dispatches through `run_lifecycle<S: NegotiationRpcStep>` so the
// algorithm (validate → prepare context → auth → send + persist) is written once.
pub(super) mod step_agreement;
pub(super) mod step_event_accepted;
pub(super) mod step_event_finalized;
pub(super) mod step_offer;
pub(super) mod step_offer_init;
pub(super) mod step_request;
pub(super) mod step_request_init;
pub(super) mod step_termination;
pub(super) mod step_trait;
pub(super) mod step_verification;

#[async_trait::async_trait]
pub trait RPCOrchestratorTrait: Send + Sync + 'static {
    async fn setup_negotiation_request_init_rpc(
        &self,
        input: &RpcNegotiationRequestInitMessageDto,
    ) -> Outcome<RpcNegotiationMessageDto<RpcNegotiationRequestInitMessageDto>>;
    async fn setup_negotiation_request_rpc(
        &self,
        input: &RpcNegotiationRequestMessageDto,
    ) -> Outcome<RpcNegotiationMessageDto<RpcNegotiationRequestMessageDto>>;
    async fn setup_negotiation_offer_init_rpc(
        &self,
        input: &RpcNegotiationOfferInitMessageDto,
    ) -> Outcome<RpcNegotiationMessageDto<RpcNegotiationOfferInitMessageDto>>;
    async fn setup_negotiation_offer_rpc(
        &self,
        input: &RpcNegotiationOfferMessageDto,
    ) -> Outcome<RpcNegotiationMessageDto<RpcNegotiationOfferMessageDto>>;
    async fn setup_negotiation_agreement_rpc(
        &self,
        input: &RpcNegotiationAgreementMessageDto,
    ) -> Outcome<RpcNegotiationMessageDto<RpcNegotiationAgreementMessageDto>>;
    async fn setup_negotiation_agreement_verification_rpc(
        &self,
        input: &RpcNegotiationVerificationMessageDto,
    ) -> Outcome<RpcNegotiationMessageDto<RpcNegotiationVerificationMessageDto>>;
    async fn setup_negotiation_event_accepted_rpc(
        &self,
        input: &RpcNegotiationEventAcceptedMessageDto,
    ) -> Outcome<RpcNegotiationMessageDto<RpcNegotiationEventAcceptedMessageDto>>;
    async fn setup_negotiation_event_finalized_rpc(
        &self,
        input: &RpcNegotiationEventFinalizedMessageDto,
    ) -> Outcome<RpcNegotiationMessageDto<RpcNegotiationEventFinalizedMessageDto>>;
    async fn setup_negotiation_termination_rpc(
        &self,
        input: &RpcNegotiationTerminationMessageDto,
    ) -> Outcome<RpcNegotiationMessageDto<RpcNegotiationTerminationMessageDto>>;
}
