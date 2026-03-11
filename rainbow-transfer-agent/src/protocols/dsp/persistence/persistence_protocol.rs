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

use crate::entities::transfer_messages::{NewTransferMessageDto, TransferAgentMessagesTrait};
use crate::entities::transfer_process::TransferProcessDto;
use crate::entities::transfer_process::{
    EditTransferProcessDto, NewTransferProcessDto, TransferAgentProcessesTrait,
};
use crate::protocols::dsp::persistence::TransferPersistenceTrait;
use crate::protocols::dsp::protocol_types::{
    TransferProcessMessageTrait, TransferProcessMessageType, TransferProcessState,
    TransferStateAttribute,
};
use crate::protocols::dsp::transfer_types::TransferState;
use anyhow::bail;
use rainbow_common::config::types::roles::RoleConfig;
use rainbow_common::errors::CommonErrors;
use rainbow_common::errors::ErrorLog;
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tracing::error;
use urn::Urn;

pub struct TransferPersistenceForProtocolService {
    pub transfer_message_service: Arc<dyn TransferAgentMessagesTrait>,
    pub transfer_process_service: Arc<dyn TransferAgentProcessesTrait>,
}

impl TransferPersistenceForProtocolService {
    pub fn new(
        transfer_message_service: Arc<dyn TransferAgentMessagesTrait>,
        transfer_process_service: Arc<dyn TransferAgentProcessesTrait>,
    ) -> Self {
        Self { transfer_message_service, transfer_process_service }
    }
}

#[async_trait::async_trait]
impl TransferPersistenceTrait for TransferPersistenceForProtocolService {
    async fn get_transfer_process_service(
        &self,
    ) -> anyhow::Result<Arc<dyn TransferAgentProcessesTrait>> {
        Ok(self.transfer_process_service.clone())
    }

    async fn get_transfer_message_service(
        &self,
    ) -> anyhow::Result<Arc<dyn TransferAgentMessagesTrait>> {
        Ok(self.transfer_message_service.clone())
    }
    async fn fetch_process(&self, id: &str) -> anyhow::Result<TransferProcessDto> {
        let urn = Urn::from_str(id)?;
        // get by value in identifiers table (since dsp uses this identifiers)
        let transfer_process =
            self.transfer_process_service.get_transfer_process_by_key_value(&urn).await.map_err(
                |_e| {
                    let err = CommonErrors::missing_resource_new(
                        urn.to_string().as_str(),
                        "Process service not found",
                    );
                    error!("{}", err.log());
                    err
                },
            )?;
        Ok(transfer_process)
    }

    async fn create_process(
        &self,
        protocol: &str,
        direction: &str,
        associated_agent_peer: &str,
        provider_pid: Option<Urn>,
        provider_address: Option<String>,
        payload_dto: Arc<dyn TransferProcessMessageTrait>,
        payload_value: serde_json::Value,
    ) -> anyhow::Result<TransferProcessDto> {
        // create id
        let transfer_process_id =
            Urn::from_str(format!("urn:transfer-process:{}", uuid::Uuid::new_v4()).as_str())?;
        let transfer_process = self
            .create_process_with_id(
                transfer_process_id,
                protocol,
                direction,
                associated_agent_peer,
                provider_pid,
                provider_address,
                payload_dto,
                payload_value,
            )
            .await?;
        Ok(transfer_process)
    }

