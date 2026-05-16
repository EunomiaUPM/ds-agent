use crate::entities::ids::{ParticipantId, TenantId};
use crate::entities::message_envelope::Direction;
use crate::entities::protocol::{ProtocolId, ProtocolState, TransferRole};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use urn::Urn;

// Pagination ────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct Page {
    #[serde(default = "default_limit")]
    pub limit: u32,
    pub cursor: Option<String>,
}

fn default_limit() -> u32 {
    20
}

#[derive(Serialize)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
    pub total: Option<u64>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Sort {
    CreatedAtAsc,
    #[default]
    CreatedAtDesc,
    UpdatedAtDesc,
}

// Filters ───────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct TransferProcessFilter {
    #[serde(skip_deserializing, default)]
    pub tenant_id: TenantId,
    pub protocol: Option<ProtocolId>,
    pub state: Option<ProtocolState>,
    pub role: Option<TransferRole>,
    pub agreement_id: Option<Urn>,
    pub peer_participant_id: Option<ParticipantId>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct TransferMessageFilter {
    #[serde(skip_deserializing, default)]
    pub tenant_id: TenantId,
    pub direction: Option<Direction>,
    pub protocol: Option<ProtocolId>,
    pub state_transition_to: Option<ProtocolState>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}
