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
    RpcNegotiationOfferInitMessageDto, RpcNegotiationProcessMessageTrait,
};
use crate::protocols::dsp::protocol_types::{
    NegotiationAckMessageDto, NegotiationOfferInitMessageDto, NegotiationProcessMessageWrapper,
};
use crate::protocols::dsp::validator::traits::validation_rpc_steps::ValidationRpcSteps;
use rainbow_common::facades::ssi_auth_facade::MatesFacadeTrait;
use rainbow_common::http_client::HttpClient;
use std::sync::Arc;

// ─── RpcOfferInitStep ─────────────────────────────────────────────────────────

/// Initiates a brand-new negotiation by sending a `ContractOfferMessage` to
/// the Consumer (Provider-initiated flow, first message).
///
/// Symmetric to [`RpcRequestInitStep`]: the Provider drives the negotiation
/// by sending the first offer.  The step:
/// 1. Reads routing info from the RPC input.
/// 2. Converts the input into a DSP-enveloped message (generating a fresh
///    `providerPid` via the `Into` impl on [`RpcNegotiationOfferInitMessageDto`]).
/// 3. POSTs to `{provider_address}/negotiations/offers`.
/// 4. Persists the new process using the PIDs returned in the ack.
pub(super) struct RpcOfferInitStep;

#[async_trait::async_trait]
impl NegotiationRpcStep for RpcOfferInitStep {
    type Input = RpcNegotiationOfferInitMessageDto;
    type Context = NegotiationRpcInitialContext;

    async fn validate(
        validator: &Arc<dyn ValidationRpcSteps>,
        input: &RpcNegotiationOfferInitMessageDto,
    ) -> anyhow::Result<()> {
        validator.negotiation_offer_init_rpc(input).await
    }

    async fn prepare_context(
        input: &RpcNegotiationOfferInitMessageDto,
        _persistence: &Arc<dyn NegotiationRpcPersistenceTrait>,
        _mates_service: &Arc<dyn MatesFacadeTrait>,
    ) -> anyhow::Result<NegotiationRpcInitialContext> {
        let provider_address = input.get_provider_address().unwrap_or_default();
        let associated_peer = input.get_associated_agent_peer().unwrap_or_default();
        Ok(NegotiationRpcInitialContext { provider_address, associated_peer })
    }

    fn auth_peer(ctx: &NegotiationRpcInitialContext) -> &str {
        &ctx.associated_peer
    }

    /// POSTs the offer message to `{provider_address}/negotiations/offers`
    /// and creates the local process record.
    async fn send_and_persist(
        http_client: &HttpClient,
        persistence: &Arc<dyn NegotiationRpcPersistenceTrait>,
        ctx: &NegotiationRpcInitialContext,
        input: &RpcNegotiationOfferInitMessageDto,
    ) -> anyhow::Result<(
        NegotiationProcessMessageWrapper<NegotiationAckMessageDto>,
        NegotiationProcessDto,
    )> {
        let peer_url = format!("{}/negotiations/offers", ctx.provider_address);
        let request_body: NegotiationProcessMessageWrapper<NegotiationOfferInitMessageDto> =
            input.clone().into();

        let response: NegotiationProcessMessageWrapper<NegotiationAckMessageDto> =
            http_client.post_json(peer_url.as_str(), &request_body).await?;

        let process =
            persistence.create_new(input, &request_body.dto, &response.dto).await?;

        Ok((response, process))
    }
}
