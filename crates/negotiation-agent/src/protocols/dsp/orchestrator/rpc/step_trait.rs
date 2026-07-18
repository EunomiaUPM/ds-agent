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
use crate::protocols::dsp::orchestrator::rpc::types::RpcNegotiationProcessMessageTrait;
use crate::protocols::dsp::persistence::NegotiationRpcPersistenceTrait;
use crate::protocols::dsp::protocol_types::{
    NegotiationAckMessageDto, NegotiationProcessMessageWrapper,
};
use crate::protocols::dsp::validator::traits::validation_rpc_steps::ValidationRpcSteps;
use common::config::types::roles::RoleConfig;
use common::dsp_common::odrl::OdrlMessageOffer;
use common::facades::ssi_auth_facade::MatesFacadeTrait;
use common::http_client::HttpClient;
use std::fmt::Debug;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::Outcome;

// Contexts ─────────────────────────────────────────────────────────────────

/// Routing context for steps that create a brand-new negotiation process
/// (initial request and initial offer).
///
/// No process record exists yet; all routing information is read directly from
/// the RPC input.
#[derive(Debug)]
pub(super) struct NegotiationRpcInitialContext {
    /// Provider's base URL; used to build the outgoing HTTP endpoint.
    pub provider_address: String,
    /// Remote peer identifier; used for auth-token lookup.
    pub associated_peer: String,
}

/// Routing context for continuation steps that operate on an existing process.
///
/// Populated by [`resolve_continuation_context`] from the database record
/// identified by the consumer PID supplied in the RPC input.
#[derive(Debug)]
pub(super) struct NegotiationRpcContinuationContext {
    /// Full process record as stored in the database.
    pub process: NegotiationProcessDto,
    /// The identifier placed in the outgoing URL (the *peer's* PID, i.e. the
    /// opposite of the local role).
    pub peer_identifier: String,
    /// Full callback address of the remote peer.
    pub peer_address: String,
}

/// Extended routing context for the agreement step.
///
/// In addition to the standard continuation fields this context pre-fetches the
/// offer content and the participant IDs required to construct the agreement
/// body.  These lookups are performed in [`prepare_context`] so that
/// `send_and_persist` remains a simple POST + persist call.
#[derive(Debug)]
pub(super) struct NegotiationRpcAgreementContext {
    /// Full process record as stored in the database.
    pub process: NegotiationProcessDto,
    /// The identifier placed in the outgoing URL (peer's PID).
    pub peer_identifier: String,
    /// Full callback address of the remote peer.
    pub peer_address: String,
    /// Participant ID of the local agent (Provider), used as ODRL `assigner`.
    pub assigner: String,
    /// Participant ID of the remote peer (Consumer), used as ODRL `assignee`.
    pub assignee: String,
    /// The last offer stored for this process; its policy fields are copied into
    /// the outgoing agreement.
    pub last_offer: OdrlMessageOffer,
}

// Lifecycle step template ──────────────────────────────────────────────────

/// Template trait for a single RPC-initiated negotiation lifecycle step.
///
/// Nine concrete steps implement this trait: two initial steps that create a new
/// process ([`RpcRequestInitStep`] and [`RpcOfferInitStep`]) and seven
/// continuation steps that update an existing one.  The shared orchestration
/// algorithm lives in [`RPCOrchestratorService::run_lifecycle`] and executes:
///
/// 1. **`validate`**         – reject malformed input early (default: no-op)
/// 2. **`prepare_context`**  – resolve or allocate routing state (may access persistence and mates
///    service)
/// 3. **`apply_auth_token`** – attach bearer token to the HTTP client (default)
/// 4. **`send_and_persist`** – build the DSP message, POST to peer, and persist the state
///    transition
///
/// Unlike the transfer RPC template, there is no `pre_hook` / `post_hook`
/// because negotiation does not involve a data-plane session.
#[async_trait::async_trait]
pub(super) trait NegotiationRpcStep: Send + Sync + 'static {
    /// Raw RPC input type this step handles.
    type Input: RpcNegotiationProcessMessageTrait + Clone + Send + Sync + 'static;
    /// Step-specific routing context produced by `prepare_context`.
    type Context: Send + Sync + Debug + 'static;

    /// Optional input validation executed before any I/O.  Default: no-op.
    async fn validate(
        _validator: &Arc<dyn ValidationRpcSteps>,
        _input: &Self::Input,
    ) -> Outcome<()> {
        Ok(())
    }

    /// Resolve or allocate the routing context for this step.
    ///
    /// Continuation steps fetch an existing process from `persistence`.
    /// Initial steps read routing info directly from the input.
    /// The agreement step additionally fetches the last offer and participant
    /// IDs from `mates_service`.
    async fn prepare_context(
        input: &Self::Input,
        persistence: &Arc<dyn NegotiationRpcPersistenceTrait>,
        mates_service: &Arc<dyn MatesFacadeTrait>,
    ) -> Outcome<Self::Context>;

    /// Return the peer identifier string used for auth-token lookup.
    fn auth_peer(ctx: &Self::Context) -> &str;

    /// Build the DSP message, POST it to the peer, and persist the resulting
    /// state transition.
    ///
    /// Initial steps build the peer URL from `{provider_address}/negotiations/…`
    /// and call `persistence.create_new`.  Continuation steps call the
    /// appropriate `update*` variant on [`NegotiationRpcPersistenceTrait`].
    async fn send_and_persist(
        http_client: &HttpClient,
        persistence: &Arc<dyn NegotiationRpcPersistenceTrait>,
        ctx: &Self::Context,
        input: &Self::Input,
    ) -> Outcome<(
        NegotiationProcessMessageWrapper<NegotiationAckMessageDto>,
        NegotiationProcessDto,
    )>;

    // Default helpers ──────────────────────────────────────────────────────

    /// Attach the peer's stored bearer token to the HTTP client.
    ///
    /// Silently skips if the peer has no token; the request proceeds
    /// unauthenticated.
    async fn apply_auth_token(
        mates_service: &Arc<dyn MatesFacadeTrait>,
        http_client: &HttpClient,
        peer: &str,
    ) {
        if let Ok(mate) = mates_service.get_mate_by_id(peer.to_string()).await {
            if let Some(token) = mate.token {
                http_client.set_auth_token(token).await;
            }
        }
    }
}

// Shared helpers for continuation steps ────────────────────────────────────

/// Fetch the negotiation process and derive the peer routing fields.
///
/// Called by all seven continuation steps from their `prepare_context`
/// implementations.  The `consumer_pid` is the local agent's identifier; the
/// method resolves the DB record and extracts the peer's PID and callback
/// address for use in the outgoing HTTP request.
pub(super) async fn resolve_continuation_context(
    consumer_pid: &Urn,
    persistence: &Arc<dyn NegotiationRpcPersistenceTrait>,
) -> Outcome<NegotiationRpcContinuationContext> {
    let process = persistence
        .fetch_process(consumer_pid.to_string().as_str())
        .await?;

    // The outgoing URL uses the *peer's* identifier (opposite of the local role).
    let peer_role_key = match process.inner.role.as_str() {
        "Provider" => "consumerPid",
        _ => "providerPid",
    };
    let peer_identifier = process.identifiers.get(peer_role_key).unwrap().clone();
    let peer_address = process.inner.callback_address.clone().unwrap_or_default();

    Ok(NegotiationRpcContinuationContext {
        process,
        peer_identifier,
        peer_address,
    })
}
