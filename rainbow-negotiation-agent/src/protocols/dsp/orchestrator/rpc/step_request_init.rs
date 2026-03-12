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
    NegotiationRpcInitialContext, NegotiationRpcStep,
};
use crate::protocols::dsp::orchestrator::rpc::types::{
    RpcNegotiationProcessMessageTrait, RpcNegotiationRequestInitMessageDto,
};
use crate::protocols::dsp::protocol_types::{
    NegotiationAckMessageDto, NegotiationProcessMessageWrapper, NegotiationRequestInitMessageDto,
};
use crate::protocols::dsp::validator::traits::validation_rpc_steps::ValidationRpcSteps;
use rainbow_common::facades::ssi_auth_facade::MatesFacadeTrait;
use rainbow_common::http_client::HttpClient;
use std::sync::Arc;

// ─── RpcRequestInitStep ───────────────────────────────────────────────────────

/// Initiates a brand-new negotiation by sending a `ContractRequestMessage` to
/// the Provider (Consumer-initiated flow, first message).
///
/// No process record exists yet.  The step:
/// 1. Reads routing info from the RPC input.
/// 2. Converts the input into a DSP-enveloped message (generating a fresh
///    `consumerPid` via the `Into` impl on [`RpcNegotiationRequestInitMessageDto`]).
/// 3. POSTs to `{provider_address}/negotiations/request`.
/// 4. Persists the new process using the `providerPid` returned in the ack.
pub(super) struct RpcRequestInitStep;

#[async_trait::async_trait]
impl NegotiationRpcStep for RpcRequestInitStep {
    type Input = RpcNegotiationRequestInitMessageDto;
    type Context = NegotiationRpcInitialContext;

    async fn validate(
        validator: &Arc<dyn ValidationRpcSteps>,
        input: &RpcNegotiationRequestInitMessageDto,
    ) -> anyhow::Result<()> {
        validator.negotiation_request_init_rpc(input).await
    }

    /// Reads the provider address and associated peer from the input.
    /// No database lookup is performed; the record is created in `send_and_persist`.
    async fn prepare_context(
        input: &RpcNegotiationRequestInitMessageDto,
        _persistence: &Arc<dyn NegotiationRpcPersistenceTrait>,
        _mates_service: &Arc<dyn MatesFacadeTrait>,
    ) -> anyhow::Result<NegotiationRpcInitialContext> {
        let provider_address =
            input.get_provider_address().unwrap_or_default();
        let associated_peer =
            input.get_associated_agent_peer().unwrap_or_default();
        Ok(NegotiationRpcInitialContext { provider_address, associated_peer })
    }

    fn auth_peer(ctx: &NegotiationRpcInitialContext) -> &str {
        &ctx.associated_peer
    }

    /// POSTs the request message to `{provider_address}/negotiations/request`
    /// and creates the local process record.
    async fn send_and_persist(
        http_client: &HttpClient,
        persistence: &Arc<dyn NegotiationRpcPersistenceTrait>,
        ctx: &NegotiationRpcInitialContext,
        input: &RpcNegotiationRequestInitMessageDto,
    ) -> anyhow::Result<(
        NegotiationProcessMessageWrapper<NegotiationAckMessageDto>,
        NegotiationProcessDto,
    )> {
        let peer_url = format!("{}/negotiations/request", ctx.provider_address);
        let request_body: NegotiationProcessMessageWrapper<NegotiationRequestInitMessageDto> =
            input.clone().into();

        let response: NegotiationProcessMessageWrapper<NegotiationAckMessageDto> =
            http_client.post_json(peer_url.as_str(), &request_body).await?;

        // Provider PID is only known after the peer acknowledges.
        let process =
            persistence.create_new(input, &request_body.dto, &response.dto).await?;

        Ok((response, process))
    }
}
