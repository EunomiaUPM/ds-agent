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

use crate::entities::transfer_process::TransferProcessDto;
use crate::protocols::dsp::facades::dataplane_facade::DataPlaneFacadeTrait;
use crate::protocols::dsp::orchestrator::rpc::step_trait::{
    continuation_send_and_persist, resolve_continuation_context, RpcPeerContext, TransferRpcStep,
};
use crate::protocols::dsp::orchestrator::rpc::types::{
    RpcTransferCompletionMessageDto, RpcTransferProcessMessageTrait,
};
use crate::protocols::dsp::persistence::TransferPersistenceTrait;
use crate::protocols::dsp::protocol_types::{
    DataAddressDto, TransferCompletionMessageDto, TransferProcessAckDto, TransferProcessMessageWrapper,
};
use crate::protocols::dsp::validator::traits::validation_rpc_steps::ValidationRpcSteps;
use rainbow_common::http_client::HttpClient;
use std::sync::Arc;
use ymir::errors::{Errors, Outcome};
// ─── CompletionStep ───────────────────────────────────────────────────────────

/// Marks the transfer as successfully finished on both the local dataplane and
/// the remote peer.
pub(super) struct CompletionStep;

#[async_trait::async_trait]
impl TransferRpcStep for CompletionStep {
    type Input = RpcTransferCompletionMessageDto;
    type DspMessage = TransferCompletionMessageDto;
    type Context = RpcPeerContext;

    fn url_suffix() -> &'static str {
        "completion"
    }

    async fn validate(
        validator: &Arc<dyn ValidationRpcSteps>,
        input: &RpcTransferCompletionMessageDto,
    ) -> Outcome<()> {
        validator.transfer_completion_rpc(input).await
    }

    async fn prepare_context(
        input: &RpcTransferCompletionMessageDto,
        persistence: &Arc<dyn TransferPersistenceTrait>,
    ) -> Outcome<RpcPeerContext> {
        let pid = input
            .get_consumer_pid()
            .ok_or_else(|| Errors::crazy("CompletionStep: missing consumer PID", None))?;
        resolve_continuation_context(&pid, persistence).await
    }

    async fn pre_hook(
        dp: &Arc<dyn DataPlaneFacadeTrait>,
        ctx: &RpcPeerContext,
    ) -> Outcome<Option<DataAddressDto>> {
        dp.on_transfer_completion_pre(&ctx.process).await?;
        Ok(None)
    }

    fn build_message(
        _input: &RpcTransferCompletionMessageDto,
        ctx: &RpcPeerContext,
        _pre_addr: Option<DataAddressDto>,
    ) -> Outcome<TransferCompletionMessageDto> {
        Ok(TransferCompletionMessageDto {
            provider_pid: ctx.provider_pid.clone(),
            consumer_pid: ctx.consumer_pid.clone(),
        })
    }

    fn auth_peer(ctx: &RpcPeerContext) -> &str {
        &ctx.process.inner.associated_agent_peer
    }

    async fn send_and_persist(
        http_client: &HttpClient,
        persistence: &Arc<dyn TransferPersistenceTrait>,
        ctx: &RpcPeerContext,
        payload: Arc<TransferCompletionMessageDto>,
        url_suffix: &str,
    ) -> Outcome<(TransferProcessMessageWrapper<TransferProcessAckDto>, TransferProcessDto)>
    {
        continuation_send_and_persist(http_client, persistence, ctx, payload, url_suffix).await
    }

    async fn post_hook(
        dp: &Arc<dyn DataPlaneFacadeTrait>,
        ctx: &RpcPeerContext,
    ) -> Outcome<()> {
        dp.on_transfer_completion_post(&ctx.process).await
    }
}
