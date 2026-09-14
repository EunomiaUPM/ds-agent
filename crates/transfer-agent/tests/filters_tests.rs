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

//! Integration tests for transfer-agent domain filters.

use chrono::{Duration, Utc};
use common::query::{QueryFilter, QuerySpec};
use transfer_agent::entities::filters::{TransferMessageFilter, TransferProcessFilter};

#[test]
fn transfer_process_filter_empty_and_populated() {
    let empty = TransferProcessFilter::default();
    assert!(empty.is_empty());
    assert!(empty.validate().is_ok());

    let now = Utc::now();
    let populated = TransferProcessFilter {
        state: Some("STARTED".to_string()),
        role: Some("CONSUMER".to_string()),
        protocol: Some("DSP_2025_1".to_string()),
        agreement_id: Some("urn:agreement:1".to_string()),
        associated_agent_peer: Some("urn:peer:1".to_string()),
        connector_instance_id: Some("urn:instance:1".to_string()),
        transfer_direction: Some("PULL".to_string()),
        created_after: Some(now),
        created_before: Some(now + Duration::hours(2)),
    };
    assert!(!populated.is_empty());
    assert!(populated.validate().is_ok());

    let invalid = TransferProcessFilter {
        state: None,
        role: None,
        protocol: None,
        agreement_id: None,
        associated_agent_peer: None,
        connector_instance_id: None,
        transfer_direction: None,
        created_after: Some(now + Duration::hours(1)),
        created_before: Some(now),
    };
    assert!(invalid.validate().is_err());
}

#[test]
fn transfer_message_filter_deserialization() {
    let json = serde_json::json!({
        "process_id": "proc-456",
        "protocol": "DSP_2025_1",
        "message_type": "TransferStartMessage",
        "direction": "OUTGOING",
        "limit": 100
    });
    let spec: QuerySpec<TransferMessageFilter> = serde_json::from_value(json).unwrap();
    assert_eq!(spec.filter.process_id.as_deref(), Some("proc-456"));
    assert_eq!(
        spec.filter.message_type.as_deref(),
        Some("TransferStartMessage")
    );
    assert_eq!(spec.filter.direction.as_deref(), Some("OUTGOING"));
    assert_eq!(spec.page.limit, 100);
    assert!(!spec.is_empty());
}
