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

use crate::entities::negotiation_process::NegotiationProcessDto;
use crate::protocols::dsp::orchestrator::protocol::persistence::OrchestrationPersistenceForProtocol;
use crate::protocols::dsp::orchestrator::protocol::step_trait::{
    NegotiationContinuationContext, NegotiationProtocolStep, continuation_prepare_context,
};
use crate::protocols::dsp::protocol_types::{
    NegotiationAckMessageDto, NegotiationEventMessageDto, NegotiationEventType,
    NegotiationProcessMessageWrapper,
};
use crate::protocols::dsp::validator::traits::validation_dsp_steps::ValidationDspSteps;
use common::facades::Mates;
use std::sync::Arc;
use ymir::errors::Outcome;
// ─── NegotiationEventStep ─────────────────────────────────────────────────────

/// Handles an inbound `ContractNegotiationEventMessage`.
///
/// The behaviour branches on the `event_type` field:
/// - **`ACCEPTED`** – the peer acknowledges the latest offer; only the process
///   state is advanced (no new offer or agreement record is created).
/// - **`FINALIZED`** – the negotiation is complete; the existing agreement record
///   is activated by setting its state to `ACTIVE`.
pub(super) struct NegotiationEventStep;

#[async_trait::async_trait]
impl NegotiationProtocolStep for NegotiationEventStep {
    type Dto = NegotiationEventMessageDto;
    type Context = NegotiationContinuationContext;

    async fn validate(
        validator: &Arc<dyn ValidationDspSteps>,
        id: &str,
        input: &NegotiationProcessMessageWrapper<NegotiationEventMessageDto>,
        _mate: &Mates,
    ) -> Outcome<()> {
        validator.on_contract_event(&id.to_string(), input).await
    }

    async fn prepare_context(
        id: &str,
        _mate: &Mates,
        _input: &NegotiationProcessMessageWrapper<NegotiationEventMessageDto>,
        persistence: &Arc<OrchestrationPersistenceForProtocol>,
    ) -> Outcome<(
        NegotiationContinuationContext,
        Option<NegotiationProcessMessageWrapper<NegotiationAckMessageDto>>,
    )> {
        continuation_prepare_context(id, persistence).await
    }

    /// Dispatches to the appropriate persistence variant based on `event_type`.
    ///
    /// - `ACCEPTED` → `update` (state advancement only)
    /// - `FINALIZED` → `update_with_agreement` (activates the existing agreement)
    async fn persist(
        persistence: &Arc<OrchestrationPersistenceForProtocol>,
        _id: &str,
        ctx: &NegotiationContinuationContext,
        input: &NegotiationProcessMessageWrapper<NegotiationEventMessageDto>,
        mate: &Mates,
    ) -> Outcome<NegotiationProcessDto> {
        match &input.dto.event_type {
            NegotiationEventType::ACCEPTED => {
                persistence.update(ctx.id.as_str(), &input.dto, mate).await
            }
            NegotiationEventType::FINALIZED => {
                persistence
                    .update_with_agreement(ctx.id.as_str(), &input.dto, mate)
                    .await
            }
        }
    }
}
