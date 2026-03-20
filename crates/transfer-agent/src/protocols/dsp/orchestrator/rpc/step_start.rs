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
    RpcTransferProcessMessageTrait, RpcTransferStartMessageDto,
};
use crate::protocols::dsp::persistence::TransferPersistenceTrait;
use crate::protocols::dsp::protocol_types::{
    DataAddressDto, TransferProcessAckDto, TransferProcessMessageWrapper, TransferStartMessageDto,
};
use common::http_client::HttpClient;
use std::sync::Arc;
use ymir::errors::{Errors, Outcome};
// ─── StartStep ────────────────────────────────────────────────────────────────

/// Activates the local dataplane and signals the peer to start the transfer.
///
/// In PULL mode the `pre_hook` returns a proxy URL that is forwarded to the
/// peer as the provider's `DataAddress`.  For restarts the data address is
/// withheld to avoid re-initiating the dataplane session.
pub(super) struct StartStep;

#[async_trait::async_trait]
impl TransferRpcStep for StartStep {
    type Input = RpcTransferStartMessageDto;
    type DspMessage = TransferStartMessageDto;
    type Context = RpcPeerContext;

    fn url_suffix() -> &'static str {
        "start"
    }

    async fn prepare_context(
        input: &RpcTransferStartMessageDto,
        persistence: &Arc<dyn TransferPersistenceTrait>,
    ) -> Outcome<RpcPeerContext> {
        let pid = input
            .get_consumer_pid()
            .ok_or_else(|| Errors::crazy("StartStep: missing consumer PID", None))?;
        resolve_continuation_context(&pid, persistence).await
    }

    async fn pre_hook(
        dp: &Arc<dyn DataPlaneFacadeTrait>,
        ctx: &RpcPeerContext,
    ) -> Outcome<Option<DataAddressDto>> {
        dp.on_transfer_start_pre(&ctx.process).await
    }

    fn build_message(
        _input: &RpcTransferStartMessageDto,
        ctx: &RpcPeerContext,
        pre_addr: Option<DataAddressDto>,
    ) -> Outcome<TransferStartMessageDto> {
        // For restarts, omit the data address to avoid re-initiating the dataplane.
        let data_address = if ctx.is_restart { None } else { pre_addr };
        Ok(TransferStartMessageDto {
            provider_pid: ctx.provider_pid.clone(),
            consumer_pid: ctx.consumer_pid.clone(),
            data_address,
        })
    }

    fn auth_peer(ctx: &RpcPeerContext) -> &str {
        &ctx.process.inner.associated_agent_peer
    }

    async fn send_and_persist(
        http_client: &HttpClient,
        persistence: &Arc<dyn TransferPersistenceTrait>,
        ctx: &RpcPeerContext,
        payload: Arc<TransferStartMessageDto>,
        url_suffix: &str,
    ) -> Outcome<(
        TransferProcessMessageWrapper<TransferProcessAckDto>,
        TransferProcessDto,
    )> {
        continuation_send_and_persist(http_client, persistence, ctx, payload, url_suffix).await
    }

    async fn post_hook(dp: &Arc<dyn DataPlaneFacadeTrait>, ctx: &Self::Context) -> Outcome<()> {
        // Outbound sender: no incoming DataAddress to apply, so pass None.
        dp.on_transfer_start_post(&ctx.process, None).await?;
        Ok(())
    }
}
