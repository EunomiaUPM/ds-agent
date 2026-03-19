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
    RpcTransferProcessMessageTrait, RpcTransferSuspensionMessageDto,
};
use crate::protocols::dsp::persistence::TransferPersistenceTrait;
use crate::protocols::dsp::protocol_types::{
    DataAddressDto, TransferProcessAckDto, TransferProcessMessageWrapper, TransferSuspensionMessageDto,
};
use crate::protocols::dsp::validator::traits::validation_rpc_steps::ValidationRpcSteps;
use rainbow_common::http_client::HttpClient;
use std::sync::Arc;
use urn::Urn;

// ─── SuspensionStep ───────────────────────────────────────────────────────────

/// Pauses the local dataplane and notifies the peer to suspend the transfer.
pub(super) struct SuspensionStep;

#[async_trait::async_trait]
impl TransferRpcStep for SuspensionStep {
    type Input = RpcTransferSuspensionMessageDto;
    type DspMessage = TransferSuspensionMessageDto;
    type Context = RpcPeerContext;

    fn url_suffix() -> &'static str {
        "suspension"
    }

    async fn validate(
        validator: &Arc<dyn ValidationRpcSteps>,
        input: &RpcTransferSuspensionMessageDto,
    ) -> anyhow::Result<()> {
        validator.transfer_suspension_rpc(input).await
    }

    async fn prepare_context(
        input: &RpcTransferSuspensionMessageDto,
        persistence: &Arc<dyn TransferPersistenceTrait>,
    ) -> anyhow::Result<RpcPeerContext> {
        let pid = input
            .get_consumer_pid()
            .ok_or_else(|| anyhow::anyhow!("SuspensionStep: missing consumer PID"))?;
        resolve_continuation_context(&pid, persistence).await
    }

    async fn pre_hook(
        dp: &Arc<dyn DataPlaneFacadeTrait>,
        ctx: &RpcPeerContext,
    ) -> anyhow::Result<Option<DataAddressDto>> {
        dp.on_transfer_suspension_pre(&ctx.process).await?;
        Ok(None)
    }

    fn build_message(
        input: &RpcTransferSuspensionMessageDto,
        ctx: &RpcPeerContext,
        _pre_addr: Option<DataAddressDto>,
    ) -> anyhow::Result<TransferSuspensionMessageDto> {
        Ok(TransferSuspensionMessageDto {
            provider_pid: ctx.provider_pid.clone(),
            consumer_pid: ctx.consumer_pid.clone(),
            code: input.code.clone(),
            reason: input.reason.clone(),
        })
    }

    fn auth_peer(ctx: &RpcPeerContext) -> &str {
        &ctx.process.inner.associated_agent_peer
    }

    async fn send_and_persist(
        http_client: &HttpClient,
        persistence: &Arc<dyn TransferPersistenceTrait>,
        ctx: &RpcPeerContext,
        payload: Arc<TransferSuspensionMessageDto>,
        url_suffix: &str,
    ) -> anyhow::Result<(TransferProcessMessageWrapper<TransferProcessAckDto>, TransferProcessDto)>
    {
        continuation_send_and_persist(http_client, persistence, ctx, payload, url_suffix).await
    }

    async fn post_hook(
        dp: &Arc<dyn DataPlaneFacadeTrait>,
        ctx: &RpcPeerContext,
    ) -> anyhow::Result<()> {
        dp.on_transfer_suspension_post(&ctx.process).await
    }
}
