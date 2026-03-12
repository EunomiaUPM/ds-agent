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

use crate::entities::negotiation_process::NegotiationProcessDto;
use crate::protocols::dsp::orchestrator::rpc::persistence::OrchestrationPersistenceForRpc;
use crate::protocols::dsp::orchestrator::rpc::types::{
    RpcNegotiationAgreementMessageDto, RpcNegotiationEventAcceptedMessageDto,
    RpcNegotiationEventFinalizedMessageDto, RpcNegotiationMessageDto,
    RpcNegotiationOfferInitMessageDto, RpcNegotiationOfferMessageDto,
    RpcNegotiationProcessMessageTrait, RpcNegotiationRequestInitMessageDto,
    RpcNegotiationRequestMessageDto, RpcNegotiationTerminationMessageDto,
    RpcNegotiationVerificationMessageDto,
};
use crate::protocols::dsp::orchestrator::rpc::RPCOrchestratorTrait;
use crate::protocols::dsp::orchestrator::traits::orchestration_helpers::OrchestrationHelpers;
use crate::protocols::dsp::persistence::NegotiationPersistenceTrait;
use crate::protocols::dsp::protocol_types::{
    NegotiationAckMessageDto, NegotiationAgreementMessageDto, NegotiationEventMessageDto,
    NegotiationEventType, NegotiationOfferInitMessageDto, NegotiationOfferMessageDto,
    NegotiationProcessMessageTrait, NegotiationProcessMessageWrapper,
    NegotiationRequestInitMessageDto, NegotiationRequestMessageDto,
    NegotiationTerminationMessageDto, NegotiationVerificationMessageDto,
};
use crate::protocols::dsp::validator::traits::validation_rpc_steps::ValidationRpcSteps;
use rainbow_common::config::services::ContractsConfig;
use rainbow_common::config::types::roles::RoleConfig;
use rainbow_common::dsp_common::context_field::ContextField;
use rainbow_common::dsp_common::odrl::{OdrlAgreement, OdrlMessageOffer, OdrlTypes};
use rainbow_common::facades::ssi_auth_facade::MatesFacadeTrait;
use rainbow_common::http_client::HttpClient;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;

// ─── Peer context ─────────────────────────────────────────────────────────────

/// Resolved routing state for a continuation message within an existing negotiation.
///
/// Most RPC operations (everything after the initial request / offer) require
/// fetching the current process and deriving the peer URL from the stored
/// identifiers.  This struct bundles that information to avoid prop-drilling.
struct NegotiationPeerContext {
    /// Full process record as stored in the database.
    process: NegotiationProcessDto,
    /// The identifier placed in the outgoing URL (opposite party's PID).
    peer_identifier: String,
    /// Full callback address of the remote peer.
    peer_address: String,
}

// ─── Service ─────────────────────────────────────────────────────────────────

/// RPC orchestrator for outbound negotiation operations.
///
/// Translates internal RPC requests into DSP protocol messages, sends them to
/// the remote peer over HTTP, and persists the resulting state transitions.
#[allow(unused)]
pub struct RPCOrchestratorService {
    validator: Arc<dyn ValidationRpcSteps>,
    persistence_service: Arc<OrchestrationPersistenceForRpc>,
    _config: Arc<ContractsConfig>,
    http_client: Arc<HttpClient>,
    mates_service: Arc<dyn MatesFacadeTrait>,
}

impl RPCOrchestratorService {
    pub fn new(
        validator: Arc<dyn ValidationRpcSteps>,
        persistence_service: Arc<OrchestrationPersistenceForRpc>,
        _config: Arc<ContractsConfig>,
        http_client: Arc<HttpClient>,
        mates_service: Arc<dyn MatesFacadeTrait>,
    ) -> RPCOrchestratorService {
        RPCOrchestratorService {
            validator,
            persistence_service,
            _config,
            http_client,
            mates_service,
        }
    }
}

impl OrchestrationHelpers for RPCOrchestratorService {}

// ─── Trait implementation ─────────────────────────────────────────────────────

