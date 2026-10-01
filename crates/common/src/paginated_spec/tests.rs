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

//! Page limits, cursor encoding and the Paginated window, without a database.

use chrono::Utc;

use crate::paginated_spec::{Cursor, Page, Paginated, Sort, DEFAULT_PAGE_LIMIT, MAX_PAGE_LIMIT};

/// A page defaults to the default limit; clamping caps it at the max and lifts zero to one.
#[test]
fn page_defaults_and_clamps_its_limit() {
    let default_page = Page::default();
    assert_eq!(default_page.limit, DEFAULT_PAGE_LIMIT);
    assert_eq!(default_page.cursor, None);

    let high_page = Page::new(500, None);
    assert_eq!(high_page.clamped().limit, MAX_PAGE_LIMIT);

    let zero_page = Page::new(0, None);
    assert!(zero_page.validate().is_err());
    assert_eq!(zero_page.clamped().limit, 1);
}

/// A timestamp cursor decodes to the same second.
#[test]
fn timestamp_cursor_round_trips() {
    let now = Utc::now();
    let cursor_str = Cursor::encode_timestamp(&now);
    assert!(!cursor_str.is_empty());

    let decoded = Cursor::decode_utc_timestamp(&cursor_str).expect("cursor should decode");
    assert_eq!(now.timestamp(), decoded.timestamp());
}

/// Bad base64, or base64 that is not a date, is rejected.
#[test]
fn malformed_cursor_is_rejected() {
    assert!(Cursor::decode_timestamp("not-base64!").is_err());
    assert!(Cursor::decode_timestamp("bm90LWEtZGF0ZQ==").is_err());
}

/// A window with more items than the limit is cut and points at the last item kept.
#[test]
fn window_over_limit_yields_next_cursor() {
    let items = vec![1, 2, 3, 4, 5];
    let paginated = Paginated::from_window(items, 4, |x| format!("cursor_{x}"));

    assert_eq!(paginated.items.len(), 4);
    assert_eq!(paginated.next_cursor, Some("cursor_4".to_string()));

    let short_items = vec![10, 20];
    let short_paginated = Paginated::from_window(short_items, 5, |x| format!("cursor_{x}"));
    assert_eq!(short_paginated.items.len(), 2);
    assert_eq!(short_paginated.next_cursor, None);
}

/// The default sort is newest first.
#[test]
fn default_sort_is_created_at_desc() {
    let sort = Sort::default();
    assert_eq!(sort, Sort::CreatedAtDesc);
    assert!(!sort.is_ascending());
    assert!(Sort::CreatedAtAsc.is_ascending());
}

/// A composite cursor decodes to the same timestamp and id.
#[test]
fn composite_cursor_round_trips_with_id() {
    let now = Utc::now();
    let id = "urn:item:uuid-456";
    let cursor_str = Cursor::encode_composite(&now, id);
    assert!(!cursor_str.is_empty());

    let decoded = Cursor::decode(&cursor_str).expect("composite cursor should decode");
    assert_eq!(now.timestamp(), decoded.timestamp.timestamp());
    assert_eq!(decoded.id.as_deref(), Some(id));
}

/// A cursor built from a non-UTC time decodes to the same instant.
#[test]
fn cursor_keeps_the_original_offset() {
    let fixed = chrono::DateTime::parse_from_rfc3339("2026-09-13T23:30:00+02:00").unwrap();
    let cursor_str = Cursor::encode_timestamp(&fixed);
    assert!(!cursor_str.is_empty());

    let decoded = Cursor::decode(&cursor_str).unwrap();
    assert_eq!(decoded.timestamp, fixed);
    assert_eq!(decoded.id, None);
}

/// `from_page` sets a next cursor only when the page came back full, and keeps the total.
#[test]
fn from_page_sets_cursor_only_on_full_pages() {
    let page = Page::new(3, None);
    let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let paginated = Paginated::from_page(items.clone(), &page, Some(10), |s| format!("cursor_{s}"));

    assert_eq!(paginated.items.len(), 3);
    assert_eq!(paginated.next_cursor, Some("cursor_c".to_string()));
    assert_eq!(paginated.total, Some(10));

    let short_items = vec!["x".to_string()];
    let short_paginated =
        Paginated::from_page(short_items, &page, Some(1), |s| format!("cursor_{s}"));
    assert_eq!(short_paginated.items.len(), 1);
    assert_eq!(short_paginated.next_cursor, None);
    assert_eq!(short_paginated.total, Some(1));
}

/// A sorted cursor encodes the column the list is sorted by, plus the id when given.
#[test]
fn sorted_cursor_encodes_the_sort_column() {
    let created = chrono::DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z").unwrap();
    let updated = chrono::DateTime::parse_from_rfc3339("2026-02-01T00:00:00Z").unwrap();

    let cur_created = Cursor::encode_sorted(&created, &updated, &Sort::CreatedAtDesc);
    let dec_created = Cursor::decode(&cur_created).unwrap();
    assert_eq!(dec_created.timestamp, created);

    let cur_updated = Cursor::encode_sorted(&created, &updated, &Sort::UpdatedAtDesc);
    let dec_updated = Cursor::decode(&cur_updated).unwrap();
    assert_eq!(dec_updated.timestamp, updated);

    let cur_with_id =
        Cursor::encode_sorted_with_id(&created, &updated, &Sort::UpdatedAtAsc, Some("id-123"));
    let dec_with_id = Cursor::decode(&cur_with_id).unwrap();
    assert_eq!(dec_with_id.timestamp, updated);
    assert_eq!(dec_with_id.id.as_deref(), Some("id-123"));
}
