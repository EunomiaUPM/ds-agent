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

//! Canonical event adapters translating negotiation writes into typed `ymir-events` envelopes.

use events::schemas::dataspace::{
    AgreementLifecyclePayload, MessageDirection, NegotiationTransitionPayload,
    ProtocolMessagePayload,
};
use events::{
    EventActor, EventContext, EventIntegrity, EventPayload, EventSinkTrait, TypedEventSink,
};
use tracing::warn;
use urn::Urn;
use ymir::oauth::{OwnedTrait, Owner};

use crate::services::agreement::views::AgreementView;
use crate::services::negotiation_message::views::NegotiationMessageView;
use crate::services::negotiation_process::views::NegotiationProcessView;
use crate::services::offer::views::OfferView;

/// Stateless adapter emitting `negotiation-agent` events with correlation and policy digests.
pub struct NegotiationEvents;

impl NegotiationEvents {
    /// Emits a typed transition event when an ODRL offer is created inside a negotiation process.
    pub async fn offer_created(
        sink: &dyn EventSinkTrait,
        view: &OfferView,
        actor: Option<EventActor>,
    ) {
        let payload = NegotiationTransitionPayload {
            transition: "offer.created".to_string(),
            negotiation_id: view.inner.negotiation_agent_process_id.clone(),
            agreement_id: None,
            dataset_id: view
                .inner
                .offer_content
                .get("target")
                .or_else(|| view.inner.offer_content.get("odrl:target"))
                .and_then(|v| v.as_str())
                .map(str::to_string),
            state: "offer".to_string(),
            previous_state: None,
            role: "offer".to_string(),
            counter_party_id: None,
            counter_party_address: None,
            protocol_event: Some(view.inner.offer_id.clone()),
            error_reason: None,
        };
        let mut ctx = EventContext::for_owner(view.inner.owner())
            .with_negotiation_id(&view.inner.negotiation_agent_process_id)
            .with_correlation(&view.inner.id);
        if let Some(a) = actor {
            ctx = ctx.with_actor(a);
        }
        Self::emit_typed(sink, &ctx, &payload).await;
    }

    /// Emits a typed transition event when an ODRL offer is deleted.
    pub async fn offer_deleted(
        sink: &dyn EventSinkTrait,
        owner: Owner,
        id: &Urn,
        actor: Option<EventActor>,
    ) {
        let payload = NegotiationTransitionPayload {
            transition: "offer.deleted".to_string(),
            negotiation_id: id.to_string(),
            agreement_id: None,
            dataset_id: None,
            state: "deleted".to_string(),
            previous_state: None,
            role: "offer".to_string(),
            counter_party_id: None,
            counter_party_address: None,
            protocol_event: Some(id.to_string()),
            error_reason: None,
        };
        let mut ctx = EventContext::for_owner(owner).with_correlation(id.to_string());
        if let Some(a) = actor {
            ctx = ctx.with_actor(a);
        }
        Self::emit_typed(sink, &ctx, &payload).await;
    }

    /// Emits a typed negotiation process lifecycle transition event.
    pub async fn process(
        sink: &dyn EventSinkTrait,
        view: &NegotiationProcessView,
        transition: &str,
        actor: Option<EventActor>,
    ) {
        let dataset_id = view
            .properties
            .get("datasetId")
            .or_else(|| view.properties.get("dataset_id"))
            .or_else(|| view.properties.get("target"))
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let payload = NegotiationTransitionPayload {
            transition: transition.to_string(),
            negotiation_id: view.id.clone(),
            agreement_id: None,
            dataset_id: dataset_id.clone(),
            state: view.state.clone(),
            previous_state: None,
            role: view.role.clone(),
            counter_party_id: Some(view.correlation.peer_participant_id.clone()),
            counter_party_address: view.correlation.callback_address.clone(),
            protocol_event: view.state_attribute.clone(),
            error_reason: view.error_details.as_ref().map(|v| v.to_string()),
        };
        let mut ctx = EventContext::for_owner(view.owner())
            .with_negotiation_id(&view.id)
            .with_correlation(&view.id)
            .with_peer_id(&view.correlation.peer_participant_id);
        if let Some(ds) = dataset_id {
            ctx = ctx.with_dataset_id(ds);
        }
        if let Some(a) = actor {
            ctx = ctx.with_actor(a);
        }
        Self::emit_typed(sink, &ctx, &payload).await;
    }