#[async_trait::async_trait]
impl RPCOrchestratorTrait for RPCOrchestratorService {
    /// Sends an initial `ContractRequestMessage` to the provider (Consumer-initiated flow).
    async fn setup_negotiation_request_init_rpc(
        &self,
        input: &RpcNegotiationRequestInitMessageDto,
    ) -> anyhow::Result<RpcNegotiationMessageDto<RpcNegotiationRequestInitMessageDto>> {
        self.validator.negotiation_request_init_rpc(input).await?;

        let provider_address = self.get_rpc_provider_address_safely(input)?;
        let peer_url = format!("{}/negotiations/request", provider_address);
        let request_body: NegotiationProcessMessageWrapper<NegotiationRequestInitMessageDto> =
            input.clone().into();

        let associated_peer = input.get_associated_agent_peer().unwrap_or_default();
        self.apply_auth_token(&associated_peer).await;
        let response: NegotiationProcessMessageWrapper<NegotiationAckMessageDto> =
            self.http_client.post_json(peer_url.as_str(), &request_body).await?;

        let process =
            self.persistence_service.create_new(input, &request_body.dto, &response.dto).await?;

        Ok(RpcNegotiationMessageDto {
            request: input.clone(),
            response,
            negotiation_agent_model: process,
        })
    }

    /// Sends a `ContractRequestMessage` continuing an existing negotiation.
    async fn setup_negotiation_request_rpc(
        &self,
        input: &RpcNegotiationRequestMessageDto,
    ) -> anyhow::Result<RpcNegotiationMessageDto<RpcNegotiationRequestMessageDto>> {
        self.validator.negotiation_request_rpc(input).await?;

        let id = self.get_rpc_consumer_pid_safely(input)?.to_string();
        let ctx = self.resolve_continuation_context(&Urn::from_str(&id)?).await?;
        let peer_url = format!("{}/negotiations/{}/request", ctx.peer_address, ctx.peer_identifier);

        let request_body: NegotiationProcessMessageWrapper<NegotiationRequestMessageDto> =
            input.clone().into();
        self.apply_auth_token(&ctx.process.inner.associated_agent_peer).await;
        let response: NegotiationProcessMessageWrapper<NegotiationAckMessageDto> =
            self.http_client.post_json(peer_url.as_str(), &request_body).await?;

        let process = self
            .persistence_service
            .update_with_offer(id.as_str(), input, &request_body.dto, &response.dto)
            .await?;

        Ok(RpcNegotiationMessageDto {
            request: input.clone(),
            response,
            negotiation_agent_model: process,
        })
    }

    /// Sends an initial `ContractOfferMessage` to the consumer (Provider-initiated flow).
    async fn setup_negotiation_offer_init_rpc(
        &self,
        input: &RpcNegotiationOfferInitMessageDto,
    ) -> anyhow::Result<RpcNegotiationMessageDto<RpcNegotiationOfferInitMessageDto>> {
        self.validator.negotiation_offer_init_rpc(input).await?;

        let provider_address = self.get_rpc_provider_address_safely(input)?;
        let peer_url = format!("{}/negotiations/offers", provider_address);
        let request_body: NegotiationProcessMessageWrapper<NegotiationOfferInitMessageDto> =
            input.clone().into();

        let associated_peer = input.get_associated_agent_peer().unwrap_or_default();
        self.apply_auth_token(&associated_peer).await;
        let response: NegotiationProcessMessageWrapper<NegotiationAckMessageDto> =
            self.http_client.post_json(peer_url.as_str(), &request_body).await?;

        let process =
            self.persistence_service.create_new(input, &request_body.dto, &response.dto).await?;

        Ok(RpcNegotiationMessageDto {
            request: input.clone(),
            response,
            negotiation_agent_model: process,
        })
    }

    /// Sends a `ContractOfferMessage` continuing an existing negotiation.
    async fn setup_negotiation_offer_rpc(
        &self,
        input: &RpcNegotiationOfferMessageDto,
    ) -> anyhow::Result<RpcNegotiationMessageDto<RpcNegotiationOfferMessageDto>> {
        self.validator.negotiation_offer_rpc(input).await?;

        let id = self.get_rpc_consumer_pid_safely(input)?.to_string();
        let ctx = self.resolve_continuation_context(&Urn::from_str(&id)?).await?;
        let peer_url = format!("{}/negotiations/{}/offers", ctx.peer_address, ctx.peer_identifier);

        let request_body: NegotiationProcessMessageWrapper<NegotiationOfferMessageDto> =
            input.clone().into();
        self.apply_auth_token(&ctx.process.inner.associated_agent_peer).await;
        let response: NegotiationProcessMessageWrapper<NegotiationAckMessageDto> =
            self.http_client.post_json(peer_url.as_str(), &request_body).await?;

        let process = self
            .persistence_service
            .update_with_offer(id.as_str(), input, &request_body.dto, &response.dto)
            .await?;

        Ok(RpcNegotiationMessageDto {
            request: input.clone(),
            response,
            negotiation_agent_model: process,
        })
    }

