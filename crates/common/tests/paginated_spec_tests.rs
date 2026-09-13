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

use chrono::Utc;
use common::paginated_spec::{Cursor, DEFAULT_PAGE_LIMIT, MAX_PAGE_LIMIT, Page, Paginated, Sort};

#[test]
fn test_page_defaults_and_clamping() {
    let default_page = Page::default();
    assert_eq!(default_page.limit, DEFAULT_PAGE_LIMIT);
    assert_eq!(default_page.cursor, None);

    let high_page = Page::new(500, None);
    assert_eq!(high_page.clamped().limit, MAX_PAGE_LIMIT);

    let zero_page = Page::new(0, None);
    assert!(zero_page.validate().is_err());
    assert_eq!(zero_page.clamped().limit, 1);
}

#[test]
fn test_cursor_timestamp_roundtrip() {
    let now = Utc::now();
    let cursor_str = Cursor::encode_timestamp(&now);
    assert!(!cursor_str.is_empty());

    let decoded = Cursor::decode_utc_timestamp(&cursor_str).expect("cursor should decode");
    assert_eq!(now.timestamp(), decoded.timestamp());
}

#[test]
fn test_cursor_invalid_decoding() {
    assert!(Cursor::decode_timestamp("not-base64!").is_err());
    assert!(Cursor::decode_timestamp("bm90LWEtZGF0ZQ==").is_err());
}

#[test]
fn test_paginated_windowing() {
    let items = vec![1, 2, 3, 4, 5];
    let paginated = Paginated::from_window(items, 4, |x| format!("cursor_{x}"));

    assert_eq!(paginated.items.len(), 4);
    assert_eq!(paginated.next_cursor, Some("cursor_4".to_string()));

    let short_items = vec![10, 20];
    let short_paginated = Paginated::from_window(short_items, 5, |x| format!("cursor_{x}"));
    assert_eq!(short_paginated.items.len(), 2);
    assert_eq!(short_paginated.next_cursor, None);
}

#[test]
fn test_sort_defaults() {
    let sort = Sort::default();
    assert_eq!(sort, Sort::CreatedAtDesc);
    assert!(!sort.is_ascending());
    assert!(Sort::CreatedAtAsc.is_ascending());
}

#[test]
fn test_cursor_composite_roundtrip() {
    let now = Utc::now();
    let id = "urn:item:uuid-456";
    let cursor_str = Cursor::encode_composite(&now, id);
    assert!(!cursor_str.is_empty());

    let decoded = Cursor::decode(&cursor_str).expect("composite cursor should decode");
    assert_eq!(now.timestamp(), decoded.timestamp.timestamp());
    assert_eq!(decoded.id.as_deref(), Some(id));
}

#[test]
fn test_cursor_generic_timezone() {
    let fixed = chrono::DateTime::parse_from_rfc3339("2026-09-13T23:30:00+02:00").unwrap();
    let cursor_str = Cursor::encode_timestamp(&fixed);
    assert!(!cursor_str.is_empty());

    let decoded = Cursor::decode(&cursor_str).unwrap();
    assert_eq!(decoded.timestamp, fixed);
    assert_eq!(decoded.id, None);
}

#[test]
fn test_paginated_from_page() {
    let page = Page::new(3, None);
    let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let paginated = Paginated::from_page(items.clone(), &page, Some(10), |s| format!("cursor_{s}"));

    assert_eq!(paginated.items.len(), 3);
    assert_eq!(paginated.next_cursor, Some("cursor_c".to_string()));
    assert_eq!(paginated.total, Some(10));

    let short_items = vec!["x".to_string()];
    let short_paginated = Paginated::from_page(short_items, &page, Some(1), |s| format!("cursor_{s}"));
    assert_eq!(short_paginated.items.len(), 1);
    assert_eq!(short_paginated.next_cursor, None);
    assert_eq!(short_paginated.total, Some(1));
}
