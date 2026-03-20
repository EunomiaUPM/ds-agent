/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use crate::entities::negotiation_process::NegotiationProcessDto;
use crate::protocols::dsp::persistence::NegotiationRpcPersistenceTrait;
use crate::protocols::dsp::orchestrator::rpc::step_agreement::RpcAgreementStep;
use crate::protocols::dsp::orchestrator::rpc::step_event_accepted::RpcEventAcceptedStep;
use crate::protocols::dsp::orchestrator::rpc::step_event_finalized::RpcEventFinalizedStep;
use crate::protocols::dsp::orchestrator::rpc::step_offer::RpcOfferStep;
use crate::protocols::dsp::orchestrator::rpc::step_offer_init::RpcOfferInitStep;
use crate::protocols::dsp::orchestrator::rpc::step_request::RpcRequestStep;
use crate::protocols::dsp::orchestrator::rpc::step_request_init::RpcRequestInitStep;
use crate::protocols::dsp::orchestrator::rpc::step_termination::RpcTerminationStep;
use crate::protocols::dsp::orchestrator::rpc::step_trait::NegotiationRpcStep;
use crate::protocols::dsp::orchestrator::rpc::step_verification::RpcVerificationStep;
use crate::protocols::dsp::orchestrator::rpc::types::{
    RpcNegotiationAgreementMessageDto, RpcNegotiationEventAcceptedMessageDto,
    RpcNegotiationEventFinalizedMessageDto, RpcNegotiationMessageDto,
    RpcNegotiationOfferInitMessageDto, RpcNegotiationOfferMessageDto,
    RpcNegotiationRequestInitMessageDto, RpcNegotiationRequestMessageDto,
    RpcNegotiationTerminationMessageDto, RpcNegotiationVerificationMessageDto,
};
use crate::protocols::dsp::orchestrator::rpc::RPCOrchestratorTrait;
use crate::protocols::dsp::protocol_types::{
    NegotiationAckMessageDto, NegotiationProcessMessageWrapper,
};
use crate::protocols::dsp::validator::traits::validation_rpc_steps::ValidationRpcSteps;
use common::config::services::ContractsConfig;
use common::facades::ssi_auth_facade::MatesFacadeTrait;
use common::http_client::HttpClient;
use std::sync::Arc;
use ymir::errors::Outcome;

// ─── Service ──────────────────────────────────────────────────────────────────

/// RPC orchestrator for outbound negotiation operations.
///
/// Translates internal RPC requests into DSP protocol messages, sends them to
/// the remote peer over HTTP, and persists the resulting state transitions.
/// All nine operations (two initial + seven lifecycle steps) are driven by the
/// [`NegotiationRpcStep`] template; `run_lifecycle` encodes the algorithm once:
///
/// validate → prepare context → auth → send + persist
#[allow(unused)]
pub struct RPCOrchestratorService {
    validator: Arc<dyn ValidationRpcSteps>,
    persistence_service: Arc<dyn NegotiationRpcPersistenceTrait>,
    _config: Arc<ContractsConfig>,
    http_client: Arc<HttpClient>,
    mates_service: Arc<dyn MatesFacadeTrait>,
}

impl RPCOrchestratorService {
    pub fn new(
        validator: Arc<dyn ValidationRpcSteps>,
        persistence_service: Arc<dyn NegotiationRpcPersistenceTrait>,
        _config: Arc<ContractsConfig>,
        http_client: Arc<HttpClient>,
        mates_service: Arc<dyn MatesFacadeTrait>,
    ) -> RPCOrchestratorService {
        RPCOrchestratorService {
            validator,
            persistence_service,
            _config,
            http_client,
            mates_service,
        }
    }
}

// ─── Trait implementation ──────────────────────────────────────────────────────

#[async_trait::async_trait]
impl RPCOrchestratorTrait for RPCOrchestratorService {
    /// Sends an initial `ContractRequestMessage` to the Provider (Consumer-initiated flow).
    async fn setup_negotiation_request_init_rpc(
        &self,
        input: &RpcNegotiationRequestInitMessageDto,
    ) -> Outcome<RpcNegotiationMessageDto<RpcNegotiationRequestInitMessageDto>> {
        let (response, process) = self.run_lifecycle::<RpcRequestInitStep>(input).await?;
        Ok(RpcNegotiationMessageDto { request: input.clone(), response, negotiation_agent_model: process })
    }

    /// Sends a continuation `ContractRequestMessage` (Consumer counter-offer).
    async fn setup_negotiation_request_rpc(
        &self,
        input: &RpcNegotiationRequestMessageDto,
    ) -> Outcome<RpcNegotiationMessageDto<RpcNegotiationRequestMessageDto>> {
        let (response, process) = self.run_lifecycle::<RpcRequestStep>(input).await?;
        Ok(RpcNegotiationMessageDto { request: input.clone(), response, negotiation_agent_model: process })
    }

