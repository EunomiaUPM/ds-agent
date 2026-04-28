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

use crate::entities::transfer_messages::{NewTransferMessageDto, TransferAgentMessagesTrait};
use crate::entities::transfer_process::{
    EditTransferProcessDto, TransferAgentProcessesTrait, TransferProcessDto,
};
use crate::protocols::dsp::context::DspTransferContext;
use crate::protocols::dsp::persistence::{create_process_record, TransferPersistenceTrait};
use crate::protocols::dsp::protocol_types::{
    TransferProcessMessageTrait, TransferProcessMessageType, TransferProcessState,
    TransferStateAttribute,
};
use common::config::types::roles::RoleConfig;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::{Errors, Outcome};


/// Persistence service for the inbound DSP protocol path.
///
/// Used when the local agent acts as **Provider**, receiving transfer messages
/// from a remote Consumer over the DSP wire protocol.  Messages are recorded
/// as `INBOUND` and state attributes are attributed to the peer (Consumer).
pub struct TransferPersistenceForProtocolService {
    pub transfer_message_service: Arc<dyn TransferAgentMessagesTrait>,
    pub transfer_process_service: Arc<dyn TransferAgentProcessesTrait>,
}

impl TransferPersistenceForProtocolService {
    pub fn new(
        transfer_message_service: Arc<dyn TransferAgentMessagesTrait>,
        transfer_process_service: Arc<dyn TransferAgentProcessesTrait>,
    ) -> Self {
        Self {
            transfer_message_service,
            transfer_process_service,
        }
    }
}

#[async_trait::async_trait]
impl TransferPersistenceTrait for TransferPersistenceForProtocolService {
    async fn get_transfer_process_service(&self) -> Outcome<Arc<dyn TransferAgentProcessesTrait>> {
        Ok(self.transfer_process_service.clone())
    }

    async fn get_transfer_message_service(&self) -> Outcome<Arc<dyn TransferAgentMessagesTrait>> {
        Ok(self.transfer_message_service.clone())
    }

    async fn fetch_process(&self, id: &str) -> Outcome<TransferProcessDto> {
        let urn = Urn::from_str(id)?;
        // Resolve by identifier value; DSP peers may send either consumerPid or providerPid.
        self.transfer_process_service
            .get_transfer_process_by_key_value(&urn)
            .await
    }

    async fn create_process(
        &self,
        ctx: &DspTransferContext,
        payload_dto: Arc<dyn TransferProcessMessageTrait>,
        payload_value: serde_json::Value,
    ) -> Outcome<TransferProcessDto> {
        let id = ctx.local_process_id.clone().unwrap_or_else(|| {
            Urn::from_str(&format!("urn:transfer-process:{}", uuid::Uuid::new_v4())).unwrap()
        });
        create_process_record(
            &self.transfer_process_service,
            &self.transfer_message_service,
            id,
            "DSP",
            "INBOUND",
            &ctx.associated_peer,
            ctx.provider_pid.clone(),
            ctx.provider_address.clone(),
            payload_dto,
            payload_value,
        )
        .await
    }

    /// Records an inbound state transition (message received from the peer).
    ///
    /// The state attribute is attributed to the *peer* party: a Provider receives
    /// actions initiated `ByConsumer`, and vice versa.
    /// Resolves the process by `ctx.peer_pid` (the URL path identifier).
    async fn update_process(
        &self,
        ctx: &DspTransferContext,
        payload_dto: Arc<dyn TransferProcessMessageTrait>,
        payload_value: serde_json::Value,
    ) -> Outcome<TransferProcessDto> {
        let urn_id = ctx
            .peer_pid
            .as_ref()
            .ok_or_else(|| Errors::crazy("peer_pid required for protocol update_process", None))?;

        let message_type = payload_dto.get_message();
        let new_state = TransferProcessState::from(message_type.clone());

        let process = self
            .transfer_process_service
            .get_transfer_process_by_key_value(urn_id)
            .await?;
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
            .map_err(|e| {
                Errors::crazy(
                    format!("Not able to parse TransferStateAttribute: {e}"),
                    None,
                )
            })?;

        let new_attr = resolve_inbound_state_attribute(&message_type, &prev_attr, &role)?;

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
                direction: "INBOUND".to_string(),
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

/// Derives the new state attribute for a message **received** from the peer.
fn resolve_inbound_state_attribute(
    message_type: &TransferProcessMessageType,
    current: &TransferStateAttribute,
    local_role: &RoleConfig,
) -> Outcome<TransferStateAttribute> {
    match message_type {
        TransferProcessMessageType::TransferStartMessage => match current {
            TransferStateAttribute::OnRequest => Ok(TransferStateAttribute::OnRequest),
            _ => peer_attribute(local_role),
        },
        _ => peer_attribute(local_role),
    }
}

fn peer_attribute(local_role: &RoleConfig) -> Outcome<TransferStateAttribute> {
    match local_role {
        RoleConfig::Provider => Ok(TransferStateAttribute::ByConsumer),
        RoleConfig::Consumer => Ok(TransferStateAttribute::ByProvider),
        _ => Err(Errors::crazy(
            "Unknown role when resolving state attribute",
            None,
        )),
    }
}
