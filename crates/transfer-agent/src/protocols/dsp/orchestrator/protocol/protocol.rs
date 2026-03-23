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
use ymir::errors::Outcome;
// ─── Service ─────────────────────────────────────────────────────────────────

/// DSP protocol orchestrator for inbound transfer operations.
///
/// Handles messages arriving on the DSP HTTP endpoints.  All five operations
/// (request + four lifecycle steps) are driven by the [`ProtocolStep`] template;
/// `run_lifecycle` encodes the algorithm once.
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
}

// ─── Trait implementation ─────────────────────────────────────────────────────

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
        input: &TransferProcessMessageWrapper<TransferRequestMessageDto>,
        associated_agent_peer: &str,
    ) -> Outcome<(TransferProcessMessageWrapper<TransferProcessAckDto>, bool)> {
        self.run_lifecycle::<ProtocolRequestStep>("", associated_agent_peer, input)
            .await
    }

    async fn on_transfer_start(
        &self,
        id: &String,
        input: &TransferProcessMessageWrapper<TransferStartMessageDto>,
    ) -> Outcome<TransferProcessMessageWrapper<TransferProcessAckDto>> {
        let (ack, _) = self
            .run_lifecycle::<ProtocolStartStep>(id, "", input)
            .await?;
        Ok(ack)
    }

    async fn on_transfer_suspension(
        &self,
        id: &String,
        input: &TransferProcessMessageWrapper<TransferSuspensionMessageDto>,
    ) -> Outcome<TransferProcessMessageWrapper<TransferProcessAckDto>> {
        let (ack, _) = self
            .run_lifecycle::<ProtocolSuspensionStep>(id, "", input)
            .await?;
        Ok(ack)
    }

    async fn on_transfer_completion(
        &self,
        id: &String,
        input: &TransferProcessMessageWrapper<TransferCompletionMessageDto>,
    ) -> Outcome<TransferProcessMessageWrapper<TransferProcessAckDto>> {
        let (ack, _) = self
            .run_lifecycle::<ProtocolCompletionStep>(id, "", input)
            .await?;
        Ok(ack)
    }

    async fn on_transfer_termination(
        &self,
        id: &String,
        input: &TransferProcessMessageWrapper<TransferTerminationMessageDto>,
    ) -> Outcome<TransferProcessMessageWrapper<TransferProcessAckDto>> {
        let (ack, _) = self
            .run_lifecycle::<ProtocolTerminationStep>(id, "", input)
            .await?;
        Ok(ack)
    }
}

// ─── Template engine ──────────────────────────────────────────────────────────

impl ProtocolOrchestratorService {
    /// Execute any inbound DSP lifecycle step using the [`ProtocolStep`] template.
    ///
    /// The algorithm is always the same regardless of step type:
    /// validate → prepare context (with optional early ack) → persist → post-hook.
    ///
    /// `id` is the peer-facing process identifier (continuation steps; pass `""` for request).
    /// `peer` is the calling agent identifier (request step; pass `""` for continuation).
    async fn run_lifecycle<S: ProtocolStep>(
        &self,
        id: &str,
        peer: &str,
        input: &TransferProcessMessageWrapper<S::Dto>,
    ) -> Outcome<(TransferProcessMessageWrapper<TransferProcessAckDto>, bool)> {
        S::validate(&self.validator, id, input).await?;

        let (ctx, early) =
            S::prepare_context(id, peer, input, &self.persistence_service, &self.facades).await?;
        if let Some(early_ack) = early {
            return Ok((early_ack, true));
        }

        let process = S::persist(&self.persistence_service, id, &ctx, input).await?;

        let dp = self.facades.get_data_plane_facade().await;
        let data_addr = S::post_hook(&dp, &ctx, input, &process).await?;

        let mut ack = TransferProcessMessageWrapper::try_from(process)?;
        ack.dto.data_address = data_addr;
        Ok((ack, false))
    }
}
