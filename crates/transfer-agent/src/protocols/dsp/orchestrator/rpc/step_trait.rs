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

use crate::entities::transfer_process::TransferProcessDto;
use crate::protocols::dsp::facades::dataplane_facade::DataPlaneFacadeTrait;
use crate::protocols::dsp::orchestrator::rpc::types::RpcTransferProcessMessageTrait;
use crate::protocols::dsp::persistence::TransferPersistenceTrait;
use crate::protocols::dsp::protocol_types::{
    DataAddressDto, TransferProcessAckDto, TransferProcessMessageTrait, TransferProcessMessageWrapper,
};
use crate::protocols::dsp::validator::traits::validation_rpc_steps::ValidationRpcSteps;
use common::dsp_common::context_field::ContextField;
use common::facades::ssi_auth_facade::MatesFacadeTrait;
use common::http_client::HttpClient;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::Outcome;
// ─── Peer context (continuation steps) ───────────────────────────────────────

/// Resolved routing state for an in-progress transfer process.
///
/// Used by the four continuation steps (start / suspension / completion /
/// termination).  Populated by [`resolve_continuation_context`] from the
/// database record identified by `consumer_pid`.
pub(super) struct RpcPeerContext {
    /// Full process record as stored in the database.
    pub process: TransferProcessDto,
    /// Canonical internal process identifier (URN).
    pub process_id: Urn,
    /// DSP `providerPid` identifier.
    pub provider_pid: Urn,
    /// DSP `consumerPid` identifier.
    pub consumer_pid: Urn,
    /// The identifier placed in the outgoing URL (the *peer's* PID).
    pub peer_url_id: String,
    /// `true` when the process has already left the initial REQUEST phase.
    pub is_restart: bool,
}

// ─── Request context (request step) ──────────────────────────────────────────

/// Routing state for a brand-new transfer process not yet in the database.
///
/// Used exclusively by [`RequestStep`] before any process record exists.
pub(super) struct RpcRequestContext {
    /// Pre-allocated local process identifier (URN), generated before the HTTP call.
    pub process_id: Urn,
    /// Provider's base address; used to build the outgoing URL.
    pub provider_address: String,
    /// Remote peer identifier; used for auth-token lookup.
    pub associated_peer: String,
    /// Consumer-supplied data address (for PUSH mode), stored here so `pre_hook`
    /// can access it without needing the raw input.
    pub input_data_address: Option<DataAddressDto>,
}

// ─── Lifecycle step template ──────────────────────────────────────────────────

/// Template trait for a single RPC-initiated transfer lifecycle step.
///
/// Both the initial request and the four continuation steps (start / suspension /
/// completion / termination) implement this trait.  The shared orchestration
/// algorithm lives in [`RPCOrchestratorService::run_lifecycle`] and executes:
///
/// 1. **`validate`**          – reject malformed input early (default: no-op)
/// 2. **`prepare_context`**   – resolve or allocate routing state
/// 3. **`pre_hook`**          – notify local dataplane before contacting peer
/// 4. **`build_message`**     – produce the outgoing DSP payload
/// 5. **`apply_auth_token`**  – attach bearer token to the HTTP client (default)
/// 6. **`send_and_persist`**  – POST to peer and persist the state transition
/// 7. **`post_hook`**         – notify local dataplane after peer acknowledges (default: no-op)
#[async_trait::async_trait]
pub(super) trait TransferRpcStep: Send + Sync + 'static {
    /// Raw RPC input type this step handles.
    type Input: RpcTransferProcessMessageTrait + Clone + Send + Sync + 'static;
    /// DSP protocol message type produced by `build_message`.
    type DspMessage: TransferProcessMessageTrait + Clone + serde::Serialize + Send + Sync + 'static;
    /// Step-specific routing context produced by `prepare_context`.
    type Context: Send + Sync + 'static;

    /// URL path suffix appended to the peer transfers path.
    fn url_suffix() -> &'static str;

    /// Optional input validation executed before any I/O.  Default: no-op.
    async fn validate(
        _validator: &Arc<dyn ValidationRpcSteps>,
        _input: &Self::Input,
    ) -> Outcome<()> {
        Ok(())
    }

    /// Resolve or allocate the routing context for this step.
    ///
    /// Continuation steps fetch an existing process from the database.
    /// The request step generates a fresh process ID and reads routing info
    /// directly from the input.
    async fn prepare_context(
        input: &Self::Input,
        persistence: &Arc<dyn TransferPersistenceTrait>,
    ) -> Outcome<Self::Context>;

    /// Local dataplane hook executed **before** sending the message to the peer.
    ///
    /// Returns an optional `DataAddressDto` forwarded to `build_message`.
    /// Only the Start and Request steps use this value; all others return `None`.
    async fn pre_hook(
        dp: &Arc<dyn DataPlaneFacadeTrait>,
        ctx: &Self::Context,
    ) -> Outcome<Option<DataAddressDto>>;

    /// Produce the outgoing DSP protocol message from the input and context.
    fn build_message(
        input: &Self::Input,
        ctx: &Self::Context,
        pre_addr: Option<DataAddressDto>,
    ) -> Outcome<Self::DspMessage>;

    /// Return the peer identifier string used for auth-token lookup.
    fn auth_peer(ctx: &Self::Context) -> &str;

    /// POST the message to the peer and persist the resulting state transition.
    ///
    /// Continuation steps call [`continuation_send_and_persist`].
    /// The request step uses [`CreateProcessInput`] to create a new record.
    async fn send_and_persist(
        http_client: &HttpClient,
        persistence: &Arc<dyn TransferPersistenceTrait>,
        ctx: &Self::Context,
        payload: Arc<Self::DspMessage>,
        url_suffix: &str,
    ) -> Outcome<(TransferProcessMessageWrapper<TransferProcessAckDto>, TransferProcessDto)>;

    /// Local dataplane hook executed **after** the peer acknowledges the message.
    /// Default: no-op (overridden by steps that need to react to the peer's ack).
    async fn post_hook(
        _dp: &Arc<dyn DataPlaneFacadeTrait>,
        _ctx: &Self::Context,
    ) -> Outcome<()> {
        Ok(())
    }

    // ─── Shared helpers ───────────────────────────────────────────────────────

    /// Attach the peer's stored bearer token to the HTTP client.
    ///
    /// Silently skips if the peer has no token; the request proceeds
    /// unauthenticated.
    async fn apply_auth_token(
        mates_facade: &Arc<dyn MatesFacadeTrait>,
        http_client: &HttpClient,
        peer: &str,
    ) {
        if let Ok(mate) = mates_facade.get_mate_by_id(peer.to_string()).await {
            if let Some(token) = mate.token {
                http_client.set_auth_token(token).await;
            }
        }
    }
}

