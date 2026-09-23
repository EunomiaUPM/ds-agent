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

pub(crate) mod persistence_rpc;
pub(crate) mod process_resolver;

use crate::protocols::dsp::orchestrator::rpc::types::RpcNegotiationProcessMessageTrait;
use crate::protocols::dsp::protocol_types::NegotiationProcessMessageTrait;
use crate::services::negotiation_process::views::NegotiationProcessView;
use crate::services::offer::views::OfferView;
use common::dsp_common::DspActor;
use ymir::errors::Outcome;

// Design notes ─────────────────────────────────────────────────────────────
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
//   INBOUND  (protocol) - peer sent the message; local agent responds
//   OUTBOUND (RPC)      - local agent initiates; peer acknowledges

// Trait ────────────────────────────────────────────────────────────────────

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
    /// Loads the process behind a pid, provided `actor` may act on it.
    async fn fetch_process(&self, id: &str, actor: &DspActor) -> Outcome<NegotiationProcessView>;
    async fn fetch_last_offer(&self, process: &NegotiationProcessView) -> Outcome<OfferView>;
    /// Records a process the local user opened; `tenant_id` is the target peer's, already checked.
    async fn create_new(
        &self,
        tenant_id: &str,
        payload: &dyn RpcNegotiationProcessMessageTrait,
        request: &dyn NegotiationProcessMessageTrait,
        response: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<NegotiationProcessView>;
    async fn update(
        &self,
        process: &NegotiationProcessView,
        payload: &dyn RpcNegotiationProcessMessageTrait,
        request: &dyn NegotiationProcessMessageTrait,
        response: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<NegotiationProcessView>;
    async fn update_with_offer(
        &self,
        process: &NegotiationProcessView,
        payload: &dyn RpcNegotiationProcessMessageTrait,
        request: &dyn NegotiationProcessMessageTrait,
        response: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<NegotiationProcessView>;
    async fn update_with_new_agreement(
        &self,
        process: &NegotiationProcessView,
        payload: &dyn RpcNegotiationProcessMessageTrait,
        request: &dyn NegotiationProcessMessageTrait,
        response: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<NegotiationProcessView>;
    async fn update_with_agreement(
        &self,
        process: &NegotiationProcessView,
        payload: &dyn RpcNegotiationProcessMessageTrait,
        request: &dyn NegotiationProcessMessageTrait,
        response: &dyn NegotiationProcessMessageTrait,
    ) -> Outcome<NegotiationProcessView>;
}
