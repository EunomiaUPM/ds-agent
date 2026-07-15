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

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::entities::role::RbacRole;

// Pagination ────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct Page {
    #[serde(default = "default_limit")]
    pub limit: u32,
    pub cursor: Option<String>,
}

fn default_limit() -> u32 {
    20
}

impl Default for Page {
    fn default() -> Self {
        Self {
            limit: default_limit(),
            cursor: None,
        }
    }
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

// Filters ───────────────────────────────────────────────────────────────────

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UserFilter {
    pub role: Option<RbacRole>,
    pub email: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}
