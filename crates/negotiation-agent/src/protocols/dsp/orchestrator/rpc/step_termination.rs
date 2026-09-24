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

use crate::protocols::dsp::orchestrator::rpc::step_trait::{
    NegotiationRpcContinuationContext, NegotiationRpcStep,
};
use crate::protocols::dsp::orchestrator::rpc::types::{
    RpcNegotiationProcessMessageTrait, RpcNegotiationTerminationMessageDto,
};
use crate::protocols::dsp::persistence::NegotiationRpcPersistenceTrait;
use crate::protocols::dsp::protocol_types::{
    NegotiationAckMessageDto, NegotiationProcessMessageWrapper, NegotiationTerminationMessageDto,
};
use crate::protocols::dsp::validator::traits::validation_rpc_steps::ValidationRpcSteps;
use crate::services::negotiation_process::views::NegotiationProcessView;
use axum::http::HeaderMap;
use common::auth::AccessScope;
use common::dsp_common::DspActor;
use common::facades::ssi_auth_facade::MatesFacadeTrait;
use std::sync::Arc;
use ymir::errors::{Errors, Outcome};
use ymir::services::client::ClientExt;
use ymir::utils::http_client;

// RpcTerminationStep ───────────────────────────────────────────────────────

/// Sends a `ContractNegotiationTerminationMessage` to the peer.
///
/// Can be sent by either party at any point before the negotiation is
/// finalized.  Only the process state is advanced; no offer or agreement
/// changes are made.
pub(super) struct RpcTerminationStep;

#[async_trait::async_trait]
impl NegotiationRpcStep for RpcTerminationStep {
    type Input = RpcNegotiationTerminationMessageDto;
    type Context = NegotiationRpcContinuationContext;

    async fn validate(
        validator: &Arc<dyn ValidationRpcSteps>,
        actor: &DspActor,
        input: &RpcNegotiationTerminationMessageDto,
    ) -> Outcome<()> {
        validator.negotiation_termination_rpc(actor, input).await
    }

    async fn prepare_context(
        scope: &AccessScope,
        input: &RpcNegotiationTerminationMessageDto,
        persistence: &Arc<dyn NegotiationRpcPersistenceTrait>,
        _mates_service: &Arc<dyn MatesFacadeTrait>,
    ) -> Outcome<NegotiationRpcContinuationContext> {
        let id = input
            .get_consumer_pid()
            .ok_or_else(|| Errors::parse("RpcTerminationStep: missing consumer PID", None))?;
        NegotiationRpcContinuationContext::resolve(&id, scope, persistence).await
    }

    fn auth_peer(ctx: &NegotiationRpcContinuationContext) -> (&str, &str) {
        (
            &ctx.process.inner.tenant_id,
            &ctx.process.inner.associated_agent_peer,
        )
    }

    async fn send_and_persist(
        headers: Option<HeaderMap>,
        persistence: &Arc<dyn NegotiationRpcPersistenceTrait>,
        ctx: &NegotiationRpcContinuationContext,
        input: &RpcNegotiationTerminationMessageDto,
    ) -> Outcome<(
        NegotiationProcessMessageWrapper<NegotiationAckMessageDto>,
        NegotiationProcessView,
    )> {
        let peer_url = format!(
            "{}/negotiations/{}/termination",
            ctx.peer_address, ctx.peer_identifier
        );
        let request_body: NegotiationProcessMessageWrapper<NegotiationTerminationMessageDto> =
            input.clone().into();

        let response: NegotiationProcessMessageWrapper<NegotiationAckMessageDto> = http_client()
            .post_json(peer_url.as_str(), headers, &request_body)
            .await?;

        let process = persistence
            .update(&ctx.process, input, &request_body.dto, &response.dto)
            .await?;

        Ok((response, process))
    }
}
