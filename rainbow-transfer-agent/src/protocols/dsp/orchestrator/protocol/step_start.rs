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
    DataAddressDto, TransferProcessAckDto, TransferProcessMessageWrapper, TransferStartMessageDto,
};
use crate::protocols::dsp::validator::traits::validation_dsp_steps::ValidationDspSteps;
use std::sync::Arc;
use urn::Urn;

// ─── StartStep ────────────────────────────────────────────────────────────────

/// Handles an inbound `TransferStartMessage` from the peer.
///
/// Starts the local dataplane session.  In PULL mode the dataplane returns a
/// consumer ingress URL which is embedded in the acknowledgement so the provider
/// knows where to push data.
pub(super) struct ProtocolStartStep;

#[async_trait::async_trait]
impl ProtocolStep for ProtocolStartStep {
    type Dto = TransferStartMessageDto;
    type Context = ProtocolContext;

    async fn validate(
        validator: &Arc<dyn ValidationDspSteps>,
        id: &str,
        input: &TransferProcessMessageWrapper<TransferStartMessageDto>,
    ) -> anyhow::Result<()> {
        validator.on_transfer_start(&id.to_string(), input).await
    }

    async fn prepare_context(
        id: &str,
        _peer: &str,
        _input: &TransferProcessMessageWrapper<TransferStartMessageDto>,
        persistence: &Arc<dyn TransferPersistenceTrait>,
        _facades: &Arc<dyn FacadeTrait>,
    ) -> anyhow::Result<(
        ProtocolContext,
        Option<TransferProcessMessageWrapper<TransferProcessAckDto>>,
    )> {
        continuation_prepare_context(id, persistence).await
    }

    async fn persist(
        persistence: &Arc<dyn TransferPersistenceTrait>,
        id: &str,
        _ctx: &ProtocolContext,
        input: &TransferProcessMessageWrapper<TransferStartMessageDto>,
    ) -> anyhow::Result<TransferProcessDto> {
        continuation_persist(persistence, id, input).await
    }

    /// Starts the local dataplane; returns the consumer's ingress URL for PULL mode.
    async fn post_hook(
        dp: &Arc<dyn DataPlaneFacadeTrait>,
        ctx: &ProtocolContext,
        input: &TransferProcessMessageWrapper<TransferStartMessageDto>,
        _process_id: &Urn,
    ) -> anyhow::Result<Option<DataAddressDto>> {
        let process = &ctx.process.clone().ok_or(anyhow::anyhow!("no process found"))?;
        dp.on_transfer_start_post(&process, input.dto.data_address.clone()).await
    }
}