    /// Emits a typed negotiation process deletion event.
    pub async fn process_deleted(
        sink: &dyn EventSinkTrait,
        owner: Owner,
        id: &Urn,
        actor: Option<EventActor>,
    ) {
        let payload = NegotiationTransitionPayload {
            transition: "process.deleted".to_string(),
            negotiation_id: id.to_string(),
            agreement_id: None,
            dataset_id: None,
            state: "deleted".to_string(),
            previous_state: None,
            role: "process".to_string(),
            counter_party_id: None,
            counter_party_address: None,
            protocol_event: None,
            error_reason: None,
        };
        let mut ctx = EventContext::for_owner(owner)
            .with_negotiation_id(id.to_string())
            .with_correlation(id.to_string());
        if let Some(a) = actor {
            ctx = ctx.with_actor(a);
        }
        Self::emit_typed(sink, &ctx, &payload).await;
    }

    /// Emits a typed agreement lifecycle event with a canonical digest of the ODRL content.
    pub async fn agreement(
        sink: &dyn EventSinkTrait,
        view: &AgreementView,
        action: &str,
        actor: Option<EventActor>,
    ) {
        let policy_id = view
            .inner
            .agreement_content
            .get("@id")
            .or_else(|| view.inner.agreement_content.get("id"))
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let payload = AgreementLifecyclePayload {
            action: action.to_string(),
            agreement_id: view.inner.id.clone(),
            negotiation_id: Some(view.inner.negotiation_agent_process_id.clone()),
            dataset_id: Some(view.inner.target.clone()),
            provider_id: Some(view.inner.provider_participant_id.clone()),
            consumer_id: Some(view.inner.consumer_participant_id.clone()),
            policy_id,
            policy_digest: Some(
                EventIntegrity::for_payload(&view.inner.agreement_content).payload_digest,
            ),
            state: view.inner.state.clone(),
        };
        let mut ctx = EventContext::for_owner(view.inner.owner())
            .with_agreement_id(&view.inner.id)
            .with_negotiation_id(&view.inner.negotiation_agent_process_id)
            .with_dataset_id(&view.inner.target)
            .with_correlation(&view.inner.negotiation_agent_process_id);
        if let Some(a) = actor {
            ctx = ctx.with_actor(a);
        }
        Self::emit_typed(sink, &ctx, &payload).await;
    }

    /// Emits a typed agreement deletion event.
    pub async fn agreement_deleted(
        sink: &dyn EventSinkTrait,
        owner: Owner,
        id: &Urn,
        actor: Option<EventActor>,
    ) {
        let payload = AgreementLifecyclePayload {
            action: "deleted".to_string(),
            agreement_id: id.to_string(),
            negotiation_id: None,
            dataset_id: None,
            provider_id: None,
            consumer_id: None,
            policy_id: None,
            policy_digest: None,
            state: "deleted".to_string(),
        };
        let mut ctx = EventContext::for_owner(owner)
            .with_agreement_id(id.to_string())
            .with_correlation(id.to_string());
        if let Some(a) = actor {
            ctx = ctx.with_actor(a);
        }
        Self::emit_typed(sink, &ctx, &payload).await;
    }

    /// Emits a typed protocol message telemetry event for DSP negotiation exchanges.
    pub async fn message(
        sink: &dyn EventSinkTrait,
        view: &NegotiationMessageView,
        actor: Option<EventActor>,
    ) {
        let direction = if view.direction.eq_ignore_ascii_case("INCOMING")
            || view.direction.eq_ignore_ascii_case("INBOUND")
        {
            MessageDirection::Incoming
        } else {
            MessageDirection::Outgoing
        };
        let payload = ProtocolMessagePayload {
            direction,
            protocol: view.protocol.clone(),
            message_type: view.message_type.clone(),
            process_kind: "negotiation".to_string(),
            process_id: view.negotiation_process_id.clone(),
            peer_id: None,
            peer_address: None,
            status: format!(
                "{} -> {}",
                view.state_transition_from, view.state_transition_to
            ),
            error_reason: None,
        };
        let mut ctx = EventContext::for_owner(view.owner())
            .with_negotiation_id(&view.negotiation_process_id)
            .with_correlation(&view.negotiation_process_id);
        if let Some(a) = actor {
            ctx = ctx.with_actor(a);
        }
        Self::emit_typed(sink, &ctx, &payload).await;
    }

    /// Emits a typed protocol deletion event when a stored negotiation message is removed.
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
            process_kind: "negotiation".to_string(),
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

    async fn emit_typed<P: EventPayload>(
        sink: &dyn EventSinkTrait,
        ctx: &EventContext,
        payload: &P,
    ) {
        if let Err(err) = sink.emit(ctx, payload).await {
            warn!(topic = %payload.topic(), error = %err, "negotiation event emit failed");
        }
    }
}
