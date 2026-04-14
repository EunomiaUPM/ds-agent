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
    NegotiationInitialContext, NegotiationProtocolStep,
};
use crate::protocols::dsp::protocol_types::{
    NegotiationAckMessageDto, NegotiationProcessMessageWrapper, NegotiationRequestInitMessageDto,
};
use crate::protocols::dsp::validator::traits::validation_dsp_steps::ValidationDspSteps;
use common::facades::Mates;
use std::sync::Arc;
use ymir::errors::Outcome;
// ─── InitialContractRequestStep ───────────────────────────────────────────────

/// Handles an inbound `ContractRequestMessage` that initiates a new negotiation
/// process (Consumer → Provider, first message of the request flow).
///
/// This is one of the two steps that creates a new process record.  The
/// algorithm is:
/// 1. Validate the message.
/// 2. No routing pre-processing needed (new process, no existing PID to look up).
/// 3. Persist the new process together with the initial consumer offer.
pub(super) struct InitialContractRequestStep;

#[async_trait::async_trait]
impl NegotiationProtocolStep for InitialContractRequestStep {
    type Dto = NegotiationRequestInitMessageDto;
    type Context = NegotiationInitialContext;

    async fn validate(
        validator: &Arc<dyn ValidationDspSteps>,
        _id: &str,
        input: &NegotiationProcessMessageWrapper<NegotiationRequestInitMessageDto>,
        _mate: &Mates,
    ) -> Outcome<()> {
        validator.on_contract_request_init(input).await
    }

    /// No existing process to look up; always proceeds to persist.
    async fn prepare_context(
        _id: &str,
        _mate: &Mates,
        _input: &NegotiationProcessMessageWrapper<NegotiationRequestInitMessageDto>,
        _persistence: &Arc<OrchestrationPersistenceForProtocol>,
    ) -> Outcome<(
        NegotiationInitialContext,
        Option<NegotiationProcessMessageWrapper<NegotiationAckMessageDto>>,
    )> {
        Ok((NegotiationInitialContext, None))
    }

    /// Creates the new negotiation process record with the initial consumer offer.
    async fn persist(
        persistence: &Arc<OrchestrationPersistenceForProtocol>,
        _id: &str,
        _ctx: &NegotiationInitialContext,
        input: &NegotiationProcessMessageWrapper<NegotiationRequestInitMessageDto>,
        mate: &Mates,
    ) -> Outcome<NegotiationProcessDto> {
        persistence.create_new(&input.dto, mate).await
    }
}
