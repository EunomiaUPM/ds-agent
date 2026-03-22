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

use crate::entities::agreement::{
    AgreementDto, EditAgreementDto, NegotiationAgentAgreementsTrait, NewAgreementDto,
};
use crate::entities::negotiation_message::{
    NegotiationAgentMessagesTrait, NegotiationMessageDto, NewNegotiationMessageDto,
};
use crate::entities::negotiation_process::{
    EditNegotiationProcessDto, NegotiationAgentProcessesTrait, NegotiationProcessDto,
    NewNegotiationProcessDto,
};
use crate::entities::offer::{NegotiationAgentOffersTrait, NewOfferDto, OfferDto};
use crate::protocols::dsp::orchestrator::rpc::types::RpcNegotiationProcessMessageTrait;
use crate::protocols::dsp::orchestrator::traits::orchestration_extractors::OrchestrationExtractors;
use crate::protocols::dsp::orchestrator::traits::orchestration_helpers::OrchestrationHelpers;
use crate::protocols::dsp::persistence::NegotiationRpcPersistenceTrait;
use crate::protocols::dsp::protocol_types::{
    NegotiationProcessMessageTrait, NegotiationProcessMessageType, NegotiationProcessState,
};
use common::config::types::roles::RoleConfig;
use common::dsp_common::odrl::ContractRequestMessageOfferTypes;
use common::errors::{CommonErrors, ErrorLog};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::error;
use urn::Urn;
use ymir::errors::{Errors, Outcome};

// ─── Service ─────────────────────────────────────────────────────────────────

/// Persistence service for the outbound RPC negotiation path.
///
/// Used when the local agent initiates DSP negotiation messages via the
/// internal RPC interface.  Messages are recorded as `OUTBOUND` and state
/// transitions are attributed to the local party.
///
/// Implements [`NegotiationRpcPersistenceTrait`], which the RPC orchestrator
/// holds as `Arc<dyn NegotiationRpcPersistenceTrait>` — mirroring how
/// `TransferPersistenceForRpcService` works in the transfer agent.
pub struct NegotiationPersistenceForRpcService {
    negotiation_process_service: Arc<dyn NegotiationAgentProcessesTrait>,
    negotiation_messages_service: Arc<dyn NegotiationAgentMessagesTrait>,
    offer_service: Arc<dyn NegotiationAgentOffersTrait>,
    agreement_service: Arc<dyn NegotiationAgentAgreementsTrait>,
}

impl NegotiationPersistenceForRpcService {
    pub fn new(
        negotiation_process_service: Arc<dyn NegotiationAgentProcessesTrait>,
        negotiation_messages_service: Arc<dyn NegotiationAgentMessagesTrait>,
        offer_service: Arc<dyn NegotiationAgentOffersTrait>,
        agreement_service: Arc<dyn NegotiationAgentAgreementsTrait>,
    ) -> Self {
        Self {
            negotiation_process_service,
            negotiation_messages_service,
            offer_service,
            agreement_service,
        }
    }
}

// ─── Trait implementation ─────────────────────────────────────────────────────

#[async_trait::async_trait]
impl NegotiationRpcPersistenceTrait for NegotiationPersistenceForRpcService {
    async fn fetch_process(&self, id: &str) -> Outcome<NegotiationProcessDto> {
        let urn = self.convert_str_to_urn(id)?;
        let process = self
            .negotiation_process_service
            .get_negotiation_process_by_key_value(&urn)
            .await?
            .ok_or_else(|| Errors::crazy("Process not found", None))?;
        Ok(process)
    }

    async fn fetch_last_offer_by_process(&self, id: &str) -> Outcome<OfferDto> {
        let urn = self.convert_str_to_urn(id)?;
        let offer = self
            .offer_service
            .get_last_offer_by_negotiation_process(&urn)
            .await?
            .ok_or_else(|| Errors::crazy("Offer not found", None))?;
        Ok(offer)
    }