    /// Sends an initial `ContractOfferMessage` to the Consumer (Provider-initiated flow).
    async fn setup_negotiation_offer_init_rpc(
        &self,
        input: &RpcNegotiationOfferInitMessageDto,
    ) -> Outcome<RpcNegotiationMessageDto<RpcNegotiationOfferInitMessageDto>> {
        let (response, process) = self.run_lifecycle::<RpcOfferInitStep>(input).await?;
        Ok(RpcNegotiationMessageDto { request: input.clone(), response, negotiation_agent_model: process })
    }

    /// Sends a continuation `ContractOfferMessage` (Provider counter-offer).
    async fn setup_negotiation_offer_rpc(
        &self,
        input: &RpcNegotiationOfferMessageDto,
    ) -> Outcome<RpcNegotiationMessageDto<RpcNegotiationOfferMessageDto>> {
        let (response, process) = self.run_lifecycle::<RpcOfferStep>(input).await?;
        Ok(RpcNegotiationMessageDto { request: input.clone(), response, negotiation_agent_model: process })
    }

    /// Sends a `ContractAgreementMessage` to the Consumer.
    ///
    /// The agreement body is enriched with offer policy fields and participant
    /// IDs in [`RpcAgreementStep::prepare_context`].
    async fn setup_negotiation_agreement_rpc(
        &self,
        input: &RpcNegotiationAgreementMessageDto,
    ) -> Outcome<RpcNegotiationMessageDto<RpcNegotiationAgreementMessageDto>> {
        let (response, process) = self.run_lifecycle::<RpcAgreementStep>(input).await?;
        Ok(RpcNegotiationMessageDto { request: input.clone(), response, negotiation_agent_model: process })
    }

    /// Sends a `ContractAgreementVerificationMessage` to the Provider.
    async fn setup_negotiation_agreement_verification_rpc(
        &self,
        input: &RpcNegotiationVerificationMessageDto,
    ) -> Outcome<RpcNegotiationMessageDto<RpcNegotiationVerificationMessageDto>> {
        let (response, process) = self.run_lifecycle::<RpcVerificationStep>(input).await?;
        Ok(RpcNegotiationMessageDto { request: input.clone(), response, negotiation_agent_model: process })
    }

    /// Sends a `ContractNegotiationEventMessage` with event type `ACCEPTED`.
    async fn setup_negotiation_event_accepted_rpc(
        &self,
        input: &RpcNegotiationEventAcceptedMessageDto,
    ) -> Outcome<RpcNegotiationMessageDto<RpcNegotiationEventAcceptedMessageDto>> {
        let (response, process) = self.run_lifecycle::<RpcEventAcceptedStep>(input).await?;
        Ok(RpcNegotiationMessageDto { request: input.clone(), response, negotiation_agent_model: process })
    }

    /// Sends a `ContractNegotiationEventMessage` with event type `FINALIZED`.
    async fn setup_negotiation_event_finalized_rpc(
        &self,
        input: &RpcNegotiationEventFinalizedMessageDto,
    ) -> Outcome<RpcNegotiationMessageDto<RpcNegotiationEventFinalizedMessageDto>> {
        let (response, process) = self.run_lifecycle::<RpcEventFinalizedStep>(input).await?;
        Ok(RpcNegotiationMessageDto { request: input.clone(), response, negotiation_agent_model: process })
    }

    /// Sends a `ContractNegotiationTerminationMessage` to the peer.
    async fn setup_negotiation_termination_rpc(
        &self,
        input: &RpcNegotiationTerminationMessageDto,
    ) -> Outcome<RpcNegotiationMessageDto<RpcNegotiationTerminationMessageDto>> {
        let (response, process) = self.run_lifecycle::<RpcTerminationStep>(input).await?;
        Ok(RpcNegotiationMessageDto { request: input.clone(), response, negotiation_agent_model: process })
    }
}

// ─── Template engine ───────────────────────────────────────────────────────────

impl RPCOrchestratorService {
    /// Execute any RPC negotiation lifecycle step using the
    /// [`NegotiationRpcStep`] template.
    ///
    /// The algorithm is the same regardless of step type:
    /// validate → prepare context → auth → send + persist.
    ///
    /// Unlike the transfer RPC template there is no `pre_hook` / `post_hook`
    /// because negotiation does not involve a data-plane session.
    async fn run_lifecycle<S: NegotiationRpcStep>(
        &self,
        input: &S::Input,
    ) -> Outcome<(
        NegotiationProcessMessageWrapper<NegotiationAckMessageDto>,
        NegotiationProcessDto,
    )> {
        S::validate(&self.validator, input).await?;
        let ctx =
            S::prepare_context(input, &self.persistence_service, &self.mates_service).await?;
        S::apply_auth_token(&self.mates_service, &self.http_client, S::auth_peer(&ctx)).await;
        let (response, process) =
            S::send_and_persist(&self.http_client, &self.persistence_service, &ctx, input).await?;
        Ok((response, process))
    }
}
