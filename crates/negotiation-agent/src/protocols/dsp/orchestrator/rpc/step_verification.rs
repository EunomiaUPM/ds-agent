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

use crate::entities::negotiation_process::NegotiationProcessDto;
use crate::protocols::dsp::orchestrator::rpc::step_trait::{
    NegotiationRpcContinuationContext, NegotiationRpcStep, resolve_continuation_context,
};
use crate::protocols::dsp::orchestrator::rpc::types::{
    RpcNegotiationProcessMessageTrait, RpcNegotiationVerificationMessageDto,
};
use crate::protocols::dsp::persistence::NegotiationRpcPersistenceTrait;
use crate::protocols::dsp::protocol_types::{
    NegotiationAckMessageDto, NegotiationProcessMessageWrapper, NegotiationVerificationMessageDto,
};
use crate::protocols::dsp::validator::traits::validation_rpc_steps::ValidationRpcSteps;
use common::facades::ssi_auth_facade::MatesFacadeTrait;
use common::http_client::HttpClient;
use std::sync::Arc;
use ymir::errors::{Errors, Outcome};

// RpcVerificationStep ──────────────────────────────────────────────────────

/// Sends a `ContractAgreementVerificationMessage` to the Provider
/// (Consumer - Provider).
///
/// Activates the existing agreement by calling `update_with_agreement` on
/// persistence, which sets the agreement state to `ACTIVE`.
pub(super) struct RpcVerificationStep;

#[async_trait::async_trait]
impl NegotiationRpcStep for RpcVerificationStep {
    type Input = RpcNegotiationVerificationMessageDto;
    type Context = NegotiationRpcContinuationContext;

    async fn validate(
        validator: &Arc<dyn ValidationRpcSteps>,
        input: &RpcNegotiationVerificationMessageDto,
    ) -> Outcome<()> {
        validator
            .negotiation_agreement_verification_rpc(input)
            .await
    }

    async fn prepare_context(
        input: &RpcNegotiationVerificationMessageDto,
        persistence: &Arc<dyn NegotiationRpcPersistenceTrait>,
        _mates_service: &Arc<dyn MatesFacadeTrait>,
    ) -> Outcome<NegotiationRpcContinuationContext> {
        let id = input
            .get_consumer_pid()
            .ok_or_else(|| Errors::parse("RpcVerificationStep: missing consumer PID", None))?;
        resolve_continuation_context(&id, persistence).await
    }

    fn auth_peer(ctx: &NegotiationRpcContinuationContext) -> &str {
        &ctx.process.inner.associated_agent_peer
    }

    async fn send_and_persist(
        http_client: &HttpClient,
        persistence: &Arc<dyn NegotiationRpcPersistenceTrait>,
        ctx: &NegotiationRpcContinuationContext,
        input: &RpcNegotiationVerificationMessageDto,
    ) -> Outcome<(
        NegotiationProcessMessageWrapper<NegotiationAckMessageDto>,
        NegotiationProcessDto,
    )> {
        let peer_url = format!(
            "{}/negotiations/{}/agreement/verification",
            ctx.peer_address, ctx.peer_identifier
        );
        let request_body: NegotiationProcessMessageWrapper<NegotiationVerificationMessageDto> =
            input.clone().into();

        let response: NegotiationProcessMessageWrapper<NegotiationAckMessageDto> = http_client
            .post_json(peer_url.as_str(), &request_body)
            .await?;

        let id = input
            .get_consumer_pid()
            .ok_or_else(|| Errors::parse("RpcVerificationStep: missing consumer PID", None))?;
        let process = persistence
            .update_with_agreement(
                id.to_string().as_str(),
                input,
                &request_body.dto,
                &response.dto,
            )
            .await?;

        Ok((response, process))
    }
}
