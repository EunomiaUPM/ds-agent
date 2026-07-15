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
    NegotiationInitialContext, NegotiationProtocolStep,
};
use crate::protocols::dsp::protocol_types::{
    NegotiationAckMessageDto, NegotiationOfferInitMessageDto, NegotiationProcessMessageWrapper,
};
use crate::protocols::dsp::validator::traits::validation_dsp_steps::ValidationDspSteps;
use std::sync::Arc;
use ymir::data::entities::shared::participant::Model as Mates;
use ymir::errors::Outcome;
// ─── InitialProviderOfferStep ─────────────────────────────────────────────────

/// Handles an inbound `ContractOfferMessage` that initiates a new negotiation
/// process (Provider - Consumer, first message of the offer flow).
///
/// This is the symmetric counterpart of [`InitialContractRequestStep`]: the
/// Provider drives the negotiation by sending the first offer.  The algorithm
/// is:
/// 1. Validate the message.
/// 2. No routing pre-processing needed (new process).
/// 3. Persist the new process together with the initial provider offer.
pub(super) struct InitialProviderOfferStep;

#[async_trait::async_trait]
impl NegotiationProtocolStep for InitialProviderOfferStep {
    type Dto = NegotiationOfferInitMessageDto;
    type Context = NegotiationInitialContext;

    async fn validate(
        validator: &Arc<dyn ValidationDspSteps>,
        _id: &str,
        input: &NegotiationProcessMessageWrapper<NegotiationOfferInitMessageDto>,
        _mate: &Mates,
    ) -> Outcome<()> {
        validator.on_contract_offer_init(input).await
    }

    /// No existing process to look up; always proceeds to persist.
    async fn prepare_context(
        _id: &str,
        _mate: &Mates,
        _input: &NegotiationProcessMessageWrapper<NegotiationOfferInitMessageDto>,
        _persistence: &Arc<OrchestrationPersistenceForProtocol>,
    ) -> Outcome<(
        NegotiationInitialContext,
        Option<NegotiationProcessMessageWrapper<NegotiationAckMessageDto>>,
    )> {
        Ok((NegotiationInitialContext, None))
    }

    /// Creates the new negotiation process record with the initial provider offer.
    async fn persist(
        persistence: &Arc<OrchestrationPersistenceForProtocol>,
        _id: &str,
        _ctx: &NegotiationInitialContext,
        input: &NegotiationProcessMessageWrapper<NegotiationOfferInitMessageDto>,
        mate: &Mates,
    ) -> Outcome<NegotiationProcessDto> {
        persistence.create_new(&input.dto, mate).await
    }
}
