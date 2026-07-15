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

// Pagination ────────────────────────────────────────────────────────────────

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ymir::errors::{BadFormat, Errors, Outcome};

/// Default page size when the client does not specify `limit`.
pub const DEFAULT_PAGE_LIMIT: u32 = 20;

/// Hard upper bound on `limit` to protect the database from unbounded scans.
/// Requests above this are clamped down to it.
pub const MAX_PAGE_LIMIT: u32 = 100;

/// Maximum number of ids accepted in a single batch lookup.
pub const MAX_BATCH_IDS: usize = 100;

/// Pagination in query requests
#[derive(Deserialize)]
pub struct Page {
    #[serde(default = "default_limit")]
    pub limit: u32,
    pub cursor: Option<String>,
}

pub fn default_limit() -> u32 {
    DEFAULT_PAGE_LIMIT
}

/// Clamps a client-supplied `limit` into `[1, MAX_PAGE_LIMIT]`.
pub fn clamp_page_limit(limit: u32) -> u32 {
    limit.clamp(1, MAX_PAGE_LIMIT)
}

/// Validate range in dates. No "to date" can be before "from date"
#[allow(clippy::result_large_err)]
pub fn validate_date_range(
    after: Option<DateTime<Utc>>,
    before: Option<DateTime<Utc>>,
) -> Outcome<()> {
    if let (Some(a), Some(b)) = (after, before) {
        if a >= b {
            return Err(Errors::format(
                BadFormat::Received,
                "createdAfter must be strictly before createdBefore",
                None,
            ));
        }
    }
    Ok(())
}

/// Pagination response with items, total count and base64 time-aware cursor
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
    pub total: Option<u64>,
}

/// Sorting system
#[derive(Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Sort {
    CreatedAtAsc,
    #[default]
    CreatedAtDesc,
    UpdatedAtDesc,
}
