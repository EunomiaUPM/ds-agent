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

//! Integration tests for catalog-agent domain filters.

use catalog_agent::entities::filters::{
    CatalogFilter, DataServiceFilter, DatasetFilter, DistributionFilter, OdrlPolicyFilter,
    PolicyTemplateFilter,
};
use chrono::{Duration, Utc};
use common::paginated_spec::Sort;
use common::query::{QueryFilter, QuerySpec};

#[test]
fn catalog_filter_empty_and_validation() {
    let empty = CatalogFilter::default();
    assert!(empty.is_empty());
    assert!(empty.validate().is_ok());

    let now = Utc::now();
    let populated = CatalogFilter {
        tenant_id: Some("default".to_string()),
        title: Some("Main Catalog".to_string()),
        creator: Some("UPM".to_string()),
        participant_id: Some("urn:participant:1".to_string()),
        with_main_catalog: Some(true),
        created_after: Some(now),
        created_before: Some(now + Duration::hours(2)),
    };
    assert!(!populated.is_empty());
    assert!(populated.validate().is_ok());

    let invalid = CatalogFilter {
        tenant_id: None,
        title: None,
        creator: None,
        participant_id: None,
        with_main_catalog: None,
        created_after: Some(now + Duration::hours(1)),
        created_before: Some(now),
    };
    assert!(invalid.validate().is_err());
}

#[test]
fn dataset_filter_empty_and_deserialization() {
    let empty = DatasetFilter::default();
    assert!(empty.is_empty());
    assert!(empty.validate().is_ok());

    let json = serde_json::json!({
        "catalog_id": "urn:catalog:test",
        "title": "Weather Data",
        "limit": 10,
        "sort": "created_at_desc"
    });
    let spec: QuerySpec<DatasetFilter> = serde_json::from_value(json).unwrap();
    assert_eq!(spec.filter.catalog_id.as_deref(), Some("urn:catalog:test"));
    assert_eq!(spec.filter.title.as_deref(), Some("Weather Data"));
    assert_eq!(spec.page.limit, 10);
    assert_eq!(spec.sort, Sort::CreatedAtDesc);
    assert!(!spec.is_empty());
}

#[test]
fn distribution_filter_empty_and_populated() {
    let empty = DistributionFilter::default();
    assert!(empty.is_empty());

    let populated = DistributionFilter {
        tenant_id: Some("default".to_string()),
        dataset_id: Some("urn:dataset:1".to_string()),
        access_service: Some("urn:service:1".to_string()),
        format: Some("application/json".to_string()),
        title: Some("JSON Dist".to_string()),
        created_after: None,
        created_before: None,
    };
    assert!(!populated.is_empty());
    assert!(populated.validate().is_ok());
}

#[test]
fn data_service_filter_empty_and_populated() {
    let empty = DataServiceFilter::default();
    assert!(empty.is_empty());

    let populated = DataServiceFilter {
        tenant_id: Some("default".to_string()),
        catalog_id: Some("urn:catalog:1".to_string()),
        endpoint_url: Some("https://example.com/api".to_string()),
        title: Some("API Service".to_string()),
        creator: Some("UPM".to_string()),
        main_data_service: Some(true),
        created_after: None,
        created_before: None,
    };
    assert!(!populated.is_empty());
    assert!(populated.validate().is_ok());
}

#[test]
fn odrl_policy_and_template_filters() {
    let policy_empty = OdrlPolicyFilter::default();
    assert!(policy_empty.is_empty());

    let policy_pop = OdrlPolicyFilter {
        tenant_id: Some("default".to_string()),
        entity: Some("urn:dataset:1".to_string()),
        entity_type: Some("dataset".to_string()),
        source_template_id: Some("tmpl-1".to_string()),
        source_template_version: Some("1.0".to_string()),
        created_after: None,
        created_before: None,
    };
    assert!(!policy_pop.is_empty());
    assert!(policy_pop.validate().is_ok());

    let template_empty = PolicyTemplateFilter::default();
    assert!(template_empty.is_empty());

    let template_pop = PolicyTemplateFilter {
        tenant_id: Some("default".to_string()),
        id: Some("tmpl-1".to_string()),
        version: Some("1.0".to_string()),
        author: Some("UPM".to_string()),
        created_after: None,
        created_before: None,
    };
    assert!(!template_pop.is_empty());
    assert!(template_pop.validate().is_ok());
}