    /// Sends a `ContractAgreementMessage` to the consumer.
    ///
    /// Enriches the agreement with `assigner`/`assignee` participant IDs resolved
    /// from the mates service before forwarding it to the peer.
    async fn setup_negotiation_agreement_rpc(
        &self,
        input: &RpcNegotiationAgreementMessageDto,
    ) -> anyhow::Result<RpcNegotiationMessageDto<RpcNegotiationAgreementMessageDto>> {
        self.validator.negotiation_agreement_rpc(input).await?;

        let id = self.get_rpc_consumer_pid_safely(input)?.to_string();
        let ctx = self.resolve_continuation_context(&Urn::from_str(&id)?).await?;
        let peer_url =
            format!("{}/negotiations/{}/agreement", ctx.peer_address, ctx.peer_identifier);

        // Fetch the last offer to copy its policy fields into the agreement.
        let last_offer = self
            .persistence_service
            .fetch_last_offer_by_process(ctx.process.inner.id.as_str())
            .await?;
        let offer = serde_json::from_value::<OdrlMessageOffer>(last_offer.inner.offer_content)?;

        // Resolve participant IDs from the mates directory.
        let assigner = self
            .mates_service
            .get_me_mate()
            .await
            .map(|m| m.participant_id)
            .unwrap_or_default();
        let assignee = self
            .mates_service
            .get_mate_by_id(ctx.process.inner.associated_agent_peer.clone())
            .await
            .map(|m| m.participant_id)
            .unwrap_or_default();

        let mut request_body: NegotiationProcessMessageWrapper<NegotiationAgreementMessageDto> =
            input.clone().into();
        request_body.dto.agreement = OdrlAgreement {
            id: self.create_entity_urn("agreement")?,
            profile: offer.profile,
            permission: offer.permission,
            obligation: offer.obligation,
            _type: OdrlTypes::Agreement,
            target: offer.target,
            assigner,
            assignee,
            timestamp: Some(chrono::Utc::now().timestamp().to_string()),
            prohibition: offer.prohibition,
            description: offer.description,
        };

        self.apply_auth_token(&ctx.process.inner.associated_agent_peer).await;
        let response: NegotiationProcessMessageWrapper<NegotiationAckMessageDto> =
            self.http_client.post_json(peer_url.as_str(), &request_body).await?;

        let process = self
            .persistence_service
            .update_with_new_agreement(id.as_str(), input, &request_body.dto, &response.dto)
            .await?;

        Ok(RpcNegotiationMessageDto {
            request: input.clone(),
            response,
            negotiation_agent_model: process,
        })
    }

    /// Sends a `ContractAgreementVerificationMessage` to the provider.
    async fn setup_negotiation_agreement_verification_rpc(
        &self,
        input: &RpcNegotiationVerificationMessageDto,
    ) -> anyhow::Result<RpcNegotiationMessageDto<RpcNegotiationVerificationMessageDto>> {
        self.validator.negotiation_agreement_verification_rpc(input).await?;

        let id = self.get_rpc_consumer_pid_safely(input)?.to_string();
        let ctx = self.resolve_continuation_context(&Urn::from_str(&id)?).await?;
        let peer_url = format!(
            "{}/negotiations/{}/agreement/verification",
            ctx.peer_address, ctx.peer_identifier
        );

        let request_body: NegotiationProcessMessageWrapper<NegotiationVerificationMessageDto> =
            input.clone().into();
        self.apply_auth_token(&ctx.process.inner.associated_agent_peer).await;
        let response: NegotiationProcessMessageWrapper<NegotiationAckMessageDto> =
            self.http_client.post_json(peer_url.as_str(), &request_body).await?;

        let process = self
            .persistence_service
            .update_with_agreement(id.as_str(), input, &request_body.dto, &response.dto)
            .await?;

        Ok(RpcNegotiationMessageDto {
            request: input.clone(),
            response,
            negotiation_agent_model: process,
        })
    }

    /// Sends a `ContractNegotiationEventMessage` with event type `ACCEPTED`.
    async fn setup_negotiation_event_accepted_rpc(
        &self,
        input: &RpcNegotiationEventAcceptedMessageDto,
    ) -> anyhow::Result<RpcNegotiationMessageDto<RpcNegotiationEventAcceptedMessageDto>> {
        self.validator.negotiation_event_accepted_rpc(input).await?;

        let id = self.get_rpc_consumer_pid_safely(input)?.to_string();
        let ctx = self.resolve_continuation_context(&Urn::from_str(&id)?).await?;
        let peer_url = format!("{}/negotiations/{}/events", ctx.peer_address, ctx.peer_identifier);

        let request_body: NegotiationProcessMessageWrapper<NegotiationEventMessageDto> =
            input.clone().into();
        self.apply_auth_token(&ctx.process.inner.associated_agent_peer).await;
        let response: NegotiationProcessMessageWrapper<NegotiationAckMessageDto> =
            self.http_client.post_json(peer_url.as_str(), &request_body).await?;

        let process = self
            .persistence_service
            .update(id.as_str(), input, &request_body.dto, &response.dto)
            .await?;

        Ok(RpcNegotiationMessageDto {
            request: input.clone(),
            response,
            negotiation_agent_model: process,
        })
    }

