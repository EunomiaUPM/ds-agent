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

//! Canonical event adapters translating transfer-agent writes into typed `ymir-events` envelopes.

use events::schemas::dataspace::{
    MessageDirection, ProtocolMessagePayload, TransferTransitionPayload,
};
use events::{EventActor, EventContext, EventPayload, EventSinkTrait, TypedEventSink};
use tracing::warn;
use urn::Urn;
use ymir::oauth::Owner;

use crate::entities::transfer_message::Direction;
use crate::services::transfer_message::views::TransferMessageView;
use crate::services::transfer_process::views::TransferProcessView;

/// Stateless adapter emitting `transfer-agent` lifecycle and DSP protocol events.
pub struct TransferEvents;

impl TransferEvents {
    /// Emits a typed control-plane transfer transition event.
    pub async fn process(
        sink: &dyn EventSinkTrait,
        owner: Owner,
        view: &TransferProcessView,
        action: &str,
        actor: Option<EventActor>,
    ) {
        let transfer_id = view.id.to_string();
        let agreement_id = view.correlation.agreement_id.as_ref().map(ToString::to_string);
        let dataset_id = view
            .properties
            .get("datasetId")
            .or_else(|| view.properties.get("dataset_id"))
            .or_else(|| view.properties.get("target"))
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let transfer_type = view
            .properties
            .get("transferType")
            .or_else(|| view.properties.get("transfer_type"))
            .or_else(|| view.properties.get("format"))
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let counter_party_id = view
            .correlation
            .peer_participant_id
            .as_ref()
            .map(ToString::to_string);
        let error_reason = view
            .state_metadata
            .reason
            .as_ref()
            .filter(|r| !r.is_empty())
            .map(|r| r.join("; "))
            .or_else(|| view.error_details.as_ref().map(|v| v.to_string()));

        let payload = TransferTransitionPayload {
            transition: Self::transition_segment(action, view.state.0.as_str()),
            transfer_id: transfer_id.clone(),
            agreement_id: agreement_id.clone(),
            dataset_id: dataset_id.clone(),
            state: view.state.0.to_string(),
            previous_state: None,
            transfer_type,
            role: view.role.to_string(),
            counter_party_id: counter_party_id.clone(),
            protocol_event: view.state_metadata.attribute.clone(),
            error_reason,
        };
        let mut ctx = EventContext::for_owner(owner)
            .with_transfer_id(&transfer_id)
            .with_correlation(&transfer_id);
        if let Some(agreement) = agreement_id {
            ctx = ctx.with_agreement_id(agreement);
        }
        if let Some(dataset) = dataset_id {
            ctx = ctx.with_dataset_id(dataset);
        }
        if let Some(peer) = counter_party_id {
            ctx = ctx.with_peer_id(peer);
        }
        if let Some(a) = actor {
            ctx = ctx.with_actor(a);
        }
        Self::emit_typed(sink, &ctx, &payload).await;
    }

    /// Emits a typed termination/deletion event when a transfer process row is deleted.
    pub async fn process_deleted(
        sink: &dyn EventSinkTrait,
        owner: Owner,
        id: &Urn,
        actor: Option<EventActor>,
    ) {
        let payload = TransferTransitionPayload {
            transition: "terminated".to_string(),
            transfer_id: id.to_string(),
            agreement_id: None,
            dataset_id: None,
            state: "deleted".to_string(),
            previous_state: None,
            transfer_type: None,
            role: "process".to_string(),
            counter_party_id: None,
            protocol_event: Some("deleted".to_string()),
            error_reason: None,
        };
        let mut ctx = EventContext::for_owner(owner)
            .with_transfer_id(id.to_string())
            .with_correlation(id.to_string());
        if let Some(a) = actor {
            ctx = ctx.with_actor(a);
        }
        Self::emit_typed(sink, &ctx, &payload).await;
    }

    /// Emits a typed protocol message telemetry event for DSP transfer exchanges.
    pub async fn message(
        sink: &dyn EventSinkTrait,
        owner: Owner,
        view: &TransferMessageView,
        actor: Option<EventActor>,
    ) {
        let direction = match view.direction {
            Direction::Inbound => MessageDirection::Incoming,
            Direction::Outbound => MessageDirection::Outgoing,
        };
        let transfer_id = view.transfer_process_id.to_string();
        let payload = ProtocolMessagePayload {
            direction,
            protocol: view.protocol.to_string(),
            message_type: view.message_type.0.to_string(),
            process_kind: "transfer".to_string(),
            process_id: transfer_id.clone(),
            peer_id: None,
            peer_address: None,
            status: format!(
                "{} -> {}",
                view.state_transition_from, view.state_transition_to
            ),
            error_reason: None,
        };
        let mut ctx = EventContext::for_owner(owner)
            .with_transfer_id(&transfer_id)
            .with_correlation(&transfer_id);
        if let Some(a) = actor {
            ctx = ctx.with_actor(a);
        }
        Self::emit_typed(sink, &ctx, &payload).await;
    }

    /// Emits a typed protocol deletion event when a stored transfer message is removed.
    pub async fn message_deleted(
        sink: &dyn EventSinkTrait,
        owner: Owner,
        id: &Urn,
        actor: Option<EventActor>,
    ) {
        let payload = ProtocolMessagePayload {
            direction: MessageDirection::Incoming,
            protocol: "dsp".to_string(),
            message_type: "deleted".to_string(),
            process_kind: "transfer".to_string(),
            process_id: id.to_string(),
            peer_id: None,
            peer_address: None,
            status: "deleted".to_string(),
            error_reason: None,
        };
        let mut ctx = EventContext::for_owner(owner).with_correlation(id.to_string());
        if let Some(a) = actor {
            ctx = ctx.with_actor(a);
        }
        Self::emit_typed(sink, &ctx, &payload).await;
    }

    fn transition_segment(action: &str, state: &str) -> String {
        if action == "create" {
            return "requested".to_string();
        }
        match state.to_ascii_lowercase().as_str() {
            "requested" | "started" | "suspended" | "completed" | "terminated" => {
                state.to_ascii_lowercase()
            }
            _ => "updated".to_string(),
        }
    }

    async fn emit_typed<P: EventPayload>(
        sink: &dyn EventSinkTrait,
        ctx: &EventContext,
        payload: &P,
    ) {
        if let Err(err) = sink.emit(ctx, payload).await {
            warn!(topic = %payload.topic(), error = %err, "transfer event emit failed");
        }
    }
}
