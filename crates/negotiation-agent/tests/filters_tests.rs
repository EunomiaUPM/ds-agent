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

//! Integration tests for negotiation-agent domain filters.

use chrono::{Duration, Utc};
use common::query::{QueryFilter, QuerySpec};
use negotiation_agent::entities::filters::{
    AgreementFilter, NegotiationMessageFilter, NegotiationProcessFilter, OfferFilter,
};

#[test]
fn negotiation_process_filter_empty_and_populated() {
    let empty = NegotiationProcessFilter::default();
    assert!(empty.is_empty());
    assert!(empty.validate().is_ok());

    let now = Utc::now();
    let populated = NegotiationProcessFilter {
        id: Some("np-1".to_string()),
        tenant_id: Some("tenant-1".to_string()),
        state: Some("REQUESTED".to_string()),
        role: Some("CONSUMER".to_string()),
        protocol: Some("DSP_2025_1".to_string()),
        associated_agent_peer: Some("urn:peer:1".to_string()),
        created_after: Some(now),
        created_before: Some(now + Duration::hours(1)),
    };
    assert!(!populated.is_empty());
    assert!(populated.validate().is_ok());

    let invalid = NegotiationProcessFilter {
        id: None,
        tenant_id: None,
        state: None,
        role: None,
        protocol: None,
        associated_agent_peer: None,
        created_after: Some(now + Duration::hours(1)),
        created_before: Some(now),
    };
    assert!(invalid.validate().is_err());
}

#[test]
fn negotiation_message_filter_deserialization() {
    let json = serde_json::json!({
        "tenant_id": "tenant-1",
        "process_id": "proc-123",
        "protocol": "DSP_2025_1",
        "message_type": "ContractRequestMessage",
        "direction": "INCOMING",
        "limit": 25,
        "sort": "created_at_desc"
    });
    let spec: QuerySpec<NegotiationMessageFilter> = serde_json::from_value(json).unwrap();
    assert_eq!(spec.filter.tenant_id.as_deref(), Some("tenant-1"));
    assert_eq!(spec.filter.process_id.as_deref(), Some("proc-123"));
    assert_eq!(spec.filter.direction.as_deref(), Some("INCOMING"));
    assert_eq!(spec.page.limit, 25);
    assert!(spec.sort.is_descending());
    assert!(!spec.is_empty());
}

#[test]
fn agreement_and_offer_filters() {
    let agreement_empty = AgreementFilter::default();
    assert!(agreement_empty.is_empty());

    let agreement_pop = AgreementFilter {
        id: None,
        tenant_id: Some("tenant-1".to_string()),
        process_id: Some("proc-1".to_string()),
        consumer_id: Some("urn:consumer".to_string()),
        provider_id: Some("urn:provider".to_string()),
        target: Some("urn:target".to_string()),
        state: Some("FINALIZED".to_string()),
        created_after: None,
        created_before: None,
    };
    assert!(!agreement_pop.is_empty());
    assert!(agreement_pop.validate().is_ok());

    let offer_empty = OfferFilter::default();
    assert!(offer_empty.is_empty());

    let offer_pop = OfferFilter {
        id: None,
        tenant_id: Some("tenant-1".to_string()),
        process_id: Some("proc-1".to_string()),
        offer_id: Some("offer-1".to_string()),
        target: Some("urn:target".to_string()),
        created_after: None,
        created_before: None,
    };
    assert!(!offer_pop.is_empty());
    assert!(offer_pop.validate().is_ok());
}
