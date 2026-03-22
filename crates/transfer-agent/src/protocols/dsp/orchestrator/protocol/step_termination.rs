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
use crate::protocols::dsp::facades::FacadeTrait;
use crate::protocols::dsp::orchestrator::protocol::step_trait::{
    continuation_persist, continuation_prepare_context, ProtocolContext, ProtocolStep,
};
use crate::protocols::dsp::persistence::TransferPersistenceTrait;
use crate::protocols::dsp::protocol_types::{
    DataAddressDto, TransferProcessAckDto, TransferProcessMessageWrapper,
    TransferTerminationMessageDto,
};
use crate::protocols::dsp::validator::traits::validation_dsp_steps::ValidationDspSteps;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::{Errors, Outcome};
// ─── TerminationStep ──────────────────────────────────────────────────────────

/// Handles an inbound `TransferTerminationMessage` from the peer.
///
/// Stops the local dataplane session abnormally.
pub(super) struct ProtocolTerminationStep;

#[async_trait::async_trait]
impl ProtocolStep for ProtocolTerminationStep {
    type Dto = TransferTerminationMessageDto;
    type Context = ProtocolContext;

    async fn validate(
        validator: &Arc<dyn ValidationDspSteps>,
        id: &str,
        input: &TransferProcessMessageWrapper<TransferTerminationMessageDto>,
    ) -> Outcome<()> {
        validator
            .on_transfer_termination(&id.to_string(), input)
            .await
    }

    async fn prepare_context(
        id: &str,
        _peer: &str,
        _input: &TransferProcessMessageWrapper<TransferTerminationMessageDto>,
        persistence: &Arc<dyn TransferPersistenceTrait>,
        _facades: &Arc<dyn FacadeTrait>,
    ) -> Outcome<(
        ProtocolContext,
        Option<TransferProcessMessageWrapper<TransferProcessAckDto>>,
    )> {
        continuation_prepare_context(id, persistence).await
    }

    async fn persist(
        persistence: &Arc<dyn TransferPersistenceTrait>,
        id: &str,
        _ctx: &ProtocolContext,
        input: &TransferProcessMessageWrapper<TransferTerminationMessageDto>,
    ) -> Outcome<TransferProcessDto> {
        continuation_persist(persistence, id, input).await
    }

    async fn post_hook(
        dp: &Arc<dyn DataPlaneFacadeTrait>,
        ctx: &ProtocolContext,
        _input: &TransferProcessMessageWrapper<TransferTerminationMessageDto>,
        _process_id: &Urn,
    ) -> Outcome<Option<DataAddressDto>> {
        let process = &ctx
            .process
            .clone()
            .ok_or(Errors::crazy("no process found", None))?;
        dp.on_transfer_termination_post(process).await?;
        Ok(None)
    }
}
