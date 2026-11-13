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

//! Canonical event adapters translating dataplane transitions and data accesses into typed envelopes.

use events::schemas::dataspace::{
    DataAccessPayload, DataplaneStatePayload, ObligationExecutionPayload, PolicyDecisionOutcome,
    PolicyDecisionPayload,
};
use events::{EventActor, EventContext, EventEnvelope, EventPayload, EventSinkTrait, EventSource};
use serde_json::Value;
use tracing::warn;
use ymir::oauth::OwnedTrait;

use crate::data::sea_orm::orm::dataplane_transfers as dataplane_transfers_model;
use crate::entities::dataplane_transfers::DataplaneTransferDto;
use crate::relay::consumer::KafkaRecord;
use crate::relay::worker::KafkaRelayTaskSpec;

/// Stateless adapter emitting `dataplane` state transitions and data-access telemetry events.
pub struct DataplaneEvents;

impl DataplaneEvents {
    /// Emits a durable state transition event for a dataplane transfer process.
    pub async fn state_changed(
        sink: &dyn EventSinkTrait,
        model: &dataplane_transfers_model::Model,
        previous_state: Option<String>,
        trigger: &str,
        reason: Option<&str>,
        error_reason: Option<String>,
        actor: Option<EventActor>,
    ) {
        let message = match reason {
            Some(r) => format!("dataplane={} trigger={trigger} reason={r}", model.id),
            None => format!("dataplane={} trigger={trigger}", model.id),
        };
        let payload = DataplaneStatePayload {
            transfer_id: model.transfer_process_id.clone(),
            agreement_id: None,
            dataset_id: None,
            state: model.state.to_string(),
            previous_state,
            transfer_type: Self::egress_kind(&model.egress_config),
            direction: model.interaction_mode.to_string(),
            message: Some(message),
            error_reason,
        };
        let mut ctx = EventContext::for_owner(model.owner())
            .with_transfer_id(&model.transfer_process_id)
            .with_correlation(&model.transfer_process_id);
        if let Some(a) = actor {
            ctx = ctx.with_actor(a);
        }
        Self::emit_typed(sink, &ctx, &payload).await;
    }

    /// Builds a canonical HTTP data-access telemetry envelope without emitting it.
    #[allow(clippy::too_many_arguments)]
    pub fn build_http_data_access_envelope(
        dataplane: &DataplaneTransferDto,
        method: &str,
        path: Option<&str>,
        status_code: Option<u16>,
        duration_ms: Option<u64>,
        allowed: bool,
        denial_reason: Option<String>,
    ) -> Option<EventEnvelope> {
        let (agreement_id, dataset_id, peer_id, ctx) = Self::dataplane_context(dataplane);
        let payload = DataAccessPayload {
            allowed,
            transfer_id: dataplane.inner.transfer_process_id.clone(),
            agreement_id,
            dataset_id,
            channel: "http_proxy".to_string(),
            method: Some(method.to_string()),
            path: path.map(str::to_string),
            status_code,
            bytes_transferred: None,
            duration_ms,
            peer_id,
            denial_reason,
        };
        Self::build_envelope(&ctx, &payload)
    }

    /// Emits a data-access telemetry event for an HTTP data proxy request (allowed or denied).
    #[allow(clippy::too_many_arguments)]
    pub async fn http_data_access(
        sink: &dyn EventSinkTrait,
        dataplane: &DataplaneTransferDto,
        method: &str,
        path: Option<&str>,
        status_code: Option<u16>,
        duration_ms: Option<u64>,
        allowed: bool,
        denial_reason: Option<String>,
    ) {
        if let Some(envelope) = Self::build_http_data_access_envelope(
            dataplane,
            method,
            path,
            status_code,
            duration_ms,
            allowed,
            denial_reason,
        ) {
            Self::emit_envelope(sink, envelope).await;
        }
    }

    /// Builds a policy enforcement decision envelope without emitting it.
    pub fn build_policy_decision_envelope(
        dataplane: &DataplaneTransferDto,
        decision: PolicyDecisionOutcome,
        action: &str,
        policy_id: Option<&str>,
        obligations_triggered: &[&str],
        reason: Option<String>,
    ) -> Option<EventEnvelope> {
        let (agreement_id, dataset_id, peer_id, ctx) = Self::dataplane_context(dataplane);
        let payload = PolicyDecisionPayload {
            decision,
            enforcement_point: "dataplane_http".to_string(),
            action: action.to_string(),
            agreement_id,
            transfer_id: Some(dataplane.inner.transfer_process_id.clone()),
            dataset_id,
            policy_id: policy_id.map(str::to_string),
            policy_digest: None,
            peer_id,
            obligations_triggered: obligations_triggered
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            reason,
        };
        Self::build_envelope(&ctx, &payload)
    }

    /// Emits a policy enforcement decision event (`Permit` or `Deny`) for the HTTP PEP.
    pub async fn policy_decision(
        sink: &dyn EventSinkTrait,
        dataplane: &DataplaneTransferDto,
        decision: PolicyDecisionOutcome,
        action: &str,
        policy_id: Option<&str>,
        obligations_triggered: &[&str],
        reason: Option<String>,
    ) {
        if let Some(envelope) = Self::build_policy_decision_envelope(
            dataplane,
            decision,
            action,
            policy_id,
            obligations_triggered,
            reason,
        ) {
            Self::emit_envelope(sink, envelope).await;
        }
    }

