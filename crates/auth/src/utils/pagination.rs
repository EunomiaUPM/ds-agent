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

//! In-memory pagination utility for ssi-auth entities.

use chrono::{DateTime, Utc};
use common::paginated_spec::{Cursor, Page, Paginated, Sort};

/// Utility for cursor-based pagination over in-memory entity vectors.
pub struct AuthPagination;

impl AuthPagination {
    /// Paginates an in-memory slice using cursor decoding and composite keyset.
    pub fn paginate<T: Clone>(
        items: &[T],
        page: &Page,
        sort: &Sort,
        cursor_extractor: impl Fn(&T, &Sort) -> (DateTime<Utc>, String),
    ) -> Paginated<T> {
        let total = items.len() as u64;
        let mut start_idx = 0;

        if let Some(cursor_str) = &page.cursor {
            if let Ok(decoded) = Cursor::decode(cursor_str) {
                let cursor_utc = decoded.timestamp_utc();
                if let Some(pos) = items.iter().position(|item| {
                    let (item_ts, item_id) = cursor_extractor(item, sort);
                    if let Some(id) = &decoded.id {
                        item_ts == cursor_utc && &item_id == id
                    } else {
                        item_ts == cursor_utc
                    }
                }) {
                    start_idx = pos + 1;
                }
            }
        }

        let end_idx = (start_idx + page.limit as usize).min(items.len());
        let paged_items: Vec<T> = if start_idx < items.len() {
            items[start_idx..end_idx].to_vec()
        } else {
            Vec::new()
        };

        let next_cursor = if paged_items.len() == page.limit as usize && end_idx < items.len() {
            paged_items.last().map(|last| {
                let (ts, id) = cursor_extractor(last, sort);
                Cursor::encode_composite(&ts, &id)
            })
        } else {
            None
        };

        Paginated::new(paged_items, next_cursor, Some(total))
    }
}
