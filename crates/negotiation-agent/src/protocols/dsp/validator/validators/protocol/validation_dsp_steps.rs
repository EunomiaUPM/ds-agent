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

use crate::protocols::dsp::protocol_types::{
    NegotiationAgreementMessageDto, NegotiationEventMessageDto, NegotiationOfferInitMessageDto,
    NegotiationOfferMessageDto, NegotiationProcessMessageWrapper, NegotiationRequestInitMessageDto,
    NegotiationRequestMessageDto, NegotiationTerminationMessageDto,
    NegotiationVerificationMessageDto,
};
use crate::protocols::dsp::validator::traits::validate_payload::ValidatePayload;
use crate::protocols::dsp::validator::traits::validate_state_transition::ValidateStateTransition;
use crate::protocols::dsp::validator::traits::validation_dsp_steps::ValidationDspSteps;
use crate::protocols::dsp::validator::traits::validation_helpers::ValidationHelpers;
use std::sync::Arc;
use ymir::errors::Outcome;

pub struct ValidationDspStepsService {
    payload_validator: Arc<dyn ValidatePayload>,
    step_transition_validator: Arc<dyn ValidateStateTransition>,
    helpers: Arc<dyn ValidationHelpers>,
}
impl ValidationDspStepsService {
    pub fn new(
        payload_validator: Arc<dyn ValidatePayload>,
        step_transition_validator: Arc<dyn ValidateStateTransition>,
        helpers: Arc<dyn ValidationHelpers>,
    ) -> Self {
        Self {
            payload_validator,
            step_transition_validator,
            helpers,
        }
    }
}

#[async_trait::async_trait]
impl ValidationDspSteps for ValidationDspStepsService {
    async fn on_contract_request_init(
        &self,
        input: &NegotiationProcessMessageWrapper<NegotiationRequestInitMessageDto>,
    ) -> Outcome<()> {
        self.payload_validator
            .validate_with_json_schema(&input.dto)
            .await?;
        self.payload_validator
            .validate_identifiers_as_urn(&input.dto)
            .await?;
        self.payload_validator.validate_auth(&input.dto).await?;
        Ok(())
    }

    async fn on_contract_request(
        &self,
        uri_id: &String,
        input: &NegotiationProcessMessageWrapper<NegotiationRequestMessageDto>,
    ) -> Outcome<()> {
        let dto = self
            .helpers
            .get_current_dto_from_payload(&input.dto)
            .await?;
        let role = self.helpers.get_role_from_dto(&dto).await?;
        let message_type = input._type.clone();
        let current_state = self.helpers.get_state_from_dto(&dto).await?;
        let current_state_attribute = self.helpers.get_state_attribute_from_dto(&dto).await?;
        self.payload_validator
            .validate_with_json_schema(&input.dto)
            .await?;
        self.payload_validator
            .validate_uri_id_as_urn(uri_id)
            .await?;
        self.payload_validator
            .validate_identifiers_as_urn(&input.dto)
            .await?;
        self.payload_validator
            .validate_uri_and_pid(uri_id, &input.dto, &role)
            .await?;
        self.payload_validator
            .validate_correlation(&input.dto, &dto)
            .await?;
        self.payload_validator.validate_auth(&input.dto).await?;
        self.step_transition_validator
            .validate_role_for_message(&role, &message_type)
            .await?;
        self.step_transition_validator
            .validate_state_transition(&current_state, &message_type)
            .await?;
        Ok(())
    }

    async fn on_contract_offer_init(
        &self,
        input: &NegotiationProcessMessageWrapper<NegotiationOfferInitMessageDto>,
    ) -> Outcome<()> {
        self.payload_validator
            .validate_with_json_schema(&input.dto)
            .await?;
        self.payload_validator
            .validate_identifiers_as_urn(&input.dto)
            .await?;
        self.payload_validator.validate_auth(&input.dto).await?;
        Ok(())
    }

    async fn on_contract_offer(
        &self,
        uri_id: &String,
        input: &NegotiationProcessMessageWrapper<NegotiationOfferMessageDto>,
    ) -> Outcome<()> {
        let dto = self
            .helpers
            .get_current_dto_from_payload(&input.dto)
            .await?;
        let role = self.helpers.get_role_from_dto(&dto).await?;
        let message_type = input._type.clone();
        let current_state = self.helpers.get_state_from_dto(&dto).await?;
        let current_state_attribute = self.helpers.get_state_attribute_from_dto(&dto).await?;
        self.payload_validator
            .validate_with_json_schema(&input.dto)
            .await?;
        self.payload_validator
            .validate_uri_id_as_urn(uri_id)
            .await?;
        self.payload_validator
            .validate_identifiers_as_urn(&input.dto)
            .await?;
        self.payload_validator
            .validate_uri_and_pid(uri_id, &input.dto, &role)
            .await?;
        self.payload_validator
            .validate_correlation(&input.dto, &dto)
            .await?;
        self.payload_validator.validate_auth(&input.dto).await?;
        self.step_transition_validator
            .validate_role_for_message(&role, &message_type)
            .await?;
        self.step_transition_validator
            .validate_state_transition(&current_state, &message_type)
            .await?;
        Ok(())
    }

