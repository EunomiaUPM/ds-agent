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
use crate::protocols::dsp::persistence::NegotiationRpcPersistenceTrait;
use crate::protocols::dsp::orchestrator::rpc::step_trait::{
    resolve_continuation_context, NegotiationRpcContinuationContext, NegotiationRpcStep,
};
use crate::protocols::dsp::orchestrator::rpc::types::{
    RpcNegotiationProcessMessageTrait, RpcNegotiationTerminationMessageDto,
};
use crate::protocols::dsp::protocol_types::{
    NegotiationAckMessageDto, NegotiationProcessMessageWrapper, NegotiationTerminationMessageDto,
};
use crate::protocols::dsp::validator::traits::validation_rpc_steps::ValidationRpcSteps;
use anyhow::anyhow;
use rainbow_common::facades::ssi_auth_facade::MatesFacadeTrait;
use rainbow_common::http_client::HttpClient;
use std::sync::Arc;

// ─── RpcTerminationStep ───────────────────────────────────────────────────────

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
        input: &RpcNegotiationTerminationMessageDto,
    ) -> anyhow::Result<()> {
        validator.negotiation_termination_rpc(input).await
    }

    async fn prepare_context(
        input: &RpcNegotiationTerminationMessageDto,
        persistence: &Arc<dyn NegotiationRpcPersistenceTrait>,
        _mates_service: &Arc<dyn MatesFacadeTrait>,
    ) -> anyhow::Result<NegotiationRpcContinuationContext> {
        let id = input
            .get_consumer_pid()
            .ok_or_else(|| anyhow!("RpcTerminationStep: missing consumer PID"))?;
        resolve_continuation_context(&id, persistence).await
    }

    fn auth_peer(ctx: &NegotiationRpcContinuationContext) -> &str {
        &ctx.process.inner.associated_agent_peer
    }

    async fn send_and_persist(
        http_client: &HttpClient,
        persistence: &Arc<dyn NegotiationRpcPersistenceTrait>,
        ctx: &NegotiationRpcContinuationContext,
        input: &RpcNegotiationTerminationMessageDto,
    ) -> anyhow::Result<(
        NegotiationProcessMessageWrapper<NegotiationAckMessageDto>,
        NegotiationProcessDto,
    )> {
        let peer_url =
            format!("{}/negotiations/{}/termination", ctx.peer_address, ctx.peer_identifier);
        let request_body: NegotiationProcessMessageWrapper<NegotiationTerminationMessageDto> =
            input.clone().into();

        let response: NegotiationProcessMessageWrapper<NegotiationAckMessageDto> =
            http_client.post_json(peer_url.as_str(), &request_body).await?;

        let id = input
            .get_consumer_pid()
            .ok_or_else(|| anyhow!("RpcTerminationStep: missing consumer PID"))?;
        let process = persistence
            .update(id.to_string().as_str(), input, &request_body.dto, &response.dto)
            .await?;

        Ok((response, process))
    }
}
