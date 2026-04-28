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
use crate::protocols::dsp::facades::FacadeTrait;
use crate::protocols::dsp::orchestrator::protocol::step_trait::{
    continuation_persist, continuation_prepare_context, ProtocolStep,
};
use crate::protocols::dsp::persistence::TransferPersistenceTrait;
use crate::protocols::dsp::protocol_types::{
    TransferProcessAckDto, TransferProcessMessageWrapper, TransferStartMessageDto,
};
use crate::protocols::dsp::validator::traits::validation_dsp_steps::ValidationDspSteps;
use std::sync::Arc;
use ymir::errors::Outcome;


pub(super) struct ProtocolStartStep;

#[async_trait::async_trait]
impl ProtocolStep for ProtocolStartStep {
    type Dto = TransferStartMessageDto;

    async fn validate(
        ctx: &DspTransferContext,
        validator: &Arc<dyn ValidationDspSteps>,
        input: &TransferProcessMessageWrapper<TransferStartMessageDto>,
    ) -> Outcome<()> {
        validator.on_transfer_start(ctx, input).await
    }

    async fn prepare_context(
        ctx: &mut DspTransferContext,
        _input: &TransferProcessMessageWrapper<TransferStartMessageDto>,
        persistence: &Arc<dyn TransferPersistenceTrait>,
        _facades: &Arc<dyn FacadeTrait>,
    ) -> Outcome<Option<TransferProcessMessageWrapper<TransferProcessAckDto>>> {
        continuation_prepare_context(ctx, persistence).await
    }

    async fn persist(
        ctx: &mut DspTransferContext,
        persistence: &Arc<dyn TransferPersistenceTrait>,
        input: &TransferProcessMessageWrapper<TransferStartMessageDto>,
    ) -> Outcome<()> {
        continuation_persist(ctx, persistence, input).await
    }

    async fn post_hook(
        ctx: &mut DspTransferContext,
        dp: &Arc<dyn DataPlaneFacadeTrait>,
        input: &TransferProcessMessageWrapper<TransferStartMessageDto>,
    ) -> Outcome<()> {
        // The data address from the inbound StartMessage is the provider's proxy URL.
        ctx.input_data_address = input.dto.data_address.clone();
        let addr = dp.on_transfer_start_post(ctx).await?;
        ctx.resolved_data_address = addr;
        Ok(())
    }
}
