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
use crate::entities::transfer_process::{
    EditTransferProcessDto, TransferAgentProcessesTrait, TransferProcessDto,
};
use crate::http::common::parse_urn;
use crate::protocols::dsp::persistence::{
    create_process_record, CreateProcessInput, TransferPersistenceTrait,
};
use crate::protocols::dsp::protocol_types::{
    TransferProcessMessageTrait, TransferProcessMessageType, TransferProcessState,
    TransferStateAttribute,
};
use anyhow::bail;
use rainbow_common::config::types::roles::RoleConfig;
use rainbow_common::errors::CommonErrors;
use rainbow_common::errors::ErrorLog;
use std::str::FromStr;
use std::sync::Arc;
use tracing::error;
use urn::Urn;
use ymir::errors::{Errors, Outcome};
// ─── Service ─────────────────────────────────────────────────────────────────

/// Persistence service for the outbound RPC path.
///
/// Used when the local agent acts as **Consumer**, sending transfer messages to a
/// remote Provider via the internal RPC interface.  Messages are recorded as
/// `OUTBOUND` and state attributes are attributed to the local party (Consumer).
pub struct TransferPersistenceForRpcService {
    pub transfer_message_service: Arc<dyn TransferAgentMessagesTrait>,
    pub transfer_process_service: Arc<dyn TransferAgentProcessesTrait>,
}

impl TransferPersistenceForRpcService {
    pub fn new(
        transfer_message_service: Arc<dyn TransferAgentMessagesTrait>,
        transfer_process_service: Arc<dyn TransferAgentProcessesTrait>,
    ) -> Self {
        Self { transfer_message_service, transfer_process_service }
    }
}

#[async_trait::async_trait]
impl TransferPersistenceTrait for TransferPersistenceForRpcService {
    async fn get_transfer_process_service(&self) -> Outcome<Arc<dyn TransferAgentProcessesTrait>> {
        Ok(self.transfer_process_service.clone())
    }

    async fn get_transfer_message_service(&self) -> Outcome<Arc<dyn TransferAgentMessagesTrait>> {
        Ok(self.transfer_message_service.clone())
    }

    async fn fetch_process(&self, id: &str) -> Outcome<TransferProcessDto> {
        let urn = parse_urn(id).unwrap();
        // RPC clients send the consumerPid; resolve by any key value in the identifiers map.
        self.transfer_process_service.get_transfer_process_by_key_value(&urn).await
    }

    async fn create_process(&self, input: CreateProcessInput) -> Outcome<TransferProcessDto> {
        let id = input.id.unwrap_or_else(|| {
            Urn::from_str(&format!("urn:transfer-process:{}", uuid::Uuid::new_v4())).unwrap()
        });
        create_process_record(
            &self.transfer_process_service,
            &self.transfer_message_service,
            id,
            input.protocol,
            input.direction,
            &input.associated_agent_peer,
            input.provider_pid,
            input.provider_address,
            input.payload_dto,
            input.payload_value,
        )
        .await
    }

    /// Records an outbound state transition (message sent by the local agent).
    ///
    /// The state attribute is attributed to the *local* party: a Consumer records
    /// actions as `ByConsumer`, and a Provider as `ByProvider`.
    async fn update_process(
        &self,
        id: &str,
        payload_dto: Arc<dyn TransferProcessMessageTrait>,
        payload_value: serde_json::Value,
    ) -> Outcome<TransferProcessDto> {
        let urn_id = Urn::from_str(id).expect("Failed to parse process URN");
        let message_type = payload_dto.get_message();
        let new_state = TransferProcessState::from(message_type.clone());

        // RPC uses the primary process ID (not a peer identifier) for lookups.
        let process = self.transfer_process_service.get_transfer_process_by_id(&urn_id).await?;
        let process_urn = Urn::from_str(process.inner.id.as_str())?;

        let role = process
            .inner
            .role
            .parse::<RoleConfig>()
            .map_err(|e| Errors::crazy(format!("Not able to parse RoleConfig: {e}"), None))?;
        let prev_attr = process
            .inner
            .state_attribute
            .unwrap_or(TransferStateAttribute::OnRequest.to_string())
            .parse::<TransferStateAttribute>()
            .map_err(|e| Errors::crazy(format!("Not able to parse TransferStateAttribute: {e}"), None))?;

        let new_attr = resolve_outbound_state_attribute(&message_type, &prev_attr, &role)?;

        let mut updated = self
            .transfer_process_service
            .put_transfer_process(
                &process_urn,
                &EditTransferProcessDto {
                    state: Some(new_state.to_string()),
                    state_attribute: Some(new_attr.to_string()),
                    properties: None,
                    error_details: None,
                    identifiers: None,
                },
            )
            .await?;

        let msg = self
            .transfer_message_service
            .create_transfer_message(&NewTransferMessageDto {
                id: None,
                transfer_agent_process_id: process_urn,
                direction: "OUTBOUND".to_string(),
                protocol: "DSP".to_string(),
                message_type: message_type.to_string(),
                state_transition_from: process.inner.state,
                state_transition_to: new_state.to_string(),
                payload: Some(payload_value),
            })
            .await?;

        updated.messages.push(msg.inner);
        Ok(updated)
    }
}

// ─── State attribute helpers ──────────────────────────────────────────────────

/// Derives the new state attribute for a message **sent** by the local agent.
///
/// For outbound messages the local agent is the initiator, so the attribute
/// reflects the *same* party as the local role.
///
/// Exception: mirroring the inbound case, a Start message while still in
/// `OnRequest` phase retains the `OnRequest` attribute.
fn resolve_outbound_state_attribute(
    message_type: &TransferProcessMessageType,
    current: &TransferStateAttribute,
    local_role: &RoleConfig,
) -> Outcome<TransferStateAttribute> {
    match message_type {
        TransferProcessMessageType::TransferStartMessage => match current {
            TransferStateAttribute::OnRequest => Ok(TransferStateAttribute::OnRequest),
            _ => own_attribute(local_role),
        },
        _ => own_attribute(local_role),
    }
}

/// Returns the state attribute that represents the *local* role itself.
fn own_attribute(local_role: &RoleConfig) -> Outcome<TransferStateAttribute> {
    match local_role {
        RoleConfig::Provider => Ok(TransferStateAttribute::ByProvider),
        RoleConfig::Consumer => Ok(TransferStateAttribute::ByConsumer),
        _ => {
            return Err(Errors::crazy("Unknown role when resolving state attribute", None));
        }
    }
}
