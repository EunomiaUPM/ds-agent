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
    RpcTransferProcessMessageTrait, RpcTransferSuspensionMessageDto,
};
use crate::protocols::dsp::persistence::TransferPersistenceTrait;
use crate::protocols::dsp::protocol_types::{
    TransferProcessAckDto, TransferProcessMessageWrapper, TransferSuspensionMessageDto,
};
use crate::protocols::dsp::validator::traits::validation_rpc_steps::ValidationRpcSteps;
use common::http_client::HttpClient;
use std::sync::Arc;
use ymir::errors::{Errors, Outcome};


pub(super) struct SuspensionStep;

#[async_trait::async_trait]
impl TransferRpcStep for SuspensionStep {
    type Input = RpcTransferSuspensionMessageDto;
    type DspMessage = TransferSuspensionMessageDto;

    fn url_suffix() -> &'static str {
        "suspension"
    }

    async fn validate(
        validator: &Arc<dyn ValidationRpcSteps>,
        input: &RpcTransferSuspensionMessageDto,
    ) -> Outcome<()> {
        validator.transfer_suspension_rpc(input).await
    }

    async fn prepare_context(
        ctx: &mut DspTransferContext,
        input: &RpcTransferSuspensionMessageDto,
        persistence: &Arc<dyn TransferPersistenceTrait>,
    ) -> Outcome<()> {
        let pid = input
            .get_consumer_pid()
            .ok_or_else(|| Errors::crazy("SuspensionStep: missing consumer PID", None))?;
        populate_continuation_context(ctx, &pid, persistence).await
    }

    async fn pre_hook(
        dp: &Arc<dyn DataPlaneFacadeTrait>,
        ctx: &mut DspTransferContext,
    ) -> Outcome<()> {
        dp.on_transfer_suspension_pre(ctx).await?;
        Ok(())
    }

    fn build_message(
        ctx: &DspTransferContext,
        input: &RpcTransferSuspensionMessageDto,
    ) -> Outcome<TransferSuspensionMessageDto> {
        Ok(TransferSuspensionMessageDto {
            provider_pid: ctx
                .provider_pid
                .clone()
                .ok_or_else(|| Errors::crazy("SuspensionStep: provider_pid missing", None))?,
            consumer_pid: ctx
                .consumer_pid
                .clone()
                .ok_or_else(|| Errors::crazy("SuspensionStep: consumer_pid missing", None))?,
            code: input.code.clone(),
            reason: input.reason.clone(),
        })
    }

    async fn send_and_persist(
        http_client: &HttpClient,
        persistence: &Arc<dyn TransferPersistenceTrait>,
        ctx: &mut DspTransferContext,
        payload: Arc<TransferSuspensionMessageDto>,
    ) -> Outcome<TransferProcessMessageWrapper<TransferProcessAckDto>> {
        continuation_send_and_persist(http_client, persistence, ctx, payload, Self::url_suffix())
            .await
    }

    async fn post_hook(
        dp: &Arc<dyn DataPlaneFacadeTrait>,
        ctx: &mut DspTransferContext,
    ) -> Outcome<()> {
        dp.on_transfer_suspension_post(ctx).await?;
        Ok(())
    }
}
