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

use crate::entities::negotiation_process::NegotiationProcessDto;
use crate::protocols::dsp::orchestrator::bff::BFFRPCOrchestratorTrait;
use crate::protocols::dsp::orchestrator::rpc::RPCOrchestratorTrait;
use crate::protocols::dsp::orchestrator::rpc::types::{
    RpcNegotiationAgreementMessageDto, RpcNegotiationEventAcceptedMessageDto,
    RpcNegotiationEventFinalizedMessageDto, RpcNegotiationMessageDto,
    RpcNegotiationOfferInitMessageDto, RpcNegotiationRequestInitMessageDto,
    RpcNegotiationTerminationMessageDto, RpcNegotiationVerificationMessageDto,
};
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::{Errors, Outcome};

pub struct BFFRPCOrchestratorService {
    rpc_service: Arc<dyn RPCOrchestratorTrait>,
}

impl BFFRPCOrchestratorService {
    pub fn new(rpc_service: Arc<dyn RPCOrchestratorTrait>) -> Self {
        Self { rpc_service }
    }

    fn extract_pids(&self, model: &NegotiationProcessDto) -> Outcome<(Urn, Urn)> {
        let consumer_str = model
            .identifiers
            .get("consumerPid")
            .ok_or_else(|| Errors::parse("BFF: missing consumerPid in identifiers", None))?;
        let provider_str = model
            .identifiers
            .get("providerPid")
            .ok_or_else(|| Errors::parse("BFF: missing providerPid in identifiers", None))?;

        let consumer_pid = Urn::from_str(consumer_str)
            .map_err(|e| Errors::parse(&format!("BFF: invalid consumerPid URN: {e}"), None))?;
        let provider_pid = Urn::from_str(provider_str)
            .map_err(|e| Errors::parse(&format!("BFF: invalid providerPid URN: {e}"), None))?;

        Ok((consumer_pid, provider_pid))
    }
}

#[async_trait::async_trait]
impl BFFRPCOrchestratorTrait for BFFRPCOrchestratorService {
    async fn setup_negotiation_request_init_bff_rpc(
        &self,
        input: &RpcNegotiationRequestInitMessageDto,
    ) -> Outcome<RpcNegotiationMessageDto<RpcNegotiationRequestInitMessageDto>> {
        self.rpc_service
            .setup_negotiation_request_init_rpc(input)
            .await
    }

    async fn setup_negotiation_offer_init_bff_rpc(
        &self,
        input: &RpcNegotiationOfferInitMessageDto,
    ) -> Outcome<RpcNegotiationMessageDto<RpcNegotiationOfferInitMessageDto>> {
        self.rpc_service
            .setup_negotiation_offer_init_rpc(input)
            .await
    }

    // ACCEPTED -> AGREED -> VERIFIED -> FINALIZED (Provider triggers the tail in one call)
    async fn setup_negotiation_agreement_bff_rpc(
        &self,
        input: &RpcNegotiationAgreementMessageDto,
    ) -> Outcome<RpcNegotiationMessageDto<RpcNegotiationEventFinalizedMessageDto>> {
        let agreement = self
            .rpc_service
            .setup_negotiation_agreement_rpc(input)
            .await?;

        let (consumer_pid, provider_pid) = self.extract_pids(&agreement.negotiation_agent_model)?;

        let verification_input =
            RpcNegotiationVerificationMessageDto::new(consumer_pid.clone(), provider_pid.clone());
        self.rpc_service
            .setup_negotiation_agreement_verification_rpc(&verification_input)
            .await?;

        let finalized_input =
            RpcNegotiationEventFinalizedMessageDto::new(consumer_pid, provider_pid);
        self.rpc_service
            .setup_negotiation_event_finalized_rpc(&finalized_input)
            .await
    }

    // OFFERED -> ACCEPTED
    async fn setup_negotiation_event_accepted_bff_rpc(
        &self,
        input: &RpcNegotiationEventAcceptedMessageDto,
    ) -> Outcome<RpcNegotiationMessageDto<RpcNegotiationEventAcceptedMessageDto>> {
        self.rpc_service
            .setup_negotiation_event_accepted_rpc(input)
            .await
    }

    async fn setup_negotiation_termination_bff_rpc(
        &self,
        input: &RpcNegotiationTerminationMessageDto,
    ) -> Outcome<RpcNegotiationMessageDto<RpcNegotiationTerminationMessageDto>> {
        self.rpc_service
            .setup_negotiation_termination_rpc(input)
            .await
    }
}
