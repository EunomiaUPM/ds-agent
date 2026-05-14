use std::collections::HashMap;
use chrono::{DateTime, Duration, Utc};
use serde_json::Value as Json;
use crate::entities::commands::EditTransferProcessCommand;
use crate::entities::ids::{TenantId, TransferProcessId};
use crate::entities::message_envelope::MessageEnvelopeRef;
use crate::entities::protocol::{
    ProtocolId, ProtocolState, StateMetadata, TransferCorrelation, TransferRole,
};

pub(crate) struct TransferProcess {
    // Common
    transfer_id: TransferProcessId,
    tenant_id: TenantId,
    role: TransferRole,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    version: u32,

    // Protocol
    protocol: ProtocolId,
    protocol_state: ProtocolState,
    state_metadata: StateMetadata,
    correlation: TransferCorrelation,

    properties: Json,
    error_details: Option<Json>,

    last_inbound_envelope: Option<MessageEnvelopeRef>,
    last_outbound_envelope: Option<MessageEnvelopeRef>,
}

impl TransferProcess {
    pub fn builder() -> TransferProcessBuilder {
        TransferProcessBuilder::default()
    }

    pub fn new(
        tenant_id: TenantId,
        role: TransferRole,
        protocol: ProtocolId,
        protocol_state: ProtocolState,
        correlation: TransferCorrelation,
    ) -> Self {
        let now = Utc::now();
        Self {
            transfer_id: TransferProcessId::generate(),
            tenant_id,
            role,
            created_at: now,
            updated_at: now,
            version: 0,
            protocol,
            protocol_state,
            state_metadata: StateMetadata::empty(),
            correlation,
            properties: serde_json::json!({}),
            error_details: None,
            last_inbound_envelope: None,
            last_outbound_envelope: None,
        }
    }

    pub(crate) fn rehydrate(
        transfer_id: TransferProcessId,
        tenant_id: TenantId,
        role: TransferRole,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        version: u32,
        protocol: ProtocolId,
        protocol_state: ProtocolState,
        state_metadata: StateMetadata,
        correlation: TransferCorrelation,
        properties: Json,
        error_details: Option<Json>,
        last_inbound_envelope: Option<MessageEnvelopeRef>,
        last_outbound_envelope: Option<MessageEnvelopeRef>,
    ) -> Self {
        Self {
            transfer_id,
            tenant_id,
            role,
            created_at,
            updated_at,
            version,
            protocol,
            protocol_state,
            state_metadata,
            correlation,
            properties,
            error_details,
            last_inbound_envelope,
            last_outbound_envelope,
        }
    }

    // ── Mutators ──────────────────────────────────────────────────────────────

    pub fn apply_edit(&mut self, cmd: EditTransferProcessCommand) {
        if let Some(state) = cmd.state {
            self.protocol_state = state;
            self.state_metadata = cmd.state_metadata.unwrap_or(StateMetadata::empty());
        }
        if let Some(ids) = cmd.identifiers {
            self.correlation.identifiers.extend(ids);
        }
        if let Some(props) = cmd.properties {
            json_merge(&mut self.properties, props);
        }
        if let Some(err) = cmd.error_details {
            self.error_details = Some(err);
        }
        self.bump();
    }

    pub fn record_inbound_envelope(&mut self, env_ref: MessageEnvelopeRef) {
        self.last_inbound_envelope = Some(env_ref);
        self.updated_at = Utc::now();
    }

    pub fn record_outbound_envelope(&mut self, env_ref: MessageEnvelopeRef) {
        self.last_outbound_envelope = Some(env_ref);
        self.updated_at = Utc::now();
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    pub fn id(&self) -> &TransferProcessId { &self.transfer_id }
    pub fn tenant_id(&self) -> &TenantId { &self.tenant_id }
    pub fn protocol(&self) -> &ProtocolId { &self.protocol }
    pub fn state(&self) -> &ProtocolState { &self.protocol_state }
    pub fn role(&self) -> TransferRole { self.role }
    pub fn version(&self) -> u64 { self.version as u64 }
    pub fn correlation(&self) -> &TransferCorrelation { &self.correlation }
    pub fn properties(&self) -> &Json { &self.properties }
    pub fn error_details(&self) -> Option<&Json> { self.error_details.as_ref() }
    pub fn last_inbound_envelope(&self) -> Option<&MessageEnvelopeRef> { self.last_inbound_envelope.as_ref() }
    pub fn last_outbound_envelope(&self) -> Option<&MessageEnvelopeRef> { self.last_outbound_envelope.as_ref() }
    pub fn created_at(&self) -> DateTime<Utc> { self.created_at }
    pub fn updated_at(&self) -> DateTime<Utc> { self.updated_at }

    // ── Predicates ────────────────────────────────────────────────────────────

    pub fn belongs_to(&self, tenant: &TenantId) -> bool { &self.tenant_id == tenant }
    pub fn uses_protocol(&self, protocol: &ProtocolId) -> bool { &self.protocol == protocol }
    pub fn age(&self) -> Duration { Utc::now() - self.created_at }

    fn bump(&mut self) {
        self.version = self.version.saturating_add(1);
        self.updated_at = Utc::now();
    }
}

fn json_merge(base: &mut Json, patch: Json) {
    if let (Json::Object(base_map), Json::Object(patch_map)) = (base, patch) {
        for (k, v) in patch_map {
            base_map.insert(k, v);
        }
    }
}

// ── Builder ───────────────────────────────────────────────────────────────────

#[derive(Clone, Default)]
pub(crate) struct TransferProcessBuilder {
    tenant_id: Option<TenantId>,
    role: Option<TransferRole>,
    protocol: Option<ProtocolId>,
    protocol_state: Option<ProtocolState>,
    correlation: Option<TransferCorrelation>,
}

impl TransferProcessBuilder {
    pub fn with_tenant(mut self, tenant_id: TenantId) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    pub fn with_role(mut self, role: TransferRole) -> Self {
        self.role = Some(role);
        self
    }

    pub fn with_protocol(mut self, protocol: ProtocolId) -> Self {
        self.protocol = Some(protocol);
        self
    }

    pub fn with_state(mut self, state: ProtocolState) -> Self {
        self.protocol_state = Some(state);
        self
    }

    pub fn with_correlation(mut self, correlation: TransferCorrelation) -> Self {
        self.correlation = Some(correlation);
        self
    }

    pub fn build(self) -> TransferProcess {
        TransferProcess::new(
            self.tenant_id.expect("tenant_id is required"),
            self.role.expect("role is required"),
            self.protocol.expect("protocol is required"),
            self.protocol_state.expect("protocol_state is required"),
            self.correlation.unwrap_or(TransferCorrelation {
                identifiers: HashMap::new(),
                consumer_pid: None,
                provider_pid: None,
                agreement_id: None,
                callback_address: None,
                peer_participant_id: None,
            }),
        )
    }
}