    async fn on_contract_agreement(
        &self,
        uri_id: &String,
        input: &NegotiationProcessMessageWrapper<NegotiationAgreementMessageDto>,
    ) -> Outcome<()> {
        let dto = self
            .helpers
            .get_current_dto_from_payload(&input.dto)
            .await?;
        let role = self.helpers.get_role_from_dto(&dto).await?;
        let message_type = input._type.clone();
        let current_state = self.helpers.get_state_from_dto(&dto).await?;
        let current_state_attribute = self.helpers.get_state_attribute_from_dto(&dto).await?;
        self.payload_validator
            .validate_with_json_schema(&input.dto)
            .await?;
        self.payload_validator
            .validate_uri_id_as_urn(uri_id)
            .await?;
        self.payload_validator
            .validate_identifiers_as_urn(&input.dto)
            .await?;
        self.payload_validator
            .validate_uri_and_pid(uri_id, &input.dto, &role)
            .await?;
        self.payload_validator
            .validate_correlation(&input.dto, &dto)
            .await?;
        self.payload_validator.validate_auth(&input.dto).await?;
        self.step_transition_validator
            .validate_role_for_message(&role, &message_type)
            .await?;
        self.step_transition_validator
            .validate_state_transition(&current_state, &message_type)
            .await?;
        Ok(())
    }

    async fn on_contract_agreement_verification(
        &self,
        uri_id: &String,
        input: &NegotiationProcessMessageWrapper<NegotiationVerificationMessageDto>,
    ) -> Outcome<()> {
        let dto = self
            .helpers
            .get_current_dto_from_payload(&input.dto)
            .await?;
        let role = self.helpers.get_role_from_dto(&dto).await?;
        let message_type = input._type.clone();
        let current_state = self.helpers.get_state_from_dto(&dto).await?;
        let current_state_attribute = self.helpers.get_state_attribute_from_dto(&dto).await?;
        self.payload_validator
            .validate_with_json_schema(&input.dto)
            .await?;
        self.payload_validator
            .validate_uri_id_as_urn(uri_id)
            .await?;
        self.payload_validator
            .validate_identifiers_as_urn(&input.dto)
            .await?;
        self.payload_validator
            .validate_uri_and_pid(uri_id, &input.dto, &role)
            .await?;
        self.payload_validator
            .validate_correlation(&input.dto, &dto)
            .await?;
        self.payload_validator.validate_auth(&input.dto).await?;
        self.step_transition_validator
            .validate_role_for_message(&role, &message_type)
            .await?;
        self.step_transition_validator
            .validate_state_transition(&current_state, &message_type)
            .await?;
        Ok(())
    }

    async fn on_contract_event(
        &self,
        uri_id: &String,
        input: &NegotiationProcessMessageWrapper<NegotiationEventMessageDto>,
    ) -> Outcome<()> {
        let dto = self
            .helpers
            .get_current_dto_from_payload(&input.dto)
            .await?;
        let role = self.helpers.get_role_from_dto(&dto).await?;
        let message_type = input._type.clone();
        let current_state = self.helpers.get_state_from_dto(&dto).await?;
        let current_state_attribute = self.helpers.get_state_attribute_from_dto(&dto).await?;
        self.payload_validator
            .validate_with_json_schema(&input.dto)
            .await?;
        self.payload_validator
            .validate_uri_id_as_urn(uri_id)
            .await?;
        self.payload_validator
            .validate_identifiers_as_urn(&input.dto)
            .await?;
        self.payload_validator
            .validate_uri_and_pid(uri_id, &input.dto, &role)
            .await?;
        self.payload_validator
            .validate_correlation(&input.dto, &dto)
            .await?;
        self.payload_validator.validate_auth(&input.dto).await?;
        self.step_transition_validator
            .validate_role_for_message(&role, &message_type)
            .await?;
        self.step_transition_validator
            .validate_state_transition(&current_state, &message_type)
            .await?;
        Ok(())
    }

    async fn on_contract_termination(
        &self,
        uri_id: &String,
        input: &NegotiationProcessMessageWrapper<NegotiationTerminationMessageDto>,
    ) -> Outcome<()> {
        let dto = self
            .helpers
            .get_current_dto_from_payload(&input.dto)
            .await?;
        let role = self.helpers.get_role_from_dto(&dto).await?;
        let message_type = input._type.clone();
        let current_state = self.helpers.get_state_from_dto(&dto).await?;
        let current_state_attribute = self.helpers.get_state_attribute_from_dto(&dto).await?;
        self.payload_validator
            .validate_with_json_schema(&input.dto)
            .await?;
        self.payload_validator
            .validate_uri_id_as_urn(uri_id)
            .await?;
        self.payload_validator
            .validate_identifiers_as_urn(&input.dto)
            .await?;
        self.payload_validator
            .validate_uri_and_pid(uri_id, &input.dto, &role)
            .await?;
        self.payload_validator
            .validate_correlation(&input.dto, &dto)
            .await?;
        self.payload_validator.validate_auth(&input.dto).await?;
        self.step_transition_validator
            .validate_role_for_message(&role, &message_type)
            .await?;
        self.step_transition_validator
            .validate_state_transition(&current_state, &message_type)
            .await?;
        Ok(())
    }
}
