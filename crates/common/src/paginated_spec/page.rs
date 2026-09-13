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

//! Pagination request configuration and bounds validation.

use serde::{Deserialize, Serialize};
use ymir::errors::{BadFormat, Errors, Outcome};

/// Default page size when the client does not specify limit.
pub const DEFAULT_PAGE_LIMIT: u32 = 20;

/// Hard upper bound on limit to protect data stores from unbounded scans.
pub const MAX_PAGE_LIMIT: u32 = 100;

/// Maximum number of ids accepted in a single batch lookup.
pub const MAX_BATCH_IDS: usize = 100;

/// Pagination parameters specifying batch size and cursor position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Page {
    #[serde(default = "Page::default_limit")]
    pub limit: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

impl Default for Page {
    fn default() -> Self {
        Self {
            limit: Self::default_limit(),
            cursor: None,
        }
    }
}

impl Page {
    /// Creates a new pagination request specification.
    pub fn new(limit: u32, cursor: Option<String>) -> Self {
        Self { limit, cursor }
    }

    /// Provides default limit for deserialization.
    pub fn default_limit() -> u32 {
        DEFAULT_PAGE_LIMIT
    }

    /// Clamps an arbitrary limit into the allowed interval.
    pub fn clamp_limit(limit: u32) -> u32 {
        limit.clamp(1, MAX_PAGE_LIMIT)
    }

    /// Returns a copy with clamped limit bounds.
    pub fn clamped(&self) -> Self {
        Self {
            limit: Self::clamp_limit(self.limit),
            cursor: self.cursor.clone(),
        }
    }

    /// Validates pagination bounds.
    pub fn validate(&self) -> Outcome<()> {
        if self.limit == 0 {
            return Err(Errors::format(
                BadFormat::Received,
                "page limit must be strictly positive",
                None,
            ));
        }
        Ok(())
    }

    /// Returns lookahead limit count for windowing queries.
    pub fn fetch_limit(&self) -> u64 {
        (Self::clamp_limit(self.limit) as u64) + 1
    }
}

/// Backwards compatible alias for default limit.
pub fn default_limit() -> u32 {
    Page::default_limit()
}

/// Backwards compatible alias for clamping page limit.
pub fn clamp_page_limit(limit: u32) -> u32 {
    Page::clamp_limit(limit)
}

/// Legacy / offset-based pagination parameters with limit and page index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PaginationParams {
    pub limit: Option<u64>,
    pub page: Option<u64>,
}

impl PaginationParams {
    /// Creates a new page-indexed pagination parameter specification.
    pub fn new(limit: Option<u64>, page: Option<u64>) -> Self {
        Self { limit, page }
    }

    /// Converts to standard cursor Page with clamped limit.
    pub fn to_page(&self) -> Page {
        let limit = self
            .limit
            .map(|l| Page::clamp_limit(l as u32))
            .unwrap_or(DEFAULT_PAGE_LIMIT);
        Page::new(limit, None)
    }
}