    /// Sends a `ContractNegotiationEventMessage` with event type `FINALIZED`.
    async fn setup_negotiation_event_finalized_rpc(
        &self,
        input: &RpcNegotiationEventFinalizedMessageDto,
    ) -> anyhow::Result<RpcNegotiationMessageDto<RpcNegotiationEventFinalizedMessageDto>> {
        self.validator.negotiation_event_finalized_rpc(input).await?;

        let id = self.get_rpc_consumer_pid_safely(input)?.to_string();
        let ctx = self.resolve_continuation_context(&Urn::from_str(&id)?).await?;
        let peer_url = format!("{}/negotiations/{}/events", ctx.peer_address, ctx.peer_identifier);

        let request_body: NegotiationProcessMessageWrapper<NegotiationEventMessageDto> =
            input.clone().into();
        self.apply_auth_token(&ctx.process.inner.associated_agent_peer).await;
        let response: NegotiationProcessMessageWrapper<NegotiationAckMessageDto> =
            self.http_client.post_json(peer_url.as_str(), &request_body).await?;

        let process = self
            .persistence_service
            .update_with_agreement(id.as_str(), input, &request_body.dto, &response.dto)
            .await?;

        Ok(RpcNegotiationMessageDto {
            request: input.clone(),
            response,
            negotiation_agent_model: process,
        })
    }

    /// Sends a `ContractNegotiationTerminationMessage` to the peer.
    async fn setup_negotiation_termination_rpc(
        &self,
        input: &RpcNegotiationTerminationMessageDto,
    ) -> anyhow::Result<RpcNegotiationMessageDto<RpcNegotiationTerminationMessageDto>> {
        self.validator.negotiation_termination_rpc(input).await?;

        let id = self.get_rpc_consumer_pid_safely(input)?.to_string();
        let ctx = self.resolve_continuation_context(&Urn::from_str(&id)?).await?;
        let peer_url =
            format!("{}/negotiations/{}/termination", ctx.peer_address, ctx.peer_identifier);

        let request_body: NegotiationProcessMessageWrapper<NegotiationTerminationMessageDto> =
            input.clone().into();
        self.apply_auth_token(&ctx.process.inner.associated_agent_peer).await;
        let response: NegotiationProcessMessageWrapper<NegotiationAckMessageDto> =
            self.http_client.post_json(peer_url.as_str(), &request_body).await?;

        let process = self
            .persistence_service
            .update(id.as_str(), input, &request_body.dto, &response.dto)
            .await?;

        Ok(RpcNegotiationMessageDto {
            request: input.clone(),
            response,
            negotiation_agent_model: process,
        })
    }
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

impl RPCOrchestratorService {
    /// Fetch an existing negotiation process and derive the peer routing context.
    ///
    /// Used by every continuation operation (all operations after the initial
    /// request / offer).  The `consumer_pid` is the local agent's identifier for
    /// the negotiation; the method resolves the stored process and extracts the
    /// peer URL identifier (the *opposite* party's PID).
    async fn resolve_continuation_context(
        &self,
        consumer_pid: &Urn,
    ) -> anyhow::Result<NegotiationPeerContext> {
        let process =
            self.persistence_service.fetch_process(consumer_pid.to_string().as_str()).await?;

        // The outgoing URL uses the peer's identifier (opposite of local role).
        let peer_role = !process.inner.role.parse::<RoleConfig>()?;
        let role_identifier = self.parse_role_into_identifier(&peer_role)?.to_string();
        let peer_identifier = process.identifiers.get(&role_identifier).unwrap().clone();
        let peer_address = process.inner.callback_address.clone().unwrap();

        Ok(NegotiationPeerContext { process, peer_identifier, peer_address })
    }

    /// Set the auth token for the HTTP client from the stored peer credentials.
    ///
    /// Silently skips if the peer has no token; the request proceeds without auth.
    async fn apply_auth_token(&self, peer: &str) {
        if let Ok(mate) = self.mates_service.get_mate_by_id(peer.to_string()).await {
            if let Some(token) = mate.token {
                self.http_client.set_auth_token(token).await;
            }
        }
    }
}
