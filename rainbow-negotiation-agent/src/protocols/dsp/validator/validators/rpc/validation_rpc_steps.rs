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

#![allow(unused)]

use crate::protocols::dsp::orchestrator::rpc::types::{
    RpcNegotiationAgreementMessageDto, RpcNegotiationEventAcceptedMessageDto,
    RpcNegotiationEventFinalizedMessageDto, RpcNegotiationOfferInitMessageDto,
    RpcNegotiationOfferMessageDto, RpcNegotiationRequestInitMessageDto,
    RpcNegotiationRequestMessageDto, RpcNegotiationTerminationMessageDto,
    RpcNegotiationVerificationMessageDto,
};
use crate::protocols::dsp::protocol_types::{
    NegotiationAgreementMessageDto, NegotiationEventMessageDto, NegotiationOfferInitMessageDto,
    NegotiationOfferMessageDto, NegotiationProcessMessageWrapper, NegotiationRequestInitMessageDto,
    NegotiationRequestMessageDto, NegotiationTerminationMessageDto,
    NegotiationVerificationMessageDto,
};
use crate::protocols::dsp::validator::traits::validate_payload::ValidatePayload;
use crate::protocols::dsp::validator::traits::validate_state_transition::ValidateStateTransition;
use crate::protocols::dsp::validator::traits::validation_helpers::ValidationHelpers;
use crate::protocols::dsp::validator::traits::validation_rpc_steps::ValidationRpcSteps;
use anyhow::bail;
use std::sync::Arc;

pub struct ValidationRpcStepsService {
    payload_validator: Arc<dyn ValidatePayload>,
    step_transition_validator: Arc<dyn ValidateStateTransition>,
    helpers: Arc<dyn ValidationHelpers>,
}
impl ValidationRpcStepsService {
    pub fn new(
        payload_validator: Arc<dyn ValidatePayload>,
        step_transition_validator: Arc<dyn ValidateStateTransition>,
        helpers: Arc<dyn ValidationHelpers>,
    ) -> Self {
        Self { payload_validator, step_transition_validator, helpers }
    }
}

#[async_trait::async_trait]
impl ValidationRpcSteps for ValidationRpcStepsService {
    async fn negotiation_request_init_rpc(
        &self,
        input: &RpcNegotiationRequestInitMessageDto,
    ) -> anyhow::Result<()> {
        let input: NegotiationProcessMessageWrapper<NegotiationRequestInitMessageDto> =
            input.clone().into();
        // let dto = self.helpers.get_current_dto_from_payload(&input.dto).await?;
        // let role = self.helpers.get_role_from_dto(&dto).await?;
        // let message_type = input._type.clone();
        // let current_state = self.helpers.get_state_from_dto(&dto).await?;
        // self.payload_validator.validate_with_json_schema(&input.dto).await?;
        // self.payload_validator.validate_identifiers_as_urn(&input.dto).await?;
        // self.payload_validator.validate_correlation(&input.dto, &dto).await?;
        // self.payload_validator.validate_auth(&input.dto).await?;
        // self.step_transition_validator.validate_role_for_message(&role, &message_type).await?;
        // self.step_transition_validator.validate_state_transition(&current_state, &message_type).await?;
        Ok(())
    }

    async fn negotiation_request_rpc(
        &self,
        input: &RpcNegotiationRequestMessageDto,
    ) -> anyhow::Result<()> {
        let input: NegotiationProcessMessageWrapper<NegotiationRequestMessageDto> =
            input.clone().into();
        let dto = self.helpers.get_current_dto_from_payload(&input.dto).await?;
        let role = self.helpers.get_role_from_dto(&dto).await?;
        let message_type = input._type.clone();
        let current_state = self.helpers.get_state_from_dto(&dto).await?;
        self.payload_validator.validate_with_json_schema(&input.dto).await?;
        self.payload_validator.validate_identifiers_as_urn(&input.dto).await?;
        self.payload_validator.validate_correlation(&input.dto, &dto).await?;
        self.payload_validator.validate_auth(&input.dto).await?;
        self.step_transition_validator.validate_role_for_message(&role, &message_type).await?;
        self.step_transition_validator
            .validate_state_transition(&current_state, &message_type)
            .await?;
        Ok(())
    }

    async fn negotiation_offer_init_rpc(
        &self,
        input: &RpcNegotiationOfferInitMessageDto,
    ) -> anyhow::Result<()> {
        let input: NegotiationProcessMessageWrapper<NegotiationOfferInitMessageDto> =
            input.clone().into();
        // let dto = self.helpers.get_current_dto_from_payload_by_provider(&input.dto).await?;
        // let role = self.helpers.get_role_from_dto(&dto).await?;
        // let message_type = input._type.clone();
        // let current_state = self.helpers.get_state_from_dto(&dto).await?;
        // self.payload_validator.validate_with_json_schema(&input.dto).await?;
        // self.payload_validator.validate_identifiers_as_urn(&input.dto).await?;
        // self.payload_validator.validate_correlation(&input.dto, &dto).await?;
        // self.payload_validator.validate_auth(&input.dto).await?;
        // self.step_transition_validator.validate_role_for_message(&role, &message_type).await?;
        // self.step_transition_validator
        //     .validate_state_transition(&current_state, &message_type)
        //     .await?;
        Ok(())
    }

