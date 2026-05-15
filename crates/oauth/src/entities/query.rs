use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::entities::role::Role;

// ── Pagination ────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct Page {
    #[serde(default = "default_limit")]
    pub limit: u32,
    pub cursor: Option<String>,
}

fn default_limit() -> u32 { 20 }

impl Default for Page {
    fn default() -> Self { Self { limit: default_limit(), cursor: None } }
}

#[derive(Serialize)]
pub(crate) struct Paginated<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
    pub total: Option<u64>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Sort {
    CreatedAtAsc,
    #[default]
    CreatedAtDesc,
}

// ── Filters ───────────────────────────────────────────────────────────────────

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UserFilter {
    pub role: Option<Role>,
    pub email: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}
