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

pub(crate) mod bff;

use crate::protocols::dsp::orchestrator::rpc::types::{
    RpcNegotiationAgreementMessageDto, RpcNegotiationEventAcceptedMessageDto,
    RpcNegotiationEventFinalizedMessageDto, RpcNegotiationMessageDto,
    RpcNegotiationOfferInitMessageDto, RpcNegotiationOfferMessageDto,
    RpcNegotiationRequestInitMessageDto, RpcNegotiationRequestMessageDto,
    RpcNegotiationTerminationMessageDto, RpcNegotiationVerificationMessageDto,
};
use ymir::errors::Outcome;

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
