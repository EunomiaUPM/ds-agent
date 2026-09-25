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

use crate::protocols::dsp::orchestrator::protocol::persistence::OrchestrationPersistenceForProtocol;
use crate::protocols::dsp::orchestrator::protocol::step_trait::{
    NegotiationContinuationContext, NegotiationProtocolStep, continuation_prepare_context,
};
use crate::protocols::dsp::protocol_types::{
    NegotiationAckMessageDto, NegotiationProcessMessageWrapper, NegotiationRequestMessageDto,
};
use crate::protocols::dsp::validator::traits::validation_dsp_steps::ValidationDspSteps;
use crate::services::negotiation_process::views::NegotiationProcessView;
use common::dsp_common::DspActor;
use std::sync::Arc;
use ymir::data::entities::shared::participant::Model as Mates;
use ymir::errors::Outcome;
// ConsumerRequestStep ──────────────────────────────────────────────────────

/// Handles a subsequent `ContractRequestMessage` from the Consumer (counter-offer
/// on an already-open negotiation process).
///
/// Advances the process state and records a new offer alongside the state
/// transition message.
pub(super) struct ConsumerRequestStep;

#[async_trait::async_trait]
impl NegotiationProtocolStep for ConsumerRequestStep {
    type Dto = NegotiationRequestMessageDto;
    type Context = NegotiationContinuationContext;

    #[tracing::instrument(level = "info", skip_all, err)]
    async fn validate(
        validator: &Arc<dyn ValidationDspSteps>,
        id: &str,
        input: &NegotiationProcessMessageWrapper<NegotiationRequestMessageDto>,
        mate: &Mates,
    ) -> Outcome<()> {
        validator
            .on_contract_request(&DspActor::peer(mate), &id.to_string(), input)
            .await
    }

    #[tracing::instrument(level = "info", skip_all, err)]
    async fn prepare_context(
        id: &str,
        mate: &Mates,
        _input: &NegotiationProcessMessageWrapper<NegotiationRequestMessageDto>,
        persistence: &Arc<OrchestrationPersistenceForProtocol>,
    ) -> Outcome<(
        NegotiationContinuationContext,
        Option<NegotiationProcessMessageWrapper<NegotiationAckMessageDto>>,
    )> {
        continuation_prepare_context(id, mate, persistence).await
    }

    /// Advances the process state and records the consumer's counter-offer.
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn persist(
        persistence: &Arc<OrchestrationPersistenceForProtocol>,
        _id: &str,
        ctx: &NegotiationContinuationContext,
        input: &NegotiationProcessMessageWrapper<NegotiationRequestMessageDto>,
        mate: &Mates,
    ) -> Outcome<NegotiationProcessView> {
        persistence
            .update_with_offer(ctx.id.as_str(), &input.dto, mate)
            .await
    }
}
