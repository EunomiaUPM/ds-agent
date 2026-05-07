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

use crate::protocols::dsp::facades::FacadeTrait;
use crate::protocols::dsp::orchestrator::protocol::ProtocolOrchestratorTrait;
use crate::protocols::dsp::orchestrator::protocol::persistence::OrchestrationPersistenceForProtocol;
use crate::protocols::dsp::orchestrator::protocol::step_agreement_reception::AgreementReceptionStep;
use crate::protocols::dsp::orchestrator::protocol::step_agreement_verification::AgreementVerificationStep;
use crate::protocols::dsp::orchestrator::protocol::step_consumer_request::ConsumerRequestStep;
use crate::protocols::dsp::orchestrator::protocol::step_initial_offer::InitialProviderOfferStep;
use crate::protocols::dsp::orchestrator::protocol::step_initial_request::InitialContractRequestStep;
use crate::protocols::dsp::orchestrator::protocol::step_negotiation_event::NegotiationEventStep;
use crate::protocols::dsp::orchestrator::protocol::step_provider_offer::ProviderOfferStep;
use crate::protocols::dsp::orchestrator::protocol::step_termination::NegotiationTerminationStep;
use crate::protocols::dsp::orchestrator::protocol::step_trait::NegotiationProtocolStep;
use crate::protocols::dsp::protocol_types::{
    NegotiationAckMessageDto, NegotiationAgreementMessageDto, NegotiationEventMessageDto,
    NegotiationOfferInitMessageDto, NegotiationOfferMessageDto, NegotiationProcessMessageWrapper,
    NegotiationRequestInitMessageDto, NegotiationRequestMessageDto,
    NegotiationTerminationMessageDto, NegotiationVerificationMessageDto,
};
use crate::protocols::dsp::validator::traits::validation_dsp_steps::ValidationDspSteps;
use common::config::services::ContractsConfig;
use common::facades::Mates;
use std::sync::Arc;
use ymir::errors::Outcome;
// ─── Service ──────────────────────────────────────────────────────────────────

/// DSP protocol orchestrator for inbound negotiation operations.
///
/// Handles messages arriving on the DSP HTTP endpoints.  All eight operations
/// (two initial + six lifecycle steps) are driven by the
/// [`NegotiationProtocolStep`] template; `run_lifecycle` encodes the algorithm
/// once:
///
/// 1. **validate** - 2. **prepare context** (with optional early ack) →
/// 3. **persist**
///
/// Unlike the transfer orchestrator there is no `post_hook` because negotiation
/// does not involve a data-plane session.
pub struct ProtocolOrchestratorService {
    facades: Arc<dyn FacadeTrait>,
    validator: Arc<dyn ValidationDspSteps>,
    persistence_service: Arc<OrchestrationPersistenceForProtocol>,
    _config: Arc<ContractsConfig>,
}

impl ProtocolOrchestratorService {
    pub fn new(
        validator: Arc<dyn ValidationDspSteps>,
        persistence_service: Arc<OrchestrationPersistenceForProtocol>,
        facades: Arc<dyn FacadeTrait>,
        _config: Arc<ContractsConfig>,
    ) -> ProtocolOrchestratorService {
        ProtocolOrchestratorService {
            validator,
            persistence_service,
            _config,
            facades,
        }
    }
}

// ─── Trait implementation ──────────────────────────────────────────────────────

#[async_trait::async_trait]
impl ProtocolOrchestratorTrait for ProtocolOrchestratorService {
    async fn on_get_negotiation(
        &self,
        id: &String,
    ) -> Outcome<NegotiationProcessMessageWrapper<NegotiationAckMessageDto>> {
        let process = self.persistence_service.fetch_process(id.as_str()).await?;
        let negotiation_process_dto = NegotiationProcessMessageWrapper::try_from(process)?;
        Ok(negotiation_process_dto)
    }

    async fn on_initial_contract_request(
        &self,
        input: &NegotiationProcessMessageWrapper<NegotiationRequestInitMessageDto>,
        mate: &Mates,
    ) -> Outcome<(
        NegotiationProcessMessageWrapper<NegotiationAckMessageDto>,
        bool,
    )> {
        self.run_lifecycle::<InitialContractRequestStep>("", mate, input)
            .await
    }

