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
    RpcTransferCompletionMessageDto, RpcTransferRequestMessageDto, RpcTransferStartMessageDto,
    RpcTransferSuspensionMessageDto, RpcTransferTerminationMessageDto,
};
use crate::protocols::dsp::protocol_types::{
    TransferCompletionMessageDto, TransferProcessMessageWrapper, TransferRequestMessageDto,
    TransferStartMessageDto, TransferSuspensionMessageDto, TransferTerminationMessageDto,
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
    async fn transfer_request_rpc(
        &self,
        input: &RpcTransferRequestMessageDto,
    ) -> anyhow::Result<()> {
        let request_body: TransferProcessMessageWrapper<TransferRequestMessageDto> =
            input.clone().into();
        //self.payload_validator.validate_format_data_address(&request_body.dto).await?;
        Ok(())
    }

    async fn transfer_start_rpc(&self, input: &RpcTransferStartMessageDto) -> anyhow::Result<()> {
        // review well this...
        let input: TransferProcessMessageWrapper<TransferStartMessageDto> = input.clone().into();
        let dto = self.helpers.get_current_dto_from_payload(&input.dto).await?;
        let role = self.helpers.get_role_from_dto(&dto).await?;
        let message_type = input._type.clone();
        let current_state = self.helpers.get_state_from_dto(&dto).await?;
        let current_state_attribute = self.helpers.get_state_attribute_from_dto(&dto).await?;
        self.payload_validator.validate_with_json_schema(&input.dto).await?;
        self.payload_validator.validate_identifiers_as_urn(&input.dto).await?;
        self.payload_validator.validate_correlation(&input.dto, &dto).await?;
        self.payload_validator.validate_auth(&input.dto).await?;
        self.payload_validator.validate_data_address_in_start(&input.dto, &dto).await?;
        self.step_transition_validator.validate_role_for_message(&role, &message_type).await?;
        self.step_transition_validator
            .validate_state_transition(&current_state, &message_type)
            .await?;
        self.step_transition_validator
            .validate_state_attribute_transition(
                &current_state,
                &current_state_attribute,
                &message_type,
                &role,
            )
            .await?;
        Ok(())
    }

    async fn transfer_completion_rpc(
        &self,
        input: &RpcTransferCompletionMessageDto,
    ) -> anyhow::Result<()> {
        let input: TransferProcessMessageWrapper<TransferCompletionMessageDto> =
            input.clone().into();
        let dto = self.helpers.get_current_dto_from_payload(&input.dto).await?;
        let role = self.helpers.get_role_from_dto(&dto).await?;

        let message_type = input._type.clone();

        let current_state = self.helpers.get_state_from_dto(&dto).await?;
        let current_state_attribute = self.helpers.get_state_attribute_from_dto(&dto).await?;
        self.payload_validator.validate_with_json_schema(&input.dto).await?;
        self.payload_validator.validate_identifiers_as_urn(&input.dto).await?;
        self.payload_validator.validate_correlation(&input.dto, &dto).await?;
        self.payload_validator.validate_auth(&input.dto).await?;
        self.step_transition_validator.validate_role_for_message(&role, &message_type).await?;
        self.step_transition_validator
            .validate_state_transition(&current_state, &message_type)
            .await?;
        self.step_transition_validator
            .validate_state_attribute_transition(
                &current_state,
                &current_state_attribute,
                &message_type,
                &role,
            )
            .await?;
        Ok(())
    }

    async fn transfer_suspension_rpc(
        &self,
        input: &RpcTransferSuspensionMessageDto,
    ) -> anyhow::Result<()> {
        let input: TransferProcessMessageWrapper<TransferSuspensionMessageDto> =
            input.clone().into();
        let dto = self.helpers.get_current_dto_from_payload(&input.dto).await?;
        let role = self.helpers.get_role_from_dto(&dto).await?;
        let message_type = input._type.clone();
        let current_state = self.helpers.get_state_from_dto(&dto).await?;
        let current_state_attribute = self.helpers.get_state_attribute_from_dto(&dto).await?;
        self.payload_validator.validate_with_json_schema(&input.dto).await?;
        self.payload_validator.validate_identifiers_as_urn(&input.dto).await?;
        self.payload_validator.validate_correlation(&input.dto, &dto).await?;
        self.payload_validator.validate_auth(&input.dto).await?;
        self.step_transition_validator.validate_role_for_message(&role, &message_type).await?;
        self.step_transition_validator
            .validate_state_transition(&current_state, &message_type)
            .await?;
        self.step_transition_validator
            .validate_state_attribute_transition(
                &current_state,
                &current_state_attribute,
                &message_type,
                &role,
            )
            .await?;
        Ok(())
    }

    async fn transfer_termination_rpc(
        &self,
        input: &RpcTransferTerminationMessageDto,
    ) -> anyhow::Result<()> {
        let input: TransferProcessMessageWrapper<TransferTerminationMessageDto> =
            input.clone().into();
        let dto = self.helpers.get_current_dto_from_payload(&input.dto).await?;
        let role = self.helpers.get_role_from_dto(&dto).await?;
        let message_type = input._type.clone();
        let current_state = self.helpers.get_state_from_dto(&dto).await?;
        let current_state_attribute = self.helpers.get_state_attribute_from_dto(&dto).await?;
        self.payload_validator.validate_with_json_schema(&input.dto).await?;
        self.payload_validator.validate_identifiers_as_urn(&input.dto).await?;
        self.payload_validator.validate_correlation(&input.dto, &dto).await?;
        self.payload_validator.validate_auth(&input.dto).await?;
        self.step_transition_validator.validate_role_for_message(&role, &message_type).await?;
        self.step_transition_validator
            .validate_state_transition(&current_state, &message_type)
            .await?;
        self.step_transition_validator
            .validate_state_attribute_transition(
                &current_state,
                &current_state_attribute,
                &message_type,
                &role,
            )
            .await?;
        Ok(())
    }
}
