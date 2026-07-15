/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use crate::entities::negotiation_process::NegotiationProcessDto;
use crate::protocols::dsp::orchestrator::protocol::persistence::OrchestrationPersistenceForProtocol;
use crate::protocols::dsp::orchestrator::protocol::step_trait::{
    NegotiationContinuationContext, NegotiationProtocolStep, continuation_prepare_context,
};
use crate::protocols::dsp::protocol_types::{
    NegotiationAckMessageDto, NegotiationProcessMessageWrapper, NegotiationTerminationMessageDto,
};
use crate::protocols::dsp::validator::traits::validation_dsp_steps::ValidationDspSteps;
use std::sync::Arc;
use ymir::data::entities::shared::participant::Model as Mates;
use ymir::errors::Outcome;
// ─── NegotiationTerminationStep ───────────────────────────────────────────────

/// Handles an inbound `ContractNegotiationTerminationMessage` from the peer.
///
/// Advances the process to the `Terminated` state.  Termination can be sent
/// by either party at any point before the negotiation is finalized.
pub(super) struct NegotiationTerminationStep;

#[async_trait::async_trait]
impl NegotiationProtocolStep for NegotiationTerminationStep {
    type Dto = NegotiationTerminationMessageDto;
    type Context = NegotiationContinuationContext;

    async fn validate(
        validator: &Arc<dyn ValidationDspSteps>,
        id: &str,
        input: &NegotiationProcessMessageWrapper<NegotiationTerminationMessageDto>,
        _mate: &Mates,
    ) -> Outcome<()> {
        validator
            .on_contract_termination(&id.to_string(), input)
            .await
    }

    async fn prepare_context(
        id: &str,
        _mate: &Mates,
        _input: &NegotiationProcessMessageWrapper<NegotiationTerminationMessageDto>,
        persistence: &Arc<OrchestrationPersistenceForProtocol>,
    ) -> Outcome<(
        NegotiationContinuationContext,
        Option<NegotiationProcessMessageWrapper<NegotiationAckMessageDto>>,
    )> {
        continuation_prepare_context(id, persistence).await
    }

    /// Advances the process state to `Terminated`; no offer or agreement changes.
    async fn persist(
        persistence: &Arc<OrchestrationPersistenceForProtocol>,
        _id: &str,
        ctx: &NegotiationContinuationContext,
        input: &NegotiationProcessMessageWrapper<NegotiationTerminationMessageDto>,
        mate: &Mates,
    ) -> Outcome<NegotiationProcessDto> {
        persistence.update(ctx.id.as_str(), &input.dto, mate).await
    }
}