    async fn create_new(
        &self,
        payload: &dyn RpcNegotiationProcessMessageTrait,
        request: &dyn NegotiationProcessMessageTrait,
        response: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<NegotiationProcessDto> {
        let mut process = self.create_process(payload, request, response).await?;
        let process_id = self.convert_string_to_urn(&process.inner.id)?;
        let message = self
            .create_message_with_old_state(&process_id, payload, &process, "-")
            .await?;
        let message_id = self.convert_string_to_urn(&message.inner.id)?;
        let offer = self.create_offer(&process_id, &message_id, payload).await?;
        process.messages.push(message.inner);
        process.offers.push(offer.inner);
        Ok(process)
    }

    async fn update(
        &self,
        identifier: &str,
        payload: &dyn RpcNegotiationProcessMessageTrait,
        request: &dyn NegotiationProcessMessageTrait,
        response: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<NegotiationProcessDto> {
        let process = self.fetch_process(identifier).await?;
        let process_id = self.convert_string_to_urn(&process.inner.id)?;
        let mut new_process = self
            .update_process(&process_id, payload, request, response)
            .await?;
        let message = self.create_message(&process_id, payload, &process).await?;
        new_process.messages.push(message.inner);
        Ok(new_process)
    }

    async fn update_with_offer(
        &self,
        identifier: &str,
        payload: &dyn RpcNegotiationProcessMessageTrait,
        request: &dyn NegotiationProcessMessageTrait,
        response: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<NegotiationProcessDto> {
        let process = self.fetch_process(identifier).await?;
        let process_id = self.convert_string_to_urn(&process.inner.id)?;
        let mut new_process = self
            .update_process(&process_id, payload, request, response)
            .await?;
        let message = self.create_message(&process_id, payload, &process).await?;
        let message_id = self.convert_string_to_urn(&message.inner.id)?;
        let offer = self.create_offer(&process_id, &message_id, payload).await?;
        new_process.messages.push(message.inner);
        new_process.offers.push(offer.inner);
        Ok(new_process)
    }

    async fn update_with_new_agreement(
        &self,
        identifier: &str,
        payload: &dyn RpcNegotiationProcessMessageTrait,
        request: &dyn NegotiationProcessMessageTrait,
        response: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<NegotiationProcessDto> {
        let process = self.fetch_process(identifier).await?;
        let associated_agent_peer = process.inner.associated_agent_peer.clone();
        let process_id = self.convert_string_to_urn(&process.inner.id)?;
        let mut new_process = self
            .update_process(&process_id, payload, request, response)
            .await?;
        let message = self.create_message(&process_id, payload, &process).await?;
        let message_id = self.convert_string_to_urn(&message.inner.id)?;
        let agreement = self
            .create_agreement(
                &process_id,
                &message_id,
                &associated_agent_peer,
                payload,
                request,
            )
            .await?;
        new_process.messages.push(message.inner);
        new_process.agreement = Some(agreement.inner);
        Ok(new_process)
    }

    async fn update_with_agreement(
        &self,
        identifier: &str,
        payload: &dyn RpcNegotiationProcessMessageTrait,
        request: &dyn NegotiationProcessMessageTrait,
        response: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<NegotiationProcessDto> {
        let process = self.fetch_process(identifier).await?;
        let process_id = self.convert_string_to_urn(&process.inner.id)?;
        let mut new_process = self
            .update_process(&process_id, payload, request, response)
            .await?;
        let message = self.create_message(&process_id, payload, &process).await?;
        let message_id = self.convert_string_to_urn(&message.inner.id)?;
        let agreement = self
            .activate_agreement(&process_id, &message_id, payload)
            .await?;
        new_process.messages.push(message.inner);
        new_process.agreement = Some(agreement.inner);
        Ok(new_process)
    }
}

// ─── Private helpers ──────────────────────────────────────────────────────────

impl NegotiationPersistenceForRpcService {
    async fn create_process(
        &self,
        message: &dyn RpcNegotiationProcessMessageTrait,
        request: &dyn NegotiationProcessMessageTrait,
        response: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<NegotiationProcessDto> {
        let id = self.create_entity_urn("negotiation-process")?;
        let agent_peer = message.get_associated_agent_peer().unwrap_or_default();
        let message_type = self.get_rpc_message_safely(message)?;
        let state: NegotiationProcessState = message_type.clone().into();
        let callback = self.get_rpc_provider_address_safely(message)?;
        let role = self.get_role_from_message_type(&message_type)?;
        let mut identifiers = HashMap::new();
        identifiers.insert(
            "consumerPid".to_string(),
            self.get_dsp_consumer_pid_safely(response)?.to_string(),
        );
        identifiers.insert(
            "providerPid".to_string(),
            self.get_dsp_provider_pid_safely(response)?.to_string(),
        );

        let new_process = self
            .negotiation_process_service
            .create_negotiation_process(&NewNegotiationProcessDto {
                id: Some(id),
                state: state.to_string(),
                state_attribute: None,
                associated_agent_peer: agent_peer,
                protocol: "DSP".to_string(),
                callback_address: Some(callback),
                role: role.to_string(),
                properties: None,
                identifiers: Some(identifiers),
            })
            .await?;

        Ok(new_process)
    }

    async fn create_message(
        &self,
        process_id: &Urn,
        message: &dyn RpcNegotiationProcessMessageTrait,
        process: &NegotiationProcessDto,
    ) -> Outcome<NegotiationMessageDto> {
        let old_state = process.inner.state.clone();
        self.create_message_with_old_state(process_id, message, process, &old_state)
            .await
    }

    async fn create_message_with_old_state(
        &self,
        process_id: &Urn,
        message: &dyn RpcNegotiationProcessMessageTrait,
        _process: &NegotiationProcessDto,
        old_state: &str,
    ) -> Outcome<NegotiationMessageDto> {
        let id = self.create_entity_urn("negotiation-message")?;
        let message_type = self.get_rpc_message_safely(message)?;
        let state: NegotiationProcessState = message_type.clone().into();
        let mut payload_json = message.as_json();

        // Wrap with DSP envelope (@context and @type)
        if let serde_json::Value::Object(ref mut map) = payload_json {
            map.insert(
                "@context".to_string(),
                serde_json::json!(["https://w3id.org/dspace/2025/1/context.jsonld"]),
            );
            map.insert(
                "@type".to_string(),
                serde_json::Value::String(message_type.to_string()),
            );
        }

        let new_message = self
            .negotiation_messages_service
            .create_negotiation_message(&NewNegotiationMessageDto {
                id: Some(id),
                negotiation_agent_process_id: process_id.clone(),
                direction: "OUTBOUND".to_string(),
                protocol: "DSP".to_string(),
                message_type: message_type.to_string(),
                state_transition_from: old_state.to_string(),
                state_transition_to: state.to_string(),
                payload: payload_json,
            })
            .await?;
        Ok(new_message)
    }

    async fn create_offer(
        &self,
        process_id: &Urn,
        message_id: &Urn,
        message: &dyn RpcNegotiationProcessMessageTrait,
    ) -> Outcome<OfferDto> {
        let id = self.create_entity_urn("offer")?;
        let offer_content = self.get_rpc_offer_safely(message)?;
        let offer_id = match &offer_content {
            ContractRequestMessageOfferTypes::OfferMessage(m) => &m.id,
            ContractRequestMessageOfferTypes::OfferId(i) => &i.id,
        }
        .to_string();

        let new_offer = self
            .offer_service
            .create_offer(&NewOfferDto {
                id: Some(id),
                negotiation_agent_process_id: process_id.clone(),
                negotiation_agent_message_id: message_id.clone(),
                offer_id,
                offer_content: serde_json::to_value(offer_content)?,
            })
            .await?;
        Ok(new_offer)
    }

    async fn create_agreement(
        &self,
        pid: &Urn,
        mid: &Urn,
        _peer: &str,
        _message: &dyn RpcNegotiationProcessMessageTrait,
        request: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<AgreementDto> {
        let agreement = self.get_dsp_agreement_safely(request)?;
        let id = agreement.id.clone();
        let target = agreement.clone().target;
        let agr = self
            .agreement_service
            .create_agreement(&NewAgreementDto {
                id: Some(id),
                negotiation_agent_process_id: pid.clone(),
                negotiation_agent_message_id: mid.clone(),
                consumer_participant_id: agreement.assignee.clone(),
                provider_participant_id: agreement.assigner.clone(),
                agreement_content: serde_json::to_value(agreement).unwrap(),
                target,
            })
            .await?;
        Ok(agr)
    }

    async fn activate_agreement(
        &self,
        pid: &Urn,
        _mid: &Urn,
        _message: &dyn RpcNegotiationProcessMessageTrait,
    ) -> Outcome<AgreementDto> {
        let fetching_agreement = self
            .agreement_service
            .get_agreement_by_negotiation_process(pid)
            .await?
            .ok_or_else(|| Errors::crazy("Agreement not found", None))?;
        let agreement_urn = self.convert_string_to_urn(&fetching_agreement.inner.id)?;
        let agreement = self
            .agreement_service
            .put_agreement(
                &agreement_urn,
                &EditAgreementDto {
                    state: Some("ACTIVE".to_string()),
                },
            )
            .await?;
        Ok(agreement)
    }

    async fn update_process(
        &self,
        pid: &Urn,
        payload: &dyn RpcNegotiationProcessMessageTrait,
        request: &dyn NegotiationProcessMessageTrait,
        _response: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<NegotiationProcessDto> {
        let message_type = self.get_dsp_message_safely(request)?;
        let state: NegotiationProcessState = message_type.clone().into();
        let process = self
            .negotiation_process_service
            .put_negotiation_process(
                pid,
                &EditNegotiationProcessDto {
                    state: Some(state.to_string()),
                    state_attribute: None,
                    properties: None,
                    error_details: None,
                    identifiers: None,
                },
            )
            .await?;
        Ok(process)
    }
}

// ─── Blanket trait impls ──────────────────────────────────────────────────────

impl OrchestrationHelpers for NegotiationPersistenceForRpcService {}

impl OrchestrationExtractors for NegotiationPersistenceForRpcService {
    fn get_role_from_message_type(
        &self,
        message: &NegotiationProcessMessageType,
    ) -> Outcome<RoleConfig> {
        match message {
            NegotiationProcessMessageType::NegotiationRequestMessage => Ok(RoleConfig::Consumer),
            NegotiationProcessMessageType::NegotiationOfferMessage => Ok(RoleConfig::Provider),
            _ => {
                let err = CommonErrors::parse_new("Message not allowed here");
                error!("{}", err.log());
                return Err(Errors::parse(err.to_string().as_str(), None));
            }
        }
    }
}
