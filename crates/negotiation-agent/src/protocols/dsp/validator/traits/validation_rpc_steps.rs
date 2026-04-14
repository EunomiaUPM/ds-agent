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

#![allow(unused)]

use crate::protocols::dsp::orchestrator::rpc::types::{
    RpcNegotiationAgreementMessageDto, RpcNegotiationEventAcceptedMessageDto,
    RpcNegotiationEventFinalizedMessageDto, RpcNegotiationOfferInitMessageDto,
    RpcNegotiationOfferMessageDto, RpcNegotiationRequestInitMessageDto,
    RpcNegotiationRequestMessageDto, RpcNegotiationTerminationMessageDto,
    RpcNegotiationVerificationMessageDto,
};
use ymir::errors::Outcome;

#[async_trait::async_trait]
pub trait ValidationRpcSteps: Send + Sync + 'static {
    async fn negotiation_request_init_rpc(
        &self,
        input: &RpcNegotiationRequestInitMessageDto,
    ) -> Outcome<()>;
    async fn negotiation_request_rpc(&self, input: &RpcNegotiationRequestMessageDto)
    -> Outcome<()>;
    async fn negotiation_offer_init_rpc(
        &self,
        input: &RpcNegotiationOfferInitMessageDto,
    ) -> Outcome<()>;
    async fn negotiation_offer_rpc(&self, input: &RpcNegotiationOfferMessageDto) -> Outcome<()>;
    async fn negotiation_agreement_rpc(
        &self,
        input: &RpcNegotiationAgreementMessageDto,
    ) -> Outcome<()>;
    async fn negotiation_agreement_verification_rpc(
        &self,
        input: &RpcNegotiationVerificationMessageDto,
    ) -> Outcome<()>;
    async fn negotiation_event_accepted_rpc(
        &self,
        input: &RpcNegotiationEventAcceptedMessageDto,
    ) -> Outcome<()>;
    async fn negotiation_event_finalized_rpc(
        &self,
        input: &RpcNegotiationEventFinalizedMessageDto,
    ) -> Outcome<()>;
    async fn negotiation_termination_rpc(
        &self,
        input: &RpcNegotiationTerminationMessageDto,
    ) -> Outcome<()>;
}
