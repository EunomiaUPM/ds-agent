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

use crate::protocols::dsp::facades::FacadeTrait;
use crate::protocols::dsp::orchestrator::protocol::ProtocolOrchestratorTrait;
use crate::protocols::dsp::orchestrator::protocol::persistence::OrchestrationPersistenceForProtocol;
use crate::protocols::dsp::persistence::NegotiationPersistenceTrait;
use crate::protocols::dsp::protocol_types::{
    NegotiationAckMessageDto, NegotiationAgreementMessageDto, NegotiationEventMessageDto,
    NegotiationEventType, NegotiationOfferInitMessageDto, NegotiationOfferMessageDto,
    NegotiationProcessMessageWrapper, NegotiationRequestInitMessageDto,
    NegotiationRequestMessageDto, NegotiationTerminationMessageDto,
    NegotiationVerificationMessageDto,
};
use crate::protocols::dsp::validator::traits::validation_dsp_steps::ValidationDspSteps;
use rainbow_common::config::services::ContractsConfig;
use rainbow_common::mates::mates::Mates;
use std::sync::Arc;

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
        ProtocolOrchestratorService { validator, persistence_service, _config, facades }
    }
}

#[async_trait::async_trait]
impl ProtocolOrchestratorTrait for ProtocolOrchestratorService {
    async fn on_get_negotiation(
        &self,
        id: &String,
    ) -> anyhow::Result<NegotiationProcessMessageWrapper<NegotiationAckMessageDto>> {
        let process = self.persistence_service.fetch_process(id.as_str()).await?;
        let negotiation_process_dto = NegotiationProcessMessageWrapper::try_from(process)?;
        Ok(negotiation_process_dto)
    }

    async fn on_initial_contract_request(
        &self,
        input: &NegotiationProcessMessageWrapper<NegotiationRequestInitMessageDto>,
        mate: &Mates,
    ) -> anyhow::Result<(NegotiationProcessMessageWrapper<NegotiationAckMessageDto>, bool)> {
        self.validator.on_contract_request_init(&input).await?;

        // persist
        let negotiation = self.persistence_service.create_new(&input.dto, mate).await?;
        // notify
        let negotiation_process_dto = NegotiationProcessMessageWrapper::try_from(negotiation)?;
        Ok((negotiation_process_dto, false))
    }

    async fn on_consumer_request(
        &self,
        id: &String,
        input: &NegotiationProcessMessageWrapper<NegotiationRequestMessageDto>,
        mate: &Mates,
    ) -> anyhow::Result<NegotiationProcessMessageWrapper<NegotiationAckMessageDto>> {
        self.validator.on_contract_request(id, &input).await?;

        // persist
        let negotiation =
            self.persistence_service.update_with_offer(id.as_str(), &input.dto, mate).await?;
        // notify
        let negotiation_process_dto = NegotiationProcessMessageWrapper::try_from(negotiation)?;
        Ok(negotiation_process_dto)
    }

    async fn on_agreement_verification(
        &self,
        id: &String,
        input: &NegotiationProcessMessageWrapper<NegotiationVerificationMessageDto>,
        mate: &Mates,
    ) -> anyhow::Result<NegotiationProcessMessageWrapper<NegotiationAckMessageDto>> {
        self.validator.on_contract_agreement_verification(&id, &input).await?;

        // persist
        let negotiation = self.persistence_service.update(id.as_str(), &input.dto, mate).await?;
        // notify
        let negotiation_process_dto = NegotiationProcessMessageWrapper::try_from(negotiation)?;
        Ok(negotiation_process_dto)
    }

    async fn on_initial_provider_offer(
        &self,
        input: &NegotiationProcessMessageWrapper<NegotiationOfferInitMessageDto>,
        mate: &Mates,
    ) -> anyhow::Result<(NegotiationProcessMessageWrapper<NegotiationAckMessageDto>, bool)> {
        self.validator.on_contract_offer_init(&input).await?;

        // persist
        let negotiation = self.persistence_service.create_new(&input.dto, mate).await?;
        // notify
        let negotiation_process_dto = NegotiationProcessMessageWrapper::try_from(negotiation)?;
        Ok((negotiation_process_dto, false))
    }

    async fn on_provider_offer(
        &self,
        id: &String,
        input: &NegotiationProcessMessageWrapper<NegotiationOfferMessageDto>,
        mate: &Mates,
    ) -> anyhow::Result<NegotiationProcessMessageWrapper<NegotiationAckMessageDto>> {
        self.validator.on_contract_offer(id, &input).await?;

        // persist
        let negotiation =
            self.persistence_service.update_with_offer(id.as_str(), &input.dto, mate).await?;
        // notify
        let negotiation_process_dto = NegotiationProcessMessageWrapper::try_from(negotiation)?;
        Ok(negotiation_process_dto)
    }

    async fn on_agreement_reception(
        &self,
        id: &String,
        input: &NegotiationProcessMessageWrapper<NegotiationAgreementMessageDto>,
        mate: &Mates,
    ) -> anyhow::Result<NegotiationProcessMessageWrapper<NegotiationAckMessageDto>> {
        self.validator.on_contract_agreement(id, &input).await?;

        // persist
        let negotiation = self
            .persistence_service
            .update_with_new_agreement(id.as_str(), &input.dto, mate)
            .await?;
        // notify
        let negotiation_process_dto = NegotiationProcessMessageWrapper::try_from(negotiation)?;
        Ok(negotiation_process_dto)
    }

    async fn on_negotiation_event(
        &self,
        id: &String,
        input: &NegotiationProcessMessageWrapper<NegotiationEventMessageDto>,
        mate: &Mates,
    ) -> anyhow::Result<NegotiationProcessMessageWrapper<NegotiationAckMessageDto>> {
        self.validator.on_contract_event(id, &input).await?;

        // persist
        let negotiation = match &input.dto.event_type {
            NegotiationEventType::ACCEPTED => {
                self.persistence_service.update(id.as_str(), &input.dto, mate).await?
            }
            NegotiationEventType::FINALIZED => {
                self.persistence_service
                    .update_with_agreement(id.as_str(), &input.dto, mate)
                    .await?
            }
        };
        // notify
        let negotiation_process_dto = NegotiationProcessMessageWrapper::try_from(negotiation)?;
        Ok(negotiation_process_dto)
    }

    async fn on_negotiation_termination(
        &self,
        id: &String,
        input: &NegotiationProcessMessageWrapper<NegotiationTerminationMessageDto>,
        mate: &Mates,
    ) -> anyhow::Result<NegotiationProcessMessageWrapper<NegotiationAckMessageDto>> {
        self.validator.on_contract_termination(id, &input).await?;

        // persist
        let negotiation = self.persistence_service.update(id.as_str(), &input.dto, mate).await?;
        // notify
        let negotiation_process_dto = NegotiationProcessMessageWrapper::try_from(negotiation)?;
        Ok(negotiation_process_dto)
    }
}
