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

use crate::entities::transfer_process::TransferProcessDto;
use crate::protocols::dsp::facades::dataplane_facade::DataPlaneFacadeTrait;
use crate::protocols::dsp::facades::FacadeTrait;
use crate::protocols::dsp::persistence::TransferPersistenceTrait;
use crate::protocols::dsp::protocol_types::{
    DataAddressDto, TransferProcessAckDto, TransferProcessMessageTrait,
    TransferProcessMessageWrapper,
};
use crate::protocols::dsp::validator::traits::validation_dsp_steps::ValidationDspSteps;
use connector::ConnectorInstanceDto;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::Outcome;

/// Resolved routing state for an inbound continuation message
/// Populated by [`continuation_prepare_context`] from the peer-facing process
/// identifier in the URL path.
pub(super) struct ProtocolContext {
    /// Canonical internal process identifier (URN), resolved from the peer-facing PID.
    pub process: Option<TransferProcessDto>,
    /// Only populated for the request step (resolved from agreement → distribution → connector).
    /// `None` for all continuation steps.
    pub connector_instance: Option<ConnectorInstanceDto>,
    pub associated_peer: String,
}

// ─── Lifecycle step template ──────────────────────────────────────────────────

/// Template trait for a single inbound DSP protocol lifecycle step.
///
/// Five concrete steps implement this trait: [`ProtocolRequestStep`] for the
/// initial `TransferRequestMessage`, and four continuation steps
/// (`StartStep`, `SuspensionStep`, `CompletionStep`, `TerminationStep`) for the
/// subsequent lifecycle messages.  The shared algorithm lives in
/// [`ProtocolOrchestratorService::run_lifecycle`]:
///
/// 1. **`validate`**         – reject malformed or out-of-sequence input
/// 2. **`prepare_context`**  – resolve routing info; may return an early ack (idempotency)
/// 3. **`persist`**          – record the state transition in the database
/// 4. **`post_hook`**        – trigger the local dataplane; may return a `DataAddress`
#[async_trait::async_trait]
pub(super) trait ProtocolStep: Send + Sync + 'static {
    /// DSP message DTO type carried by the inbound `TransferProcessMessageWrapper`.
    type Dto: TransferProcessMessageTrait + Clone + serde::Serialize + Send + Sync + 'static;
    /// Step-specific routing context produced by `prepare_context`.
    type Context: Send + Sync + 'static;

    /// Validate the inbound message before any I/O.
    ///
    /// `id` is the peer-facing process identifier from the URL path.
    /// For the request step, `id` is unused (pass `""`).
    async fn validate(
        validator: &Arc<dyn ValidationDspSteps>,
        id: &str,
        input: &TransferProcessMessageWrapper<Self::Dto>,
    ) -> Outcome<()>;

    /// Resolve or build the routing context for this step.
    ///
    /// Returns `(context, None)` to proceed normally, or `(context, Some(ack))` to
    /// short-circuit with an existing ack (used by the request step for idempotency).
    ///
    /// `id` is the peer-facing PID (continuation steps).
    /// `peer` is the calling agent identifier (request step).
    async fn prepare_context(
        id: &str,
        peer: &str,
        input: &TransferProcessMessageWrapper<Self::Dto>,
        persistence: &Arc<dyn TransferPersistenceTrait>,
        facades: &Arc<dyn FacadeTrait>,
    ) -> Outcome<(
        Self::Context,
        Option<TransferProcessMessageWrapper<TransferProcessAckDto>>,
    )>;

    /// Persist the inbound message as a state transition.
    ///
    /// Continuation steps call [`continuation_persist`].
    /// The request step calls `create_process`.
    async fn persist(
        persistence: &Arc<dyn TransferPersistenceTrait>,
        id: &str,
        ctx: &Self::Context,
        input: &TransferProcessMessageWrapper<Self::Dto>,
    ) -> Outcome<TransferProcessDto>;

    /// Trigger the local dataplane after persisting the state transition.
    ///
    /// `process` is the record just returned by `persist` — passed directly so
    /// steps do not need a second database round-trip to fetch it.
    ///
    /// Returns an optional `DataAddressDto` to embed in the ack.  Only the Start
    /// step returns a non-`None` value (the consumer's ingress URL in PULL mode).
    async fn post_hook(
        dp: &Arc<dyn DataPlaneFacadeTrait>,
        ctx: &Self::Context,
        input: &TransferProcessMessageWrapper<Self::Dto>,
        process: &TransferProcessDto,
    ) -> Outcome<Option<DataAddressDto>>;
}

// ─── Shared helpers for continuation steps ────────────────────────────────────

/// Resolve the canonical internal process from a peer-facing identifier.
///
/// DSP endpoints receive a `providerPid` (or `consumerPid`) in the URL path;
/// this function looks it up in the identifiers index and returns the local URN.
pub(super) async fn resolve_process(
    peer_id: &str,
    persistence: &Arc<dyn TransferPersistenceTrait>,
) -> Outcome<TransferProcessDto> {
    let dpid = Urn::from_str(peer_id)?;
    let process = persistence
        .get_transfer_process_service()
        .await?
        .get_transfer_process_by_key_value(&dpid)
        .await?;
    Ok(process)
}

/// Build the continuation context by resolving the process ID from the peer-facing id.
///
/// Called by all four continuation steps from their `prepare_context` implementations.
pub(super) async fn continuation_prepare_context(
    id: &str,
    persistence: &Arc<dyn TransferPersistenceTrait>,
) -> Outcome<(
    ProtocolContext,
    Option<TransferProcessMessageWrapper<TransferProcessAckDto>>,
)> {
    let process = resolve_process(id, persistence).await?;
    Ok((
        ProtocolContext {
            process: Some(process),
            connector_instance: None,
            associated_peer: "".to_string(),
        },
        None,
    ))
}

/// Persist an inbound continuation message as a state-transition update.
///
/// Called by all four continuation steps from their `persist` implementations.
pub(super) async fn continuation_persist<T>(
    persistence: &Arc<dyn TransferPersistenceTrait>,
    id: &str,
    input: &TransferProcessMessageWrapper<T>,
) -> Outcome<TransferProcessDto>
where
    T: TransferProcessMessageTrait + Clone + serde::Serialize + Send + Sync + 'static,
{
    persistence
        .update_process(
            id,
            Arc::new(input.dto.clone()),
            serde_json::to_value(input).unwrap(),
        )
        .await
}
