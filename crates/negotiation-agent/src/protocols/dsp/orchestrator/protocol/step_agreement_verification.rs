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
    NegotiationAckMessageDto, NegotiationProcessMessageWrapper, NegotiationVerificationMessageDto,
};
use crate::protocols::dsp::validator::traits::validation_dsp_steps::ValidationDspSteps;
use std::sync::Arc;
use ymir::data::entities::shared::participant::Model as Mates;
use ymir::errors::Outcome;
// ─── AgreementVerificationStep ────────────────────────────────────────────────

/// Handles an inbound `ContractAgreementVerificationMessage` from the Consumer.
///
/// Advances the process to the `Verified` state.  The agreement itself was
/// already created on [`AgreementReceptionStep`]; this step only records the
/// acknowledgement from the Consumer that the agreement is accepted.
pub(super) struct AgreementVerificationStep;

#[async_trait::async_trait]
impl NegotiationProtocolStep for AgreementVerificationStep {
    type Dto = NegotiationVerificationMessageDto;
    type Context = NegotiationContinuationContext;

    async fn validate(
        validator: &Arc<dyn ValidationDspSteps>,
        id: &str,
        input: &NegotiationProcessMessageWrapper<NegotiationVerificationMessageDto>,
        _mate: &Mates,
    ) -> Outcome<()> {
        validator
            .on_contract_agreement_verification(&id.to_string(), input)
            .await
    }

    async fn prepare_context(
        id: &str,
        _mate: &Mates,
        _input: &NegotiationProcessMessageWrapper<NegotiationVerificationMessageDto>,
        persistence: &Arc<OrchestrationPersistenceForProtocol>,
    ) -> Outcome<(
        NegotiationContinuationContext,
        Option<NegotiationProcessMessageWrapper<NegotiationAckMessageDto>>,
    )> {
        continuation_prepare_context(id, persistence).await
    }

    /// Advances the process state only; no offer or agreement records are added.
    async fn persist(
        persistence: &Arc<OrchestrationPersistenceForProtocol>,
        _id: &str,
        ctx: &NegotiationContinuationContext,
        input: &NegotiationProcessMessageWrapper<NegotiationVerificationMessageDto>,
        mate: &Mates,
    ) -> Outcome<NegotiationProcessDto> {
        persistence.update(ctx.id.as_str(), &input.dto, mate).await
    }
}