    async fn create_process_with_id(
        &self,
        id: Urn,
        protocol: &str,
        direction: &str,
        associated_agent_peer: &str,
        provider_pid: Option<Urn>,
        provider_address: Option<String>,
        payload_dto: Arc<dyn TransferProcessMessageTrait>,
        payload_value: Value,
    ) -> anyhow::Result<TransferProcessDto> {
        // get from payload
        let consumer_pid = payload_dto.get_consumer_pid().unwrap(); // always
        let format = payload_dto.get_format().unwrap();
        let agreement_id = payload_dto.get_agreement_id().unwrap();
        let message_type = payload_dto.get_message();
        // create dsp compliant identifiers
        let mut identifiers = HashMap::new();
        let role = if direction == "INBOUND" { "Provider" } else { "Consumer" };
        if direction == "INBOUND" {
            let provider_pid = format!("urn:provider-pid:{}", uuid::Uuid::new_v4());
            identifiers.insert("providerPid".to_string(), provider_pid);
        } else {
            identifiers.insert("providerPid".to_string(), provider_pid.unwrap().to_string());
        }
        identifiers.insert("consumerPid".to_string(), consumer_pid.to_string());
        // create callback address
        let callback_address =
            provider_address.unwrap_or(payload_dto.get_callback_address().unwrap());
        // create entities
        let mut transfer_process = self
            .transfer_process_service
            .create_transfer_process(&NewTransferProcessDto {
                id: Some(id.clone()),
                state: TransferState::REQUESTED.to_string(),
                associated_agent_peer: associated_agent_peer.to_string(),
                protocol: protocol.to_string(),
                transfer_direction: format,
                agreement_id,
                callback_address: Some(callback_address),
                role: role.to_string(),
                state_attribute: Some(TransferStateAttribute::OnRequest.to_string()),
                properties: None,
                identifiers: Some(identifiers),
            })
            .await?;
        let transfer_message = self
            .transfer_message_service
            .create_transfer_message(&NewTransferMessageDto {
                id: None,
                transfer_agent_process_id: id.clone(),
                direction: direction.to_string(),
                protocol: protocol.to_string(),
                message_type: message_type.to_string(),
                state_transition_from: "-".to_string(),
                state_transition_to: TransferState::REQUESTED.to_string(),
                payload: Some(payload_value),
            })
            .await?;
        transfer_process.messages.push(transfer_message.inner);
        Ok(transfer_process)
    }

    async fn update_process(
        &self,
        id: &str,
        payload_dto: Arc<dyn TransferProcessMessageTrait>,
        payload_value: serde_json::Value,
    ) -> anyhow::Result<TransferProcessDto> {
        let urn_id = Urn::from_str(id).expect("Failed to parse urnID");
        let message_type = payload_dto.get_message();
        // new state request
        let new_state: TransferProcessState = TransferProcessState::from(message_type.clone());
        // current state
        let transfer_process =
            self.transfer_process_service.get_transfer_process_by_key_value(&urn_id).await?;
        // update
        let transfer_process_urn = Urn::from_str(transfer_process.inner.id.as_str())?;
        // role
        let role = transfer_process.inner.role.parse::<RoleConfig>()?;
        let state_attribute = transfer_process
            .inner
            .state_attribute
            .unwrap_or(TransferStateAttribute::OnRequest.to_string())
            .parse::<TransferStateAttribute>()?;
        // new state attribute
        // logical semaphore for avoiding consumer to start provider's suspension and viceversa
        let new_state_attribute = match &message_type {
            TransferProcessMessageType::TransferStartMessage => match &state_attribute {
                TransferStateAttribute::OnRequest => TransferStateAttribute::OnRequest,
                _ => match &role {
                    RoleConfig::Provider => TransferStateAttribute::ByConsumer,
                    RoleConfig::Consumer => TransferStateAttribute::ByProvider,
                    _ => {
                        let err = CommonErrors::parse_new("Process service not found");
                        error!("{}", err.log());
                        bail!(err)
                    }
                },
            },
            _ => match &role {
                RoleConfig::Provider => TransferStateAttribute::ByConsumer,
                RoleConfig::Consumer => TransferStateAttribute::ByProvider,
                _ => {
                    let err = CommonErrors::parse_new("Process service not found");
                    error!("{}", err.log());
                    bail!(err)
                }
            },
        };

        let mut new_transfer_process = self
            .transfer_process_service
            .put_transfer_process(
                &transfer_process_urn,
                &EditTransferProcessDto {
                    state: Some(new_state.to_string()),
                    state_attribute: Some(new_state_attribute.to_string()),
                    properties: None,
                    error_details: None,
                    identifiers: None,
                },
            )
            .await?;
        let message = self
            .transfer_message_service
            .create_transfer_message(&NewTransferMessageDto {
                id: None,
                transfer_agent_process_id: transfer_process_urn.clone(),
                direction: "INBOUND".to_string(),
                protocol: "DSP".to_string(),
                message_type: message_type.to_string(),
                state_transition_from: transfer_process.inner.state.to_string(),
                state_transition_to: new_state.to_string(),
                payload: Some(payload_value.clone()),
            })
            .await?;
        new_transfer_process.messages.push(message.inner);
        Ok(new_transfer_process)
    }
}
