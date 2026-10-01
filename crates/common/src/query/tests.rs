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

//! QuerySpec deserialization and the DateRange invariant, with a sample filter.

use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use ymir::errors::Outcome;

use crate::query::{validate_date_range, DateRange, Page, QueryFilter, QuerySpec, Sort};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct DummyFilter {
    pub tenant_id: Option<String>,
    pub status: Option<String>,
    #[serde(flatten)]
    pub dates: DateRange,
}

impl QueryFilter for DummyFilter {
    fn validate(&self) -> Outcome<()> {
        self.dates.validate()
    }
}

/// Filter, page and sort are read from one flat JSON object.
#[test]
fn query_spec_reads_flat_json() {
    let json = r#"{
        "tenant_id": "tenant-1",
        "status": "ACTIVE",
        "limit": 50,
        "cursor": "token-123",
        "sort": "created_at_asc"
    }"#;

    let spec: QuerySpec<DummyFilter, Sort> =
        serde_json::from_str(json).expect("should deserialize");
    assert_eq!(spec.filter.tenant_id, Some("tenant-1".to_string()));
    assert_eq!(spec.filter.status, Some("ACTIVE".to_string()));
    assert_eq!(spec.page.limit, 50);
    assert_eq!(spec.page.cursor, Some("token-123".to_string()));
    assert_eq!(spec.sort, Sort::CreatedAtAsc);
}

/// Query strings deserialize too, and an unknown sort falls back to `Sort::Other`.
#[test]
fn query_spec_reads_query_strings() {
    let qs = "limit=10&sort=created_at_desc";
    let spec: QuerySpec<DummyFilter, Sort> =
        serde_urlencoded::from_str(qs).expect("should deserialize urlencoded query string");
    assert_eq!(spec.page.limit, 10);
    assert_eq!(spec.page.cursor, None);
    assert_eq!(spec.page.page, None);
    assert_eq!(spec.sort, Sort::CreatedAtDesc);

    let qs_with_page = "limit=25&page=3&sort=created_at_asc&status=ACTIVE";
    let spec2: QuerySpec<DummyFilter, Sort> =
        serde_urlencoded::from_str(qs_with_page).expect("should deserialize urlencoded with page");
    assert_eq!(spec2.page.limit, 25);
    assert_eq!(spec2.page.page, Some(3));
    assert_eq!(spec2.filter.status, Some("ACTIVE".to_string()));
    assert_eq!(spec2.sort, Sort::CreatedAtAsc);

    let qs_unknown_sort = "limit=10&sort=state_asc";
    let spec3: QuerySpec<DummyFilter, Sort> = serde_urlencoded::from_str(qs_unknown_sort)
        .expect("should deserialize unknown sort to Other");
    assert_eq!(spec3.sort, Sort::Other);
}

/// `into_parts` hands back the filter, page and sort it was built with.
#[test]
fn into_parts_returns_filter_page_and_sort() {
    let spec = QuerySpec::new(
        DummyFilter {
            tenant_id: Some("t1".into()),
            status: None,
            dates: DateRange::default(),
        },
        Page::new(10, None),
        Sort::CreatedAtDesc,
    );

    let (filter, page, sort) = spec.into_parts();
    assert_eq!(filter.tenant_id, Some("t1".into()));
    assert_eq!(page.limit, 10);
    assert_eq!(sort, Sort::CreatedAtDesc);
}

/// `after` must be strictly earlier than `before`, alone and inside a QuerySpec.
#[test]
fn date_range_requires_after_before_before() {
    let now = Utc::now();
    let past = now - Duration::hours(1);

    assert!(validate_date_range(Some(past), Some(now)).is_ok());
    assert!(validate_date_range(Some(now), Some(past)).is_err());
    assert!(validate_date_range(Some(now), Some(now)).is_err());

    let mut invalid_spec = QuerySpec::new(
        DummyFilter {
            tenant_id: None,
            status: None,
            dates: DateRange::new(Some(now), Some(past)),
        },
        Page::new(20, None),
        Sort::default(),
    );

    assert!(invalid_spec.validate().is_err());
    invalid_spec.filter.dates = DateRange::new(Some(past), Some(now));
    assert!(invalid_spec.validate().is_ok());
}
