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

pub(crate) mod persistence_protocol;
pub(crate) mod persistence_rpc;

use crate::entities::negotiation_process::NegotiationProcessDto;
use crate::entities::offer::OfferDto;
use crate::protocols::dsp::orchestrator::rpc::types::RpcNegotiationProcessMessageTrait;
use crate::protocols::dsp::protocol_types::NegotiationProcessMessageTrait;
use ymir::errors::Outcome;

// ─── Design notes ─────────────────────────────────────────────────────────────
//
// This module provides the persistence abstraction for the **RPC orchestrator**
// (outbound, Consumer- or Provider-initiated messages).
//
// One concrete implementation exists:
//
// • `persistence_rpc::NegotiationPersistenceForRpcService` – handles outbound
//   DSP messages sent by the local agent via the internal RPC interface.
//   Messages are recorded as `OUTBOUND` and the state machine is advanced
//   from the local agent's perspective.
//
// The **protocol orchestrator** (inbound messages arriving on DSP HTTP endpoints)
// uses a separate concrete object, `OrchestrationPersistenceForProtocol`
// (in `orchestrator/protocol/persistence`), which has its own richer API.
//
// Direction symmetry mirrors the transfer agent:
//   INBOUND  (protocol) → peer sent the message; local agent responds
//   OUTBOUND (RPC)      → local agent initiates; peer acknowledges

// ─── Trait ────────────────────────────────────────────────────────────────────

/// Persistence contract for the outbound DSP negotiation RPC path.
///
/// Mirrors [`crate::protocols::dsp::persistence::TransferPersistenceTrait`] in
/// the transfer agent: the RPC orchestrator holds `Arc<dyn NegotiationRpcPersistenceTrait>`
/// so implementations can be swapped (e.g. for testing) without touching the
/// orchestration logic.
///
/// Each method corresponds to one category of state transition:
/// - `create_new`              — first message; creates the process record.
/// - `update`                  — state-only advance (accepted, termination, …).
/// - `update_with_offer`       — advance + attach a new offer record.
/// - `update_with_new_agreement` — advance + create a fresh agreement record.
/// - `update_with_agreement`   — advance + activate the existing agreement.
#[async_trait::async_trait]
pub trait NegotiationRpcPersistenceTrait: Send + Sync {
    /// Fetch a negotiation process by any of its known identifiers
    /// (process URN, consumerPid, or providerPid).
    async fn fetch_process(&self, id: &str) -> Outcome<NegotiationProcessDto>;

    /// Fetch the most recent offer stored for a process (by process URN).
    ///
    /// Used by the agreement step to copy offer policy fields into the
    /// outgoing `ContractAgreementMessage`.
    async fn fetch_last_offer_by_process(&self, id: &str) -> Outcome<OfferDto>;

    /// Persist a brand-new negotiation process from the initial outbound message.
    ///
    /// Creates the process record, the initial message log entry, and the
    /// first offer record.  The `response` ack is used to extract the
    /// peer-assigned PID and store it in the identifiers map.
    async fn create_new(
        &self,
        payload: &dyn RpcNegotiationProcessMessageTrait,
        request: &dyn NegotiationProcessMessageTrait,
        response: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<NegotiationProcessDto>;

    /// Advance the process state and log an outbound message.
    ///
    /// Used for steps that only change state (event-accepted, termination, …).
    async fn update(
        &self,
        identifier: &str,
        payload: &dyn RpcNegotiationProcessMessageTrait,
        request: &dyn NegotiationProcessMessageTrait,
        response: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<NegotiationProcessDto>;

    /// Advance the process state, log the message, and attach a new offer record.
    ///
    /// Used for continuation request / offer counter-offer steps.
    async fn update_with_offer(
        &self,
        identifier: &str,
        payload: &dyn RpcNegotiationProcessMessageTrait,
        request: &dyn NegotiationProcessMessageTrait,
        response: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<NegotiationProcessDto>;

    /// Advance the process state, log the message, and create a new agreement record.
    ///
    /// Used by the agreement step (Provider → Consumer).
    async fn update_with_new_agreement(
        &self,
        identifier: &str,
        payload: &dyn RpcNegotiationProcessMessageTrait,
        request: &dyn NegotiationProcessMessageTrait,
        response: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<NegotiationProcessDto>;

    /// Advance the process state, log the message, and activate the existing agreement.
    ///
    /// Used by the verification step (Consumer → Provider) and the
    /// event-finalized step (Provider → Consumer).
    async fn update_with_agreement(
        &self,
        identifier: &str,
        payload: &dyn RpcNegotiationProcessMessageTrait,
        request: &dyn NegotiationProcessMessageTrait,
        response: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<NegotiationProcessDto>;
}
