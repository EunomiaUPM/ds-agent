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

//! Integration tests for connector domain filters.

use chrono::{Duration, Utc};
use common::query::{QueryFilter, QuerySpec, Sort};
use connector::entities::filters::{ConnectorInstanceFilter, ConnectorTemplateFilter};
use std::str::FromStr;
use urn::Urn;

#[test]
fn connector_template_filter_empty_and_populated() {
    let empty = ConnectorTemplateFilter::default();
    assert!(empty.is_empty());
    assert!(empty.validate().is_ok());

    let populated = ConnectorTemplateFilter {
        tenant_id: None,
        name: Some("http-pull".to_string()),
        author: Some("UPM".to_string()),
        version: Some("1.0.0".to_string()),
        created_after: None,
        created_before: None,
    };
    assert!(!populated.is_empty());
    assert!(populated.validate().is_ok());
}

#[test]
fn connector_template_filter_date_range_validation() {
    let now = Utc::now();
    let valid = ConnectorTemplateFilter {
        tenant_id: None,
        name: None,
        author: None,
        version: None,
        created_after: Some(now),
        created_before: Some(now + Duration::hours(1)),
    };
    assert!(valid.validate().is_ok());

    let invalid = ConnectorTemplateFilter {
        tenant_id: None,
        name: None,
        author: None,
        version: None,
        created_after: Some(now + Duration::hours(1)),
        created_before: Some(now),
    };
    assert!(invalid.validate().is_err());
}

#[test]
fn connector_instance_filter_empty_and_populated() {
    let empty = ConnectorInstanceFilter::default();
    assert!(empty.is_empty());
    assert!(empty.validate().is_ok());

    let urn = Urn::from_str("urn:uuid:f47ac10b-58cc-4372-a567-0e02b2c3d479").unwrap();
    let populated = ConnectorInstanceFilter {
        tenant_id: None,
        distribution_id: Some(urn),
        template_name: Some("http-pull".to_string()),
        template_version: Some("1.0.0".to_string()),
        author: Some("UPM".to_string()),
        owner_id: Some("tenant-1".to_string()),
        created_after: None,
        created_before: None,
    };
    assert!(!populated.is_empty());
    assert!(populated.validate().is_ok());
}

#[test]
fn connector_template_query_deserialization() {
    let json = serde_json::json!({
        "name": "http-source",
        "author": "tester",
        "limit": 50,
        "sort": "created_at_desc"
    });
    let spec: QuerySpec<ConnectorTemplateFilter> = serde_json::from_value(json).unwrap();
    assert_eq!(spec.filter.name.as_deref(), Some("http-source"));
    assert_eq!(spec.filter.author.as_deref(), Some("tester"));
    assert_eq!(spec.page.limit, 50);
    assert_eq!(spec.sort, Sort::CreatedAtDesc);
    assert!(spec.sort.is_descending());
}
