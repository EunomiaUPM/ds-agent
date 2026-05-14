use chrono::{DateTime, Utc};
use urn::Urn;
use crate::entities::ids::{ParticipantId, TenantId};
use crate::entities::message_envelope::Direction;
use crate::entities::protocol::{ProtocolId, ProtocolState, TransferRole};

// ── Pagination ────────────────────────────────────────────────────────────────

pub struct Page {
    pub limit: u32,
    pub cursor: Option<String>,
}

pub struct Paginated<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
    pub total: Option<u64>,
}

pub enum Sort {
    CreatedAtAsc,
    CreatedAtDesc,
    UpdatedAtDesc,
}

// ── Filters ───────────────────────────────────────────────────────────────────

pub struct TransferProcessFilter {
    pub tenant_id: TenantId,
    pub protocol: Option<ProtocolId>,
    pub state: Option<ProtocolState>,
    pub role: Option<TransferRole>,
    pub agreement_id: Option<Urn>,
    pub peer_participant_id: Option<ParticipantId>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

pub struct TransferMessageFilter {
    pub tenant_id: TenantId,
    pub direction: Option<Direction>,
    pub protocol: Option<ProtocolId>,
    pub state_transition_to: Option<ProtocolState>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}