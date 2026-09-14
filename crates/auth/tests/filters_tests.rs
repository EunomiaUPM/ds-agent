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

//! Integration tests for ssi-auth domain filters.

use auth::entities::filters::{ParticipantFilter, RecvGrantFilter, SentGrantFilter};
use chrono::{Duration, Utc};
use common::query::{QueryFilter, QuerySpec, Sort};
use ymir::types::gnap::grant_request::GrantKind;
use ymir::types::gnap::GrantStatus;
use ymir::types::participants::ParticipantType;

#[test]
fn participant_filter_empty_and_populated() {
    let empty = ParticipantFilter::default();
    assert!(empty.is_empty());
    assert!(empty.validate().is_ok());

    let populated = ParticipantFilter {
        r#type: Some(ParticipantType::Agent),
        participant_nick: Some("Alice".to_string()),
        participant_id: Some("did:example:alice".to_string()),
        exclude_myself: Some(true),
        created_after: None,
        created_before: None,
    };
    assert!(!populated.is_empty());
    assert!(populated.validate().is_ok());
}

#[test]
fn participant_filter_date_range_validation() {
    let now = Utc::now();
    let valid = ParticipantFilter {
        created_after: Some(now),
        created_before: Some(now + Duration::hours(1)),
        ..Default::default()
    };
    assert!(valid.validate().is_ok());

    let invalid = ParticipantFilter {
        created_after: Some(now + Duration::hours(1)),
        created_before: Some(now),
        ..Default::default()
    };
    assert!(invalid.validate().is_err());
}

#[test]
fn participant_query_deserialization() {
    let json = serde_json::json!({
        "type": "Agent",
        "participantNick": "Alice",
        "excludeMyself": true,
        "limit": 25,
        "sort": "created_at_desc"
    });
    let spec: QuerySpec<ParticipantFilter> = serde_json::from_value(json).unwrap();
    assert_eq!(spec.filter.r#type, Some(ParticipantType::Agent));
    assert_eq!(spec.filter.participant_nick.as_deref(), Some("Alice"));
    assert_eq!(spec.filter.exclude_myself, Some(true));
    assert_eq!(spec.page.limit, 25);
    assert_eq!(spec.sort, Sort::CreatedAtDesc);
}

#[test]
fn participant_query_urlencoded_deserialization() {
    let qs = "limit=10&sort=created_at_desc";
    let spec: QuerySpec<ParticipantFilter> = serde_urlencoded::from_str(qs).unwrap();
    assert_eq!(spec.page.limit, 10);
    assert_eq!(spec.sort, Sort::CreatedAtDesc);
    assert!(spec.filter.is_empty());
}

#[test]
fn sent_grant_filter_empty_and_populated() {
    let empty = SentGrantFilter::default();
    assert!(empty.is_empty());
    assert!(empty.validate().is_ok());

    let populated = SentGrantFilter {
        participant_id: Some("did:example:bob".to_string()),
        participant_nick: Some("Bob".to_string()),
        status: Some(GrantStatus::Approved),
        kind: Some(GrantKind::AccessToken),
        created_after: None,
        created_before: None,
    };
    assert!(!populated.is_empty());
    assert!(populated.validate().is_ok());
}

#[test]
fn sent_grant_query_deserialization() {
    let json = serde_json::json!({
        "participantId": "did:example:123",
        "status": "Approved",
        "limit": 10,
        "sort": "created_at_asc"
    });
    let spec: QuerySpec<SentGrantFilter> = serde_json::from_value(json).unwrap();
    assert_eq!(
        spec.filter.participant_id.as_deref(),
        Some("did:example:123")
    );
    assert_eq!(spec.filter.status, Some(GrantStatus::Approved));
    assert_eq!(spec.page.limit, 10);
    assert_eq!(spec.sort, Sort::CreatedAtAsc);
}

#[test]
fn recv_grant_filter_empty_and_populated() {
    let empty = RecvGrantFilter::default();
    assert!(empty.is_empty());
    assert!(empty.validate().is_ok());

    let populated = RecvGrantFilter {
        participant_nick: Some("ProviderX".to_string()),
        status: Some(GrantStatus::Pending),
        kind: Some(GrantKind::AccessToken),
        created_after: None,
        created_before: None,
    };
    assert!(!populated.is_empty());
    assert!(populated.validate().is_ok());
}

#[test]
fn recv_grant_query_deserialization() {
    let json = serde_json::json!({
        "participantNick": "ProviderX",
        "limit": 15,
        "sort": "updated_at_desc"
    });
    let spec: QuerySpec<RecvGrantFilter> = serde_json::from_value(json).unwrap();
    assert_eq!(spec.filter.participant_nick.as_deref(), Some("ProviderX"));
    assert_eq!(spec.page.limit, 15);
    assert_eq!(spec.sort, Sort::UpdatedAtDesc);
}
