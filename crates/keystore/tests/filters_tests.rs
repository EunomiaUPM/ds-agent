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

//! Integration tests for keystore domain filters.

use common::query::{QueryFilter, QuerySpec, Sort};
use keystore::entities::filters::PrefixFilter;

#[test]
fn prefix_filter_empty_and_populated() {
    let empty = PrefixFilter::default();
    assert!(empty.is_empty());
    assert!(empty.validate().is_ok());

    let populated = PrefixFilter {
        prefix: Some("vault/secrets/".to_string()),
    };
    assert!(!populated.is_empty());
    assert!(populated.validate().is_ok());
}

#[test]
fn prefix_query_deserialization() {
    let json = serde_json::json!({
        "prefix": "app/config/",
        "limit": 50,
        "sort": "created_at_desc"
    });
    let spec: QuerySpec<PrefixFilter> = serde_json::from_value(json).unwrap();
    assert_eq!(spec.filter.prefix.as_deref(), Some("app/config/"));
    assert_eq!(spec.page.limit, 50);
    assert_eq!(spec.sort, Sort::CreatedAtDesc);
    assert!(!spec.is_empty());
}
