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

pub(crate) mod persistence_protocol;
pub(crate) mod persistence_rpc;

use crate::entities::transfer_messages::{NewTransferMessageDto, TransferAgentMessagesTrait};
use crate::entities::transfer_process::{
    NewTransferProcessDto, TransferAgentProcessesTrait, TransferProcessDto,
};
use crate::protocols::dsp::context::DspTransferContext;
use crate::protocols::dsp::protocol_types::{TransferProcessMessageTrait, TransferStateAttribute};
use crate::protocols::dsp::transfer_types::TransferState;
use std::collections::HashMap;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::Outcome;

#[async_trait::async_trait]
#[allow(unused)]
pub trait TransferPersistenceTrait: Send + Sync {
    async fn get_transfer_process_service(&self) -> Outcome<Arc<dyn TransferAgentProcessesTrait>>;

    async fn get_transfer_message_service(&self) -> Outcome<Arc<dyn TransferAgentMessagesTrait>>;

    async fn fetch_process(&self, id: &str) -> Outcome<TransferProcessDto>;

    async fn create_process(
        &self,
        ctx: &DspTransferContext,
        payload_dto: Arc<dyn TransferProcessMessageTrait>,
        payload_value: serde_json::Value,
    ) -> Outcome<TransferProcessDto>;

    async fn update_process(
        &self,
        ctx: &DspTransferContext,
        payload_dto: Arc<dyn TransferProcessMessageTrait>,
        payload_value: serde_json::Value,
    ) -> Outcome<TransferProcessDto>;
}

pub(crate) async fn create_process_record(
    process_svc: &Arc<dyn TransferAgentProcessesTrait>,
    message_svc: &Arc<dyn TransferAgentMessagesTrait>,
    id: Urn,
    protocol: &str,
    protocol_direction: &str,
    associated_agent_peer: &str,
    provider_pid: Option<Urn>,
    provider_address: Option<String>,
    payload_dto: Arc<dyn TransferProcessMessageTrait>,
    payload_value: serde_json::Value,
) -> Outcome<TransferProcessDto> {
    // Extract fields from the typed message payload.
    let consumer_pid = payload_dto.get_consumer_pid().unwrap(); // always present on TransferRequest
    let format = payload_dto.get_format().unwrap();
    let agreement_id = payload_dto.get_agreement_id().unwrap();
    let message_type = payload_dto.get_message();
    let callback_address =
        provider_address.unwrap_or_else(|| payload_dto.get_callback_address().unwrap());

    // The local role and providerPid assignment depend on who initiated the transfer.
    // INBOUND (Provider): we generate the providerPid ourselves.
    // OUTBOUND (Consumer): the providerPid was received from the peer.
    let role = if protocol_direction == "INBOUND" {
        "Provider"
    } else {
        "Consumer"
    };

    // The transfer direction based on DSP spec. If TransferRequestMessage has dataAddress defined
    // Direction is PUSH, otherwise is PULL
    let transfer_direction = match payload_dto.get_data_address() {
        Some(_) => "Push",
        None => "Pull",
    };

    // Create providerPid identifiers if INBOUND
    let mut identifiers = HashMap::new();
    if protocol_direction == "INBOUND" {
        identifiers.insert(
            "providerPid".to_string(),
            format!("urn:provider-pid:{}", uuid::Uuid::new_v4()),
        );
    } else {
        identifiers.insert("providerPid".to_string(), provider_pid.unwrap().to_string());
    }
    // In any case INBOUND or OUTBOUND consumerPid must be created
    identifiers.insert("consumerPid".to_string(), consumer_pid.to_string());

    // Persist Process
    let mut transfer_process = process_svc
        .create_transfer_process(&NewTransferProcessDto {
            id: Some(id.clone()),
            state: TransferState::REQUESTED.to_string(),
            associated_agent_peer: associated_agent_peer.to_string(),
            protocol: protocol.to_string(),
            connector_instance_id: format,
            transfer_direction: transfer_direction.to_string(),
            agreement_id,
            callback_address: Some(callback_address),
            role: role.to_string(),
            state_attribute: Some(TransferStateAttribute::OnRequest.to_string()),
            properties: None,
            identifiers: Some(identifiers),
        })
        .await?;

    // Create a message
    let message = message_svc
        .create_transfer_message(&NewTransferMessageDto {
            id: None,
            transfer_agent_process_id: id,
            direction: protocol_direction.to_string(),
            protocol: protocol.to_string(),
            message_type: message_type.to_string(),
            state_transition_from: "-".to_string(),
            state_transition_to: TransferState::REQUESTED.to_string(),
            payload: Some(payload_value),
        })
        .await?;

    transfer_process.messages.push(message.inner);
    Ok(transfer_process)
}
