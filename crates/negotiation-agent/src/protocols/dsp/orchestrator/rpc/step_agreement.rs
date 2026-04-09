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

use crate::entities::negotiation_process::NegotiationProcessDto;
use crate::protocols::dsp::orchestrator::rpc::step_trait::{
    NegotiationRpcAgreementContext, NegotiationRpcStep, resolve_continuation_context,
};
use crate::protocols::dsp::orchestrator::rpc::types::{
    RpcNegotiationAgreementMessageDto, RpcNegotiationProcessMessageTrait,
};
use crate::protocols::dsp::orchestrator::traits::orchestration_helpers::OrchestrationHelpers;
use crate::protocols::dsp::persistence::NegotiationRpcPersistenceTrait;
use crate::protocols::dsp::protocol_types::{
    NegotiationAckMessageDto, NegotiationAgreementMessageDto, NegotiationProcessMessageWrapper,
};
use crate::protocols::dsp::validator::traits::validation_rpc_steps::ValidationRpcSteps;
use common::dsp_common::odrl::{OdrlAgreement, OdrlMessageOffer, OdrlTypes};
use common::facades::ssi_auth_facade::MatesFacadeTrait;
use common::http_client::HttpClient;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::{Errors, Outcome};

// ─── AgreementEnricher (helper for build_message) ─────────────────────────────

/// Unit struct that implements [`OrchestrationHelpers`] to gain access to the
/// `create_entity_urn` helper needed in `send_and_persist`.
struct AgreementEnricher;
impl OrchestrationHelpers for AgreementEnricher {}

// ─── RpcAgreementStep ─────────────────────────────────────────────────────────

/// Sends a `ContractAgreementMessage` to the Consumer (Provider → Consumer).
///
/// This is the most complex RPC step because the outgoing agreement body must
/// be enriched before transmission:
/// - The last offer stored for this process is copied into the agreement policy.
/// - Participant IDs (`assigner` / `assignee`) are resolved from the mates
///   directory.
///
/// These lookups are performed in `prepare_context` so that `send_and_persist`
/// stays a focused POST + persist call.
pub(super) struct RpcAgreementStep;

#[async_trait::async_trait]
impl NegotiationRpcStep for RpcAgreementStep {
    type Input = RpcNegotiationAgreementMessageDto;
    type Context = NegotiationRpcAgreementContext;

    async fn validate(
        validator: &Arc<dyn ValidationRpcSteps>,
        input: &RpcNegotiationAgreementMessageDto,
    ) -> Outcome<()> {
        validator.negotiation_agreement_rpc(input).await
    }

    /// Resolves the continuation context and pre-fetches the enrichment data:
    /// the last offer (for agreement policy) and the participant IDs (from mates).
    async fn prepare_context(
        input: &RpcNegotiationAgreementMessageDto,
        persistence: &Arc<dyn NegotiationRpcPersistenceTrait>,
        mates_service: &Arc<dyn MatesFacadeTrait>,
    ) -> Outcome<NegotiationRpcAgreementContext> {
        let id = input
            .get_consumer_pid()
            .ok_or_else(|| Errors::parse("RpcAgreementStep: missing consumer PID", None))?;
        let base = resolve_continuation_context(&id, persistence).await?;

        // Fetch the last offer to copy its policy fields into the agreement.
        let last_offer_record = persistence
            .fetch_last_offer_by_process(base.process.inner.id.as_str())
            .await?;
        let last_offer =
            serde_json::from_value::<OdrlMessageOffer>(last_offer_record.inner.offer_content)?;

        // Resolve participant IDs from the mates directory.
        let assigner = mates_service
            .get_me_mate()
            .await
            .map(|m| m.participant_id)
            .unwrap_or_default();
        let assignee = mates_service
            .get_mate_by_id(base.process.inner.associated_agent_peer.clone())
            .await
            .map(|m| m.participant_id)
            .unwrap_or_default();

        Ok(NegotiationRpcAgreementContext {
            process: base.process,
            peer_identifier: base.peer_identifier,
            peer_address: base.peer_address,
            assigner,
            assignee,
            last_offer,
        })
    }

    fn auth_peer(ctx: &NegotiationRpcAgreementContext) -> &str {
        &ctx.process.inner.associated_agent_peer
    }

    /// Builds the enriched agreement message, POSTs it, and persists the result.
    ///
    /// The agreement body is constructed from the last offer's policy fields plus
    /// the participant IDs resolved in `prepare_context`.
    async fn send_and_persist(
        http_client: &HttpClient,
        persistence: &Arc<dyn NegotiationRpcPersistenceTrait>,
        ctx: &NegotiationRpcAgreementContext,
        input: &RpcNegotiationAgreementMessageDto,
    ) -> Outcome<(
        NegotiationProcessMessageWrapper<NegotiationAckMessageDto>,
        NegotiationProcessDto,
    )> {
        let peer_url = format!(
            "{}/negotiations/{}/agreement",
            ctx.peer_address, ctx.peer_identifier
        );

        let helper = AgreementEnricher;
        let mut request_body: NegotiationProcessMessageWrapper<NegotiationAgreementMessageDto> =
            input.clone().into();

        // Enrich the agreement with offer policy fields and participant IDs.
        request_body.dto.agreement = OdrlAgreement {
            id: helper.create_entity_urn("agreement")?,
            profile: ctx.last_offer.profile.clone(),
            permission: ctx.last_offer.permission.clone(),
            obligation: ctx.last_offer.obligation.clone(),
            _type: OdrlTypes::Agreement,
            target: ctx.last_offer.target.clone(),
            assigner: ctx.assigner.clone(),
            assignee: ctx.assignee.clone(),
            timestamp: Some(chrono::Utc::now().timestamp().to_string()),
            prohibition: ctx.last_offer.prohibition.clone(),
            description: ctx.last_offer.description.clone(),
        };

        let response: NegotiationProcessMessageWrapper<NegotiationAckMessageDto> = http_client
            .post_json(peer_url.as_str(), &request_body)
            .await?;

        let id = input
            .get_consumer_pid()
            .ok_or_else(|| Errors::parse("RpcAgreementStep: missing consumer PID", None))?;
        let process = persistence
            .update_with_new_agreement(
                id.to_string().as_str(),
                input,
                &request_body.dto,
                &response.dto,
            )
            .await?;

        Ok((response, process))
    }
}