    async fn negotiation_offer_rpc(
        &self,
        input: &RpcNegotiationOfferMessageDto,
    ) -> anyhow::Result<()> {
        let input: NegotiationProcessMessageWrapper<NegotiationOfferMessageDto> =
            input.clone().into();
        let dto = self.helpers.get_current_dto_from_payload(&input.dto).await?;
        let role = self.helpers.get_role_from_dto(&dto).await?;
        let message_type = input._type.clone();
        let current_state = self.helpers.get_state_from_dto(&dto).await?;
        self.payload_validator.validate_with_json_schema(&input.dto).await?;
        self.payload_validator.validate_identifiers_as_urn(&input.dto).await?;
        self.payload_validator.validate_correlation(&input.dto, &dto).await?;
        self.payload_validator.validate_auth(&input.dto).await?;
        self.step_transition_validator.validate_role_for_message(&role, &message_type).await?;
        self.step_transition_validator
            .validate_state_transition(&current_state, &message_type)
            .await?;
        Ok(())
    }

    async fn negotiation_agreement_rpc(
        &self,
        input: &RpcNegotiationAgreementMessageDto,
    ) -> anyhow::Result<()> {
        let input: NegotiationProcessMessageWrapper<NegotiationAgreementMessageDto> =
            input.clone().into();
        let dto = self.helpers.get_current_dto_from_payload(&input.dto).await?;
        let role = self.helpers.get_role_from_dto(&dto).await?;
        let message_type = input._type.clone();
        let current_state = self.helpers.get_state_from_dto(&dto).await?;
        self.payload_validator.validate_with_json_schema(&input.dto).await?;
        self.payload_validator.validate_identifiers_as_urn(&input.dto).await?;
        self.payload_validator.validate_correlation(&input.dto, &dto).await?;
        self.payload_validator.validate_auth(&input.dto).await?;
        self.step_transition_validator.validate_role_for_message(&role, &message_type).await?;
        self.step_transition_validator
            .validate_state_transition(&current_state, &message_type)
            .await?;
        Ok(())
    }

    async fn negotiation_agreement_verification_rpc(
        &self,
        input: &RpcNegotiationVerificationMessageDto,
    ) -> anyhow::Result<()> {
        let input: NegotiationProcessMessageWrapper<NegotiationVerificationMessageDto> =
            input.clone().into();
        let dto = self.helpers.get_current_dto_from_payload(&input.dto).await?;
        let role = self.helpers.get_role_from_dto(&dto).await?;
        let message_type = input._type.clone();
        let current_state = self.helpers.get_state_from_dto(&dto).await?;
        self.payload_validator.validate_with_json_schema(&input.dto).await?;
        self.payload_validator.validate_identifiers_as_urn(&input.dto).await?;
        self.payload_validator.validate_correlation(&input.dto, &dto).await?;
        self.payload_validator.validate_auth(&input.dto).await?;
        self.step_transition_validator.validate_role_for_message(&role, &message_type).await?;
        self.step_transition_validator
            .validate_state_transition(&current_state, &message_type)
            .await?;
        Ok(())
    }

    async fn negotiation_event_accepted_rpc(
        &self,
        input: &RpcNegotiationEventAcceptedMessageDto,
    ) -> anyhow::Result<()> {
        let input: NegotiationProcessMessageWrapper<NegotiationEventMessageDto> =
            input.clone().into();
        let dto = self.helpers.get_current_dto_from_payload(&input.dto).await?;
        let role = self.helpers.get_role_from_dto(&dto).await?;
        let message_type = input._type.clone();
        let current_state = self.helpers.get_state_from_dto(&dto).await?;
        self.payload_validator.validate_with_json_schema(&input.dto).await?;
        self.payload_validator.validate_identifiers_as_urn(&input.dto).await?;
        self.payload_validator.validate_correlation(&input.dto, &dto).await?;
        self.payload_validator.validate_auth(&input.dto).await?;
        self.step_transition_validator.validate_role_for_message(&role, &message_type).await?;
        self.step_transition_validator
            .validate_state_transition(&current_state, &message_type)
            .await?;
        Ok(())
    }

    async fn negotiation_event_finalized_rpc(
        &self,
        input: &RpcNegotiationEventFinalizedMessageDto,
    ) -> anyhow::Result<()> {
        let input: NegotiationProcessMessageWrapper<NegotiationEventMessageDto> =
            input.clone().into();
        let dto = self.helpers.get_current_dto_from_payload(&input.dto).await?;
        let role = self.helpers.get_role_from_dto(&dto).await?;
        let message_type = input._type.clone();
        let current_state = self.helpers.get_state_from_dto(&dto).await?;
        self.payload_validator.validate_with_json_schema(&input.dto).await?;
        self.payload_validator.validate_identifiers_as_urn(&input.dto).await?;
        self.payload_validator.validate_correlation(&input.dto, &dto).await?;
        self.payload_validator.validate_auth(&input.dto).await?;
        self.step_transition_validator.validate_role_for_message(&role, &message_type).await?;
        self.step_transition_validator
            .validate_state_transition(&current_state, &message_type)
            .await?;
        Ok(())
    }

    async fn negotiation_termination_rpc(
        &self,
        input: &RpcNegotiationTerminationMessageDto,
    ) -> anyhow::Result<()> {
        let input: NegotiationProcessMessageWrapper<NegotiationTerminationMessageDto> =
            input.clone().into();
        let dto = self.helpers.get_current_dto_from_payload(&input.dto).await?;
        let role = self.helpers.get_role_from_dto(&dto).await?;
        let message_type = input._type.clone();
        let current_state = self.helpers.get_state_from_dto(&dto).await?;
        self.payload_validator.validate_with_json_schema(&input.dto).await?;
        self.payload_validator.validate_identifiers_as_urn(&input.dto).await?;
        self.payload_validator.validate_correlation(&input.dto, &dto).await?;
        self.payload_validator.validate_auth(&input.dto).await?;
        self.step_transition_validator.validate_role_for_message(&role, &message_type).await?;
        self.step_transition_validator
            .validate_state_transition(&current_state, &message_type)
            .await?;
        Ok(())
    }
}