    /// Builds an obligation execution audit envelope without emitting it.
    pub fn build_obligation_executed_envelope(
        dataplane: &DataplaneTransferDto,
        obligation_id: &str,
        obligation_kind: &str,
        succeeded: bool,
        duration_ms: Option<u64>,
        details: Option<String>,
        error_reason: Option<String>,
    ) -> Option<EventEnvelope> {
        let (agreement_id, _, _, ctx) = Self::dataplane_context(dataplane);
        let payload = ObligationExecutionPayload {
            obligation_id: obligation_id.to_string(),
            obligation_kind: obligation_kind.to_string(),
            enforcement_point: "dataplane_http".to_string(),
            agreement_id,
            transfer_id: Some(dataplane.inner.transfer_process_id.clone()),
            succeeded,
            duration_ms,
            details,
            error_reason,
        };
        Self::build_envelope(&ctx, &payload)
    }

    /// Emits an obligation execution audit event recorded by the HTTP PEP.
    #[allow(clippy::too_many_arguments)]
    pub async fn obligation_executed(
        sink: &dyn EventSinkTrait,
        dataplane: &DataplaneTransferDto,
        obligation_id: &str,
        obligation_kind: &str,
        succeeded: bool,
        duration_ms: Option<u64>,
        details: Option<String>,
        error_reason: Option<String>,
    ) {
        if let Some(envelope) = Self::build_obligation_executed_envelope(
            dataplane,
            obligation_id,
            obligation_kind,
            succeeded,
            duration_ms,
            details,
            error_reason,
        ) {
            Self::emit_envelope(sink, envelope).await;
        }
    }

    fn dataplane_context(
        dataplane: &DataplaneTransferDto,
    ) -> (
        Option<String>,
        Option<String>,
        Option<String>,
        EventContext,
    ) {
        let agreement_id = dataplane
            .fields
            .get("agreementId")
            .or_else(|| dataplane.fields.get("agreement_id"))
            .cloned();
        let dataset_id = dataplane
            .fields
            .get("datasetId")
            .or_else(|| dataplane.fields.get("dataset_id"))
            .cloned();
        let peer_id = dataplane
            .fields
            .get("peerParticipantId")
            .or_else(|| dataplane.fields.get("peerId"))
            .cloned();
        let mut ctx = EventContext::for_owner(dataplane.inner.owner())
            .with_transfer_id(&dataplane.inner.transfer_process_id)
            .with_correlation(&dataplane.inner.transfer_process_id);
        if let Some(a) = &agreement_id {
            ctx = ctx.with_agreement_id(a);
        }
        if let Some(d) = &dataset_id {
            ctx = ctx.with_dataset_id(d);
        }
        if let Some(p) = &peer_id {
            ctx = ctx.with_peer_id(p);
        }
        (agreement_id, dataset_id, peer_id, ctx)
    }

    /// Emits a data-access telemetry event for a Kafka relay delivery attempt.
    pub async fn kafka_data_access(
        sink: &dyn EventSinkTrait,
        spec: &KafkaRelayTaskSpec,
        record: &KafkaRecord,
        status_code: Option<u16>,
        allowed: bool,
        denial_reason: Option<String>,
    ) {
        let transfer_id = spec.transfer_process_id.to_string();
        let payload = DataAccessPayload {
            allowed,
            transfer_id: transfer_id.clone(),
            agreement_id: None,
            dataset_id: None,
            channel: "kafka_relay".to_string(),
            method: Some("POST".to_string()),
            path: Some(format!(
                "{}:{}@{}",
                record.topic, record.partition, record.offset
            )),
            status_code,
            bytes_transferred: Some(record.payload.len() as u64),
            duration_ms: None,
            peer_id: None,
            denial_reason,
        };
        let ctx = EventContext::for_owner(spec.owner.clone())
            .with_transfer_id(&transfer_id)
            .with_correlation(&transfer_id);
        Self::emit_typed(sink, &ctx, &payload).await;
    }

    fn egress_kind(egress: &Value) -> String {
        egress
            .get("type")
            .or_else(|| egress.get("kind"))
            .and_then(|v| v.as_str())
            .or_else(|| egress.as_str())
            .or_else(|| {
                egress
                    .as_object()
                    .and_then(|map| map.keys().next())
                    .map(String::as_str)
            })
            .unwrap_or("unknown")
            .to_string()
    }

    fn build_envelope<P: EventPayload>(
        ctx: &EventContext,
        payload: &P,
    ) -> Option<EventEnvelope> {
        match payload.into_envelope(ctx, EventSource::for_crate(P::CRATE_NAME)) {
            Ok(env) => Some(env),
            Err(err) => {
                warn!(topic = %payload.topic(), error = %err, "dataplane event build failed");
                None
            }
        }
    }

    /// Emits a pre-built canonical envelope through `sink`, warning on failure.
    pub async fn emit_envelope(sink: &dyn EventSinkTrait, envelope: EventEnvelope) {
        let topic = envelope.topic.clone();
        if let Err(err) = sink.emit_envelope(envelope).await {
            warn!(topic = %topic, error = %err, "dataplane event emit failed");
        }
    }

    async fn emit_typed<P: EventPayload>(
        sink: &dyn EventSinkTrait,
        ctx: &EventContext,
        payload: &P,
    ) {
        if let Some(envelope) = Self::build_envelope(ctx, payload) {
            Self::emit_envelope(sink, envelope).await;
        }
    }
}
