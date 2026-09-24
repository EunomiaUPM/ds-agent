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

use crate::protocols::dsp::orchestrator::rpc::types::RpcNegotiationProcessMessageTrait;
use crate::protocols::dsp::persistence::NegotiationRpcPersistenceTrait;
use crate::protocols::dsp::protocol_types::{
    NegotiationAckMessageDto, NegotiationProcessMessageWrapper,
};
use crate::protocols::dsp::validator::traits::validation_rpc_steps::ValidationRpcSteps;
use crate::services::negotiation_process::views::NegotiationProcessView;
use axum::http::HeaderMap;
use common::auth::AccessScope;
use common::dsp_common::DspActor;
use common::dsp_common::odrl::OdrlMessageOffer;
use common::facades::ssi_auth_facade::MatesFacadeTrait;
use std::fmt::Debug;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::{Errors, Outcome};
use ymir::utils::bearer_headers;

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
    /// Tenant owning the target peer; the new process belongs to it.
    pub tenant_id: String,
}

impl NegotiationRpcInitialContext {
    /// Checks that the user may negotiate with `associated_peer`, which must be registered in the
    /// user's acting tenant; any other peer answers as not found.
    pub(super) async fn resolve(
        scope: &AccessScope,
        provider_address: String,
        associated_peer: String,
        mates_service: &Arc<dyn MatesFacadeTrait>,
    ) -> Outcome<Self> {
        scope.require_write()?;
        let peer = mates_service
            .get_mate_by_id(scope.acting_tenant().clone(), associated_peer.clone())
            .await?;
        Ok(Self {
            provider_address,
            associated_peer,
            tenant_id: peer.tenant_id,
        })
    }
}

/// Routing context for continuation steps that operate on an existing process.
///
/// Populated by [`resolve_continuation_context`] from the database record
/// identified by the consumer PID supplied in the RPC input.
#[derive(Debug)]
pub(super) struct NegotiationRpcContinuationContext {
    /// Full process record as stored in the database.
    pub process: NegotiationProcessView,
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
    pub process: NegotiationProcessView,
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
        _actor: &DspActor,
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
        scope: &AccessScope,
        input: &Self::Input,
        persistence: &Arc<dyn NegotiationRpcPersistenceTrait>,
        mates_service: &Arc<dyn MatesFacadeTrait>,
    ) -> Outcome<Self::Context>;

    /// Return the tenant and peer identifier used for auth-token lookup.
    fn auth_peer(ctx: &Self::Context) -> (&str, &str);

    /// Build the DSP message, POST it to the peer, and persist the resulting
    /// state transition.
    ///
    /// Initial steps build the peer URL from `{provider_address}/negotiations/…`
    /// and call `persistence.create_new`.  Continuation steps call the
    /// appropriate `update*` variant on [`NegotiationRpcPersistenceTrait`].
    async fn send_and_persist(
        headers: Option<HeaderMap>,
        persistence: &Arc<dyn NegotiationRpcPersistenceTrait>,
        ctx: &Self::Context,
        input: &Self::Input,
    ) -> Outcome<(
        NegotiationProcessMessageWrapper<NegotiationAckMessageDto>,
        NegotiationProcessView,
    )>;

    // Default helpers ──────────────────────────────────────────────────────

    /// Bearer headers carrying the peer's stored token, sent with this request only.
    ///
    /// `None` when the peer has no token; the request proceeds unauthenticated.
    async fn peer_headers(
        mates_service: &Arc<dyn MatesFacadeTrait>,
        (tenant_id, peer): (&str, &str),
    ) -> Outcome<Option<HeaderMap>> {
        match mates_service
            .get_mate_by_id(tenant_id.to_string(), peer.to_string())
            .await
        {
            Ok(mate) => mate.token.as_deref().map(bearer_headers).transpose(),
            Err(_) => Ok(None),
        }
    }
}

// Shared helpers for continuation steps ────────────────────────────────────

impl NegotiationRpcContinuationContext {
    /// Fetch the process on behalf of the user and derive the peer routing fields.
    ///
    /// The `consumer_pid` is the local agent's identifier; a process outside the user's
    /// tenants answers as not found.
    pub(super) async fn resolve(
        consumer_pid: &Urn,
        scope: &AccessScope,
        persistence: &Arc<dyn NegotiationRpcPersistenceTrait>,
    ) -> Outcome<Self> {
        scope.require_write()?;
        let process = persistence
            .fetch_process(consumer_pid.to_string().as_str(), &DspActor::user(scope))
            .await?;

        // The outgoing URL uses the *peer's* identifier (opposite of the local role).
        let peer_role_key = match process.inner.role.as_str() {
            "Provider" => "consumerPid",
            _ => "providerPid",
        };
        let peer_identifier = process.identifiers.get(peer_role_key).unwrap().clone();
        let peer_address = process.inner.callback_address.clone().unwrap_or_default();

        Ok(Self {
            process,
            peer_identifier,
            peer_address,
        })
    }
}
