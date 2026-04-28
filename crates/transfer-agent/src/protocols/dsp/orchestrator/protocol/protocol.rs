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
use crate::protocols::dsp::facades::FacadeTrait;
use crate::protocols::dsp::orchestrator::protocol::step_completion::ProtocolCompletionStep;
use crate::protocols::dsp::orchestrator::protocol::step_request::ProtocolRequestStep;
use crate::protocols::dsp::orchestrator::protocol::step_start::ProtocolStartStep;
use crate::protocols::dsp::orchestrator::protocol::step_suspension::ProtocolSuspensionStep;
use crate::protocols::dsp::orchestrator::protocol::step_termination::ProtocolTerminationStep;
use crate::protocols::dsp::orchestrator::protocol::step_trait::ProtocolStep;
use crate::protocols::dsp::orchestrator::protocol::ProtocolOrchestratorTrait;
use crate::protocols::dsp::persistence::TransferPersistenceTrait;
use crate::protocols::dsp::protocol_types::{
    TransferCompletionMessageDto, TransferProcessAckDto, TransferProcessMessageWrapper,
    TransferRequestMessageDto, TransferStartMessageDto, TransferSuspensionMessageDto,
    TransferTerminationMessageDto,
};
use crate::protocols::dsp::validator::traits::validation_dsp_steps::ValidationDspSteps;
use std::sync::Arc;
use ymir::errors::{Errors, Outcome};

pub struct ProtocolOrchestratorService {
    facades: Arc<dyn FacadeTrait>,
    validator: Arc<dyn ValidationDspSteps>,
    pub persistence_service: Arc<dyn TransferPersistenceTrait>,
}

impl ProtocolOrchestratorService {
    pub fn new(
        validator: Arc<dyn ValidationDspSteps>,
        persistence_service: Arc<dyn TransferPersistenceTrait>,
        facades: Arc<dyn FacadeTrait>,
    ) -> ProtocolOrchestratorService {
        ProtocolOrchestratorService {
            validator,
            persistence_service,
            facades,
        }
    }

    /// Template instantiation for protocol DSP
    async fn run_lifecycle<S: ProtocolStep>(
        &self,
        ctx: DspTransferContext,
        input: &TransferProcessMessageWrapper<S::Dto>,
    ) -> Outcome<(TransferProcessMessageWrapper<TransferProcessAckDto>, bool)> {
        // validation
        S::validate(&ctx, &self.validator, input).await?;
        // create context
        let mut ctx = ctx;
        if let Some(early_ack) =
            S::prepare_context(&mut ctx, input, &self.persistence_service, &self.facades).await?
        {
            return Ok((early_ack, true));
        }
        // persist
        S::persist(&mut ctx, &self.persistence_service, input).await?;
        // dataplane
        let dp = self.facades.get_data_plane_facade().await;
        S::post_hook(&mut ctx, &dp, input).await?;
        // prepare return
        let data_address = ctx.resolved_data_address;
        let process = ctx
            .process
            .ok_or_else(|| Errors::crazy("process missing after persist", None))?;
        let mut ack = TransferProcessMessageWrapper::try_from(process)?;
        ack.dto.data_address = data_address;
        Ok((ack, false))
    }
}

#[async_trait::async_trait]
impl ProtocolOrchestratorTrait for ProtocolOrchestratorService {
    async fn on_get_transfer_process(
        &self,
        id: &String,
    ) -> Outcome<TransferProcessMessageWrapper<TransferProcessAckDto>> {
        let process = self.persistence_service.fetch_process(id.as_str()).await?;
        Ok(TransferProcessMessageWrapper::try_from(process)?)
    }

    async fn on_transfer_request(
        &self,
        ctx: DspTransferContext,
        input: &TransferProcessMessageWrapper<TransferRequestMessageDto>,
    ) -> Outcome<(TransferProcessMessageWrapper<TransferProcessAckDto>, bool)> {
        self.run_lifecycle::<ProtocolRequestStep>(ctx, input).await
    }

    async fn on_transfer_start(
        &self,
        ctx: DspTransferContext,
        input: &TransferProcessMessageWrapper<TransferStartMessageDto>,
    ) -> Outcome<TransferProcessMessageWrapper<TransferProcessAckDto>> {
        let (ack, _) = self.run_lifecycle::<ProtocolStartStep>(ctx, input).await?;
        Ok(ack)
    }

    async fn on_transfer_suspension(
        &self,
        ctx: DspTransferContext,
        input: &TransferProcessMessageWrapper<TransferSuspensionMessageDto>,
    ) -> Outcome<TransferProcessMessageWrapper<TransferProcessAckDto>> {
        let (ack, _) = self
            .run_lifecycle::<ProtocolSuspensionStep>(ctx, input)
            .await?;
        Ok(ack)
    }

    async fn on_transfer_completion(
        &self,
        ctx: DspTransferContext,
        input: &TransferProcessMessageWrapper<TransferCompletionMessageDto>,
    ) -> Outcome<TransferProcessMessageWrapper<TransferProcessAckDto>> {
        let (ack, _) = self
            .run_lifecycle::<ProtocolCompletionStep>(ctx, input)
            .await?;
        Ok(ack)
    }

    async fn on_transfer_termination(
        &self,
        ctx: DspTransferContext,
        input: &TransferProcessMessageWrapper<TransferTerminationMessageDto>,
    ) -> Outcome<TransferProcessMessageWrapper<TransferProcessAckDto>> {
        let (ack, _) = self
            .run_lifecycle::<ProtocolTerminationStep>(ctx, input)
            .await?;
        Ok(ack)
    }
}