    async fn on_consumer_request(
        &self,
        id: &String,
        input: &NegotiationProcessMessageWrapper<NegotiationRequestMessageDto>,
        mate: &Mates,
    ) -> Outcome<NegotiationProcessMessageWrapper<NegotiationAckMessageDto>> {
        let (ack, _) = self
            .run_lifecycle::<ConsumerRequestStep>(id, mate, input)
            .await?;
        Ok(ack)
    }

    async fn on_agreement_verification(
        &self,
        id: &String,
        input: &NegotiationProcessMessageWrapper<NegotiationVerificationMessageDto>,
        mate: &Mates,
    ) -> Outcome<NegotiationProcessMessageWrapper<NegotiationAckMessageDto>> {
        let (ack, _) = self
            .run_lifecycle::<AgreementVerificationStep>(id, mate, input)
            .await?;
        Ok(ack)
    }

    async fn on_initial_provider_offer(
        &self,
        input: &NegotiationProcessMessageWrapper<NegotiationOfferInitMessageDto>,
        mate: &Mates,
    ) -> Outcome<(
        NegotiationProcessMessageWrapper<NegotiationAckMessageDto>,
        bool,
    )> {
        self.run_lifecycle::<InitialProviderOfferStep>("", mate, input)
            .await
    }

    async fn on_provider_offer(
        &self,
        id: &String,
        input: &NegotiationProcessMessageWrapper<NegotiationOfferMessageDto>,
        mate: &Mates,
    ) -> Outcome<NegotiationProcessMessageWrapper<NegotiationAckMessageDto>> {
        let (ack, _) = self
            .run_lifecycle::<ProviderOfferStep>(id, mate, input)
            .await?;
        Ok(ack)
    }

    async fn on_agreement_reception(
        &self,
        id: &String,
        input: &NegotiationProcessMessageWrapper<NegotiationAgreementMessageDto>,
        mate: &Mates,
    ) -> Outcome<NegotiationProcessMessageWrapper<NegotiationAckMessageDto>> {
        let (ack, _) = self
            .run_lifecycle::<AgreementReceptionStep>(id, mate, input)
            .await?;
        Ok(ack)
    }

    async fn on_negotiation_event(
        &self,
        id: &String,
        input: &NegotiationProcessMessageWrapper<NegotiationEventMessageDto>,
        mate: &Mates,
    ) -> Outcome<NegotiationProcessMessageWrapper<NegotiationAckMessageDto>> {
        let (ack, _) = self
            .run_lifecycle::<NegotiationEventStep>(id, mate, input)
            .await?;
        Ok(ack)
    }

    async fn on_negotiation_termination(
        &self,
        id: &String,
        input: &NegotiationProcessMessageWrapper<NegotiationTerminationMessageDto>,
        mate: &Mates,
    ) -> Outcome<NegotiationProcessMessageWrapper<NegotiationAckMessageDto>> {
        let (ack, _) = self
            .run_lifecycle::<NegotiationTerminationStep>(id, mate, input)
            .await?;
        Ok(ack)
    }
}

// ─── Template engine ───────────────────────────────────────────────────────────

impl ProtocolOrchestratorService {
    /// Execute any inbound DSP negotiation lifecycle step using the
    /// [`NegotiationProtocolStep`] template.
    ///
    /// The algorithm is the same regardless of step type:
    /// validate - prepare context (with optional early ack) - persist.
    ///
    /// `id` is the peer-facing process identifier (continuation steps; pass `""`
    /// for initial steps that create a new process).
    /// `mate` is the authenticated peer identity provided by the DSP middleware.
    async fn run_lifecycle<S: NegotiationProtocolStep>(
        &self,
        id: &str,
        mate: &Mates,
        input: &NegotiationProcessMessageWrapper<S::Dto>,
    ) -> Outcome<(
        NegotiationProcessMessageWrapper<NegotiationAckMessageDto>,
        bool,
    )> {
        S::validate(&self.validator, id, input, mate).await?;

        let (ctx, early) = S::prepare_context(id, mate, input, &self.persistence_service).await?;
        if let Some(early_ack) = early {
            return Ok((early_ack, true));
        }

        let process = S::persist(&self.persistence_service, id, &ctx, input, mate).await?;
        let ack = NegotiationProcessMessageWrapper::try_from(process)?;
        Ok((ack, false))
    }
}
