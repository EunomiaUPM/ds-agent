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

use crate::protocols::dsp::context::DspTransferContext;
use crate::protocols::dsp::facades::dataplane_facade::DataPlaneFacadeTrait;
use crate::protocols::dsp::orchestrator::rpc::step_trait::{
    continuation_send_and_persist, populate_continuation_context, TransferRpcStep,
};
use crate::protocols::dsp::orchestrator::rpc::types::{
    RpcTransferProcessMessageTrait, RpcTransferStartMessageDto,
};
use crate::protocols::dsp::persistence::TransferPersistenceTrait;
use crate::protocols::dsp::protocol_types::{
    TransferProcessAckDto, TransferProcessMessageWrapper, TransferStartMessageDto,
};
use common::http_client::HttpClient;
use std::sync::Arc;
use ymir::errors::{Errors, Outcome};

pub(super) struct StartStep;

#[async_trait::async_trait]
impl TransferRpcStep for StartStep {
    type Input = RpcTransferStartMessageDto;
    type DspMessage = TransferStartMessageDto;

    fn url_suffix() -> &'static str {
        "start"
    }

    async fn prepare_context(
        ctx: &mut DspTransferContext,
        input: &RpcTransferStartMessageDto,
        persistence: &Arc<dyn TransferPersistenceTrait>,
    ) -> Outcome<()> {
        dbg!("1.", &ctx);
        let pid = input
            .get_consumer_pid()
            .ok_or_else(|| Errors::crazy("StartStep: missing consumer PID", None))?;
        populate_continuation_context(ctx, &pid, persistence).await
    }

    async fn pre_hook(
        dp: &Arc<dyn DataPlaneFacadeTrait>,
        ctx: &mut DspTransferContext,
    ) -> Outcome<()> {
        dbg!("2.", &ctx);
        let addr = dp.on_transfer_start_pre(ctx).await?;
        ctx.resolved_data_address = addr;
        Ok(())
    }

    fn build_message(
        ctx: &DspTransferContext,
        _input: &RpcTransferStartMessageDto,
    ) -> Outcome<TransferStartMessageDto> {
        dbg!("3.", &ctx);
        let data_address = if ctx.is_restart {
            None
        } else {
            ctx.resolved_data_address.clone()
        };
        Ok(TransferStartMessageDto {
            provider_pid: ctx
                .provider_pid
                .clone()
                .ok_or_else(|| Errors::crazy("StartStep: provider_pid missing in ctx", None))?,
            consumer_pid: ctx
                .consumer_pid
                .clone()
                .ok_or_else(|| Errors::crazy("StartStep: consumer_pid missing in ctx", None))?,
            data_address,
        })
    }

    async fn send_and_persist(
        http_client: &HttpClient,
        persistence: &Arc<dyn TransferPersistenceTrait>,
        ctx: &mut DspTransferContext,
        payload: Arc<TransferStartMessageDto>,
    ) -> Outcome<TransferProcessMessageWrapper<TransferProcessAckDto>> {
        dbg!("4.", &ctx);
        continuation_send_and_persist(http_client, persistence, ctx, payload, Self::url_suffix())
            .await
    }

    async fn post_hook(
        dp: &Arc<dyn DataPlaneFacadeTrait>,
        ctx: &mut DspTransferContext,
    ) -> Outcome<()> {
        dbg!("5.", &ctx);
        dp.on_transfer_start_post(ctx).await?;
        Ok(())
    }
}
