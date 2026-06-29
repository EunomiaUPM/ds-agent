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
use crate::protocols::dsp::protocol_types::DataAddressDto;
use connector::ConnectorInstanceDto;
use urn::Urn;
use ymir::data::entities::shared::participant::Model as Mates;

/// Unified DSP transfer context.
#[derive(Debug, Clone)]
pub struct DspTransferContext {
    // ── Set at the HTTP boundary ──────────────────────────────────────────────
    /// Peer-facing process PID from the URL path `/{id}`.
    /// `None` for the request step (no prior process exists).
    /// `Some` for all continuation steps (inbound protocol path).
    pub peer_pid: Option<Urn>,

    /// Participant ID of the remote agent.
    /// Inbound: from the `Mates` auth extension.
    /// Outbound: from `associated_agent_peer` in the RPC input.
    pub associated_peer_id: String,

    /// Full `Mates` record for the remote agent.
    /// `Some` on the inbound path (available from auth middleware).
    /// `None` on the outbound RPC path.
    pub associated_peer: Option<Mates>,

    // ── Populated in prepare_context ─────────────────────────────────────────
    /// Full persisted process record. `None` until the first DB interaction.
    /// Updated in-place on every state transition.
    pub process: Option<TransferProcessDto>,

    /// Consumer-side DSP process identifier.
    pub consumer_pid: Option<Urn>,

    /// Provider-side DSP process identifier.
    pub provider_pid: Option<Urn>,

    /// The peer's PID as a plain string for URL-path routing:
    /// `{provider_address}/transfers/{peer_url_id}/{suffix}`
    pub peer_url_id: Option<String>,

    /// Connector instance resolved from the agreement. Just being provider
    pub connector_instance: Option<ConnectorInstanceDto>,

    /// Pre-allocated local process ID generated before the first HTTP send.
    pub local_process_id: Option<Urn>,

    // ── Data-plane negotiation ────────────────────────────────────────────────
    /// Data address contributed by the message sender.
    /// PUSH mode request: consumer's ingest URL.
    /// `None` in PULL mode request.
    pub input_data_address: Option<DataAddressDto>,

    /// Data address produced by our local dataplane (proxy / ingress URL).
    pub resolved_data_address: Option<DataAddressDto>,

    // ── Outbound routing ──────────────────────────────────────────────────────
    /// Provider's base URL for outbound DSP messages (RPC path only).
    pub provider_address: Option<String>,

    // ── Lifecycle flags ───────────────────────────────────────────────────────
    /// `true` when resuming a previously suspended transfer.
    /// Prevents the dataplane from re-initialising an active session.
    pub is_restart: bool,
}

impl Default for DspTransferContext {
    fn default() -> Self {
        Self {
            peer_pid: None,
            associated_peer_id: String::new(),
            associated_peer: None,
            process: None,
            consumer_pid: None,
            provider_pid: None,
            peer_url_id: None,
            connector_instance: None,
            local_process_id: None,
            input_data_address: None,
            resolved_data_address: None,
            provider_address: None,
            is_restart: false,
        }
    }
}

impl DspTransferContext {
    /// Create the initial context for an **inbound** DSP message.
    pub fn inbound(
        peer_pid: Option<Urn>,
        associated_peer_id: String,
        associated_peer: Mates,
    ) -> Self {
        Self {
            peer_pid,
            associated_peer_id,
            associated_peer: Some(associated_peer),
            ..Default::default()
        }
    }

    /// Create the initial context for an **outbound** RPC transfer request.
    pub fn outbound_request(
        associated_peer_id: String,
        provider_address: String,
        input_data_address: Option<DataAddressDto>,
    ) -> Self {
        Self {
            associated_peer_id,
            provider_address: Some(provider_address),
            input_data_address,
            ..Default::default()
        }
    }

    /// Create the initial context for an **outbound** RPC continuation step.
    /// `prepare_context` populates all remaining fields from the database.
    pub fn outbound_continuation() -> Self {
        Self::default()
    }
}
