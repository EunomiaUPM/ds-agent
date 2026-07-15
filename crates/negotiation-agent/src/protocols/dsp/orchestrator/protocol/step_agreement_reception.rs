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
    NegotiationAckMessageDto, NegotiationAgreementMessageDto, NegotiationProcessMessageWrapper,
};
use crate::protocols::dsp::validator::traits::validation_dsp_steps::ValidationDspSteps;
use std::sync::Arc;
use ymir::data::entities::shared::participant::Model as Mates;
use ymir::errors::Outcome;
// ─── AgreementReceptionStep ───────────────────────────────────────────────────

/// Handles an inbound `ContractAgreementMessage` from the Provider.
///
/// This step creates the agreement record in the database and advances the
/// process to the `Agreed` state.  A subsequent [`AgreementVerificationStep`]
/// from the Consumer will then activate the agreement.
pub(super) struct AgreementReceptionStep;

#[async_trait::async_trait]
impl NegotiationProtocolStep for AgreementReceptionStep {
    type Dto = NegotiationAgreementMessageDto;
    type Context = NegotiationContinuationContext;

    async fn validate(
        validator: &Arc<dyn ValidationDspSteps>,
        id: &str,
        input: &NegotiationProcessMessageWrapper<NegotiationAgreementMessageDto>,
        _mate: &Mates,
    ) -> Outcome<()> {
        validator
            .on_contract_agreement(&id.to_string(), input)
            .await
    }

    async fn prepare_context(
        id: &str,
        _mate: &Mates,
        _input: &NegotiationProcessMessageWrapper<NegotiationAgreementMessageDto>,
        persistence: &Arc<OrchestrationPersistenceForProtocol>,
    ) -> Outcome<(
        NegotiationContinuationContext,
        Option<NegotiationProcessMessageWrapper<NegotiationAckMessageDto>>,
    )> {
        continuation_prepare_context(id, persistence).await
    }

    /// Advances the process state and creates the agreement record.
    async fn persist(
        persistence: &Arc<OrchestrationPersistenceForProtocol>,
        _id: &str,
        ctx: &NegotiationContinuationContext,
        input: &NegotiationProcessMessageWrapper<NegotiationAgreementMessageDto>,
        mate: &Mates,
    ) -> Outcome<NegotiationProcessDto> {
        persistence
            .update_with_new_agreement(ctx.id.as_str(), &input.dto, mate)
            .await
    }
}