// ─── Shared helpers for continuation steps ────────────────────────────────────

/// Fetch the transfer process and derive all peer-routing fields.
///
/// Called by all four continuation steps from their `prepare_context`
/// implementations.  The `consumer_pid` is the local agent's identifier;
/// the method resolves the DB record and extracts both DSP PIDs plus the
/// peer URL identifier (the opposite party's PID).
pub(super) async fn resolve_continuation_context(
    consumer_pid: &Urn,
    persistence: &Arc<dyn TransferPersistenceTrait>,
) -> Outcome<RpcPeerContext> {
    let process = persistence.fetch_process(consumer_pid.to_string().as_str()).await?;

    let provider_pid = Urn::from_str(process.identifiers.get("providerPid").unwrap().as_str())?;
    let consumer_pid_resolved =
        Urn::from_str(process.identifiers.get("consumerPid").unwrap().as_str())?;

    // Outgoing URL embeds the *peer's* identifier, not the local one.
    let identifier_key = match process.inner.role.as_str() {
        "Provider" => "consumerPid",
        _ => "providerPid",
    };
    let peer_url_id = process.identifiers.get(identifier_key).unwrap().clone();
    let process_id = Urn::from_str(process.inner.id.as_str())?;
    let is_restart = process.inner.state_attribute.as_deref().unwrap_or("") != "OnRequest";

    Ok(RpcPeerContext {
        process,
        process_id,
        provider_pid,
        consumer_pid: consumer_pid_resolved,
        peer_url_id,
        is_restart,
    })
}

/// Wrap, POST, and persist one outbound transfer message for a continuation step.
///
/// Constructs the DSP JSON envelope, sends it to the peer, and records the
/// resulting state transition via `update_process`.
pub(super) async fn continuation_send_and_persist<T>(
    http_client: &HttpClient,
    persistence: &Arc<dyn TransferPersistenceTrait>,
    ctx: &RpcPeerContext,
    payload: Arc<T>,
    url_suffix: &str,
) -> Outcome<(TransferProcessMessageWrapper<TransferProcessAckDto>, TransferProcessDto)>
where
    T: TransferProcessMessageTrait + Clone + serde::Serialize + Send + Sync + 'static,
{
    let callback = ctx.process.inner.callback_address.clone().unwrap_or_default();
    let peer_url = format!("{}/transfers/{}/{}", callback, ctx.peer_url_id, url_suffix);

    let message = TransferProcessMessageWrapper {
        context: ContextField::default(),
        _type: payload.get_message(),
        dto: payload.as_ref().clone(),
    };

    let response: TransferProcessMessageWrapper<TransferProcessAckDto> =
        http_client.post_json(peer_url.as_str(), &message).await?;

    let updated = persistence
        .update_process(
            ctx.process.inner.id.as_str(),
            payload,
            serde_json::to_value(message).unwrap(),
        )
        .await?;

    Ok((response, updated))
}
