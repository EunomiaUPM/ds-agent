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

use crate::entities::negotiation_process::NegotiationProcessDto;
use crate::protocols::dsp::orchestrator::protocol::persistence::OrchestrationPersistenceForProtocol;
use crate::protocols::dsp::protocol_types::{
    NegotiationAckMessageDto, NegotiationProcessMessageTrait, NegotiationProcessMessageWrapper,
};
use crate::protocols::dsp::validator::traits::validation_dsp_steps::ValidationDspSteps;
use std::sync::Arc;
use ymir::errors::Outcome;
use common::facades::Mates;
// ─── Contexts ─────────────────────────────────────────────────────────────────

/// Context for steps that create a new negotiation process (initial request and
/// initial offer).
///
/// Currently carries no routing state beyond what is already present in the
/// message DTO; the type exists to keep the lifecycle template uniform and to
/// leave room for future pre-processing (e.g. policy look-ups or idempotency
/// checks).
pub(super) struct NegotiationInitialContext;

/// Context for continuation steps that operate on an existing negotiation process.
///
/// `id` is the peer-facing process identifier (consumerPid or providerPid)
/// extracted from the HTTP path and verified to exist by
/// [`continuation_prepare_context`].  It is passed directly to the
/// [`OrchestrationPersistenceForProtocol`] `update*` methods.
pub(super) struct NegotiationContinuationContext {
    pub id: String,
}

// ─── Lifecycle step template ──────────────────────────────────────────────────

/// Template trait for a single inbound DSP negotiation protocol lifecycle step.
///
/// Eight concrete steps implement this trait: two initial steps that create a new
/// negotiation process ([`InitialContractRequestStep`] and
/// [`InitialProviderOfferStep`]), and six continuation steps that update an
/// existing one.  The shared algorithm lives in
/// [`ProtocolOrchestratorService::run_lifecycle`]:
///
/// 1. **`validate`**        – reject malformed or out-of-sequence input
/// 2. **`prepare_context`** – resolve routing info; may return an early ack
///                            (idempotency hook for future use)
/// 3. **`persist`**         – record the state transition in the database
///
/// Unlike the transfer agent's [`ProtocolStep`], there is no `post_hook` because
/// negotiation does not involve a data-plane session.
#[async_trait::async_trait]
pub(super) trait NegotiationProtocolStep: Send + Sync + 'static {
    /// DSP message DTO type carried by the inbound
    /// [`NegotiationProcessMessageWrapper`].
    type Dto: NegotiationProcessMessageTrait + Clone + Send + Sync + 'static;
    /// Step-specific routing context produced by `prepare_context`.
    type Context: Send + Sync + 'static;

    /// Validate the inbound message before any I/O.
    ///
    /// `id` is the peer-facing process identifier from the URL path.
    /// For initial steps, `id` is unused (pass `""`).
    async fn validate(
        validator: &Arc<dyn ValidationDspSteps>,
        id: &str,
        input: &NegotiationProcessMessageWrapper<Self::Dto>,
        mate: &Mates,
    ) -> Outcome<()>;

    /// Resolve or build the routing context for this step.
    ///
    /// Returns `(context, None)` to proceed normally, or `(context, Some(ack))`
    /// to short-circuit with an existing ack.  The early-ack path is reserved
    /// for idempotency handling; all current steps return `None`.
    ///
    /// `id` is the peer-facing PID (continuation steps; pass `""` for initial).
    async fn prepare_context(
        id: &str,
        mate: &Mates,
        input: &NegotiationProcessMessageWrapper<Self::Dto>,
        persistence: &Arc<OrchestrationPersistenceForProtocol>,
    ) -> Outcome<(
        Self::Context,
        Option<NegotiationProcessMessageWrapper<NegotiationAckMessageDto>>,
    )>;

    /// Persist the inbound message as a state transition.
    ///
    /// Initial steps call [`OrchestrationPersistenceForProtocol::create_new`].
    /// Continuation steps call the appropriate `update*` variant:
    /// - `update` – generic state advancement (verification, event ACCEPTED, termination)
    /// - `update_with_offer` – also records a new offer (consumer request, provider offer)
    /// - `update_with_new_agreement` – creates the agreement record (agreement reception)
    /// - `update_with_agreement` – activates the existing agreement (event FINALIZED)
    async fn persist(
        persistence: &Arc<OrchestrationPersistenceForProtocol>,
        id: &str,
        ctx: &Self::Context,
        input: &NegotiationProcessMessageWrapper<Self::Dto>,
        mate: &Mates,
    ) -> Outcome<NegotiationProcessDto>;
}

// ─── Shared helpers for continuation steps ────────────────────────────────────

/// Build the continuation context by verifying the process identified by `id` exists.
///
/// Called by all six continuation steps from their `prepare_context`
/// implementations.  If the process is not found the error propagates and
/// terminates the request before any state mutation occurs.
pub(super) async fn continuation_prepare_context(
    id: &str,
    persistence: &Arc<OrchestrationPersistenceForProtocol>,
) -> Outcome<(
    NegotiationContinuationContext,
    Option<NegotiationProcessMessageWrapper<NegotiationAckMessageDto>>,
)> {
    // Eagerly verify existence; propagate the error if the process is not found.
    persistence.fetch_process(id).await?;
    Ok((NegotiationContinuationContext { id: id.to_string() }, None))
}
