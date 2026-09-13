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

//! Integration tests for oauth domain filters.

use chrono::{Duration, Utc};
use common::query::{QueryFilter, QuerySpec, Sort};
use oauth::entities::filters::UserFilter;
use oauth::entities::role::RbacRole;

#[test]
fn user_filter_empty_and_populated() {
    let empty = UserFilter::default();
    assert!(empty.is_empty());
    assert!(empty.validate().is_ok());

    let now = Utc::now();
    let populated = UserFilter {
        tenant_id: Some("tenant-1".to_string()),
        role: Some(RbacRole::Admin),
        email: Some("admin@example.com".to_string()),
        created_after: Some(now),
        created_before: Some(now + Duration::days(1)),
    };
    assert!(!populated.is_empty());
    assert!(populated.validate().is_ok());

    let invalid = UserFilter {
        tenant_id: None,
        role: None,
        email: None,
        created_after: Some(now + Duration::days(1)),
        created_before: Some(now),
    };
    assert!(invalid.validate().is_err());
}

#[test]
fn user_query_deserialization() {
    let json = serde_json::json!({
        "tenantId": "tenant-test",
        "email": "user@example.com",
        "limit": 15,
        "sort": "created_at_desc"
    });
    let spec: QuerySpec<UserFilter> = serde_json::from_value(json).unwrap();
    assert_eq!(spec.filter.tenant_id.as_deref(), Some("tenant-test"));
    assert_eq!(spec.filter.email.as_deref(), Some("user@example.com"));
    assert_eq!(spec.page.limit, 15);
    assert_eq!(spec.sort, Sort::CreatedAtDesc);
    assert!(!spec.is_empty());
}
