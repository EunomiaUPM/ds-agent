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

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use base64::Engine;
use chrono::{Duration, Utc};
use compact_str::CompactString;
use urn::Urn;

use crate::data::repo::transfer_process::{
    MockTransferProcessRepoTrait, TransferProcessRepoErrors,
};
use crate::data::repo::transfer_process_identifier::{
    MockTransferIdentifierRepoTrait, TransferIdentifierRepoErrors,
};
use crate::entities::commands::{EditTransferProcessCommand, NewTransferProcessCommand};
use crate::entities::filters::TransferProcessFilter;
use crate::entities::ids::{ParticipantId, TenantId, TransferProcessId};
use crate::entities::protocol::{
    ProtocolId, ProtocolState, StateMetadata, TransferCorrelation, TransferRole,
};
use crate::entities::transfer_process::TransferProcess;
use crate::entities::transfer_process_identifier::TransferProcessIdentifier;
use crate::services::transfer_process::TransferProcessServiceTrait;
use crate::services::transfer_process::service::TransferProcessService;
use common::auth::access::AccessScope;
use common::auth::claims::RbacRole;
use common::batch_requests::BatchRequests;
use common::query::{Page, Sort};
use ymir::errors::RepoIntoErrors;

/// Unrestricted scope used by the bulk of the tests, which exercise behaviour
/// unrelated to tenant isolation.
fn admin_scope() -> AccessScope {
    AccessScope::from_role(RbacRole::Admin, &"tenant-1".to_string())
}

/// Scope restricted to a single tenant, for the isolation-specific tests.
fn tenant_scope(tenant: &str) -> AccessScope {
    AccessScope::from_role(RbacRole::Owner, &tenant.to_string())
}

// Unit tests for TransferProcessService. Both repository dependencies
// (TransferProcessRepoTrait and TransferIdentifierRepoTrait) are replaced
// with mockall mocks so no database is required.

// ─── Fixtures ────────────────────────────────────────────────────────────────

fn p_urn(n: u32) -> Urn {
    Urn::from_str(&format!("urn:uuid:{:08x}-0000-0000-0000-000000000000", n)).expect("static URN")
}

fn empty_correlation() -> TransferCorrelation {
    TransferCorrelation {
        identifiers: HashMap::new(),
        consumer_pid: None,
        provider_pid: None,
        agreement_id: None,
        callback_address: None,
        peer_participant_id: None,
    }
}

// make_process(n) produces a deterministic process: URN encodes n, and
// created_at/updated_at are offset by n so distinct processes have distinct timestamps.
fn make_process(n: u32) -> TransferProcess {
    let now = Utc::now();
    TransferProcess::rehydrate(
        TransferProcessId::new(p_urn(n)),
        "tenant-1".to_string(),
        TransferRole::Provider,
        now - Duration::seconds(n as i64 * 10),
        now - Duration::seconds(n as i64 * 5),
        0,
        ProtocolId::Dsp2024,
        ProtocolState(CompactString::from("STARTED")),
        StateMetadata::empty(),
        empty_correlation(),
        serde_json::json!({}),
        None,
    )
}

fn make_identifier(process_urn: Urn, key: &str, val: &str) -> TransferProcessIdentifier {
    TransferProcessIdentifier::new(process_urn, key.to_string(), Some(val.to_string()))
}

fn empty_filter() -> TransferProcessFilter {
    TransferProcessFilter {
        tenant_id: None,
        protocol: None,
        state: None,
        role: None,
        agreement_id: None,
        peer_participant_id: None,
        created_after: None,
        created_before: None,
    }
}

fn default_page() -> Page {
    Page {
        limit: 20,
        cursor: None,
    }
}

fn make_new_cmd(identifiers: Option<HashMap<String, String>>) -> NewTransferProcessCommand {
    NewTransferProcessCommand {
        id: None,
        tenant_id: Some("tenant-1".to_string()),
        role: TransferRole::Consumer,
        protocol: ProtocolId::Dsp2024,
        initial_state: ProtocolState(CompactString::from("INITIATED")),
        initial_state_metadata: StateMetadata::empty(),
        callback_address: None,
        connector_instance_id: None,
        agreement_id: p_urn(99),
        peer_participant_id: ParticipantId::new(p_urn(100)),
        identifiers,
        properties: None,
    }
}

fn make_edit_cmd(
    state: Option<&str>,
    identifiers: Option<HashMap<String, String>>,
) -> EditTransferProcessCommand {
    EditTransferProcessCommand {
        state: state.map(|s| ProtocolState(CompactString::from(s))),
        state_metadata: None,
        identifiers,
        properties: None,
        error_details: None,
    }
}

fn make_svc(
    proc_repo: MockTransferProcessRepoTrait,
    id_repo: MockTransferIdentifierRepoTrait,
) -> TransferProcessService {
    TransferProcessService::new(Arc::new(proc_repo), Arc::new(id_repo))
}

fn io_err() -> Box<dyn std::error::Error + Send + Sync> {
    Box::new(std::io::Error::from(std::io::ErrorKind::Other))
}

// ─── get_all ─────────────────────────────────────────────────────────────────
// get_all fires two repo calls concurrently via tokio::try_join!:
// get_all_transfer_processes (items) and count_transfer_processes (total).
// Identifiers are fetched in a third sequential call and merged into each view.

#[tokio::test]
async fn get_all_empty_result() {
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_all_transfer_processes()
        .returning(|_, _, _| Ok(vec![]));
    proc_repo
        .expect_count_transfer_processes()
        .returning(|_| Ok(0));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_batch_process_id()
        .returning(|_| Ok(vec![]));

    let svc = make_svc(proc_repo, id_repo);
    let result = svc
        .get_all(
            &admin_scope(),
            &empty_filter(),
            &default_page(),
            &Sort::CreatedAtDesc,
        )
        .await
        .unwrap();

    assert!(result.items.is_empty());
    assert!(result.next_cursor.is_none());
    assert_eq!(result.total, Some(0));
}

// Cursor is base64url(RFC3339 timestamp). When the returned slice fills the
// requested limit exactly, the service encodes the last item's timestamp as the
// next-page cursor; a shorter slice means there is no next page.
#[tokio::test]
async fn get_all_full_page_produces_cursor() {
    let process = make_process(1);
    let pc = process.clone();
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_all_transfer_processes()
        .returning(move |_, _, _| Ok(vec![pc.clone()]));
    proc_repo
        .expect_count_transfer_processes()
        .returning(|_| Ok(1));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_batch_process_id()
        .returning(|_| Ok(vec![]));

    let svc = make_svc(proc_repo, id_repo);
    let result = svc
        .get_all(
            &admin_scope(),
            &empty_filter(),
            &Page {
                limit: 1,
                cursor: None,
            },
            &Sort::CreatedAtDesc,
        )
        .await
        .unwrap();

    let expected =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(process.created_at().to_rfc3339());
    assert_eq!(result.items.len(), 1);
    assert_eq!(result.next_cursor.as_deref(), Some(expected.as_str()));
}

#[tokio::test]
async fn get_all_partial_page_no_cursor() {
    let pc = make_process(1);
    let pcc = pc.clone();
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_all_transfer_processes()
        .returning(move |_, _, _| Ok(vec![pcc.clone()]));
    proc_repo
        .expect_count_transfer_processes()
        .returning(|_| Ok(1));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_batch_process_id()
        .returning(|_| Ok(vec![]));

    let svc = make_svc(proc_repo, id_repo);
    let result = svc
        .get_all(
            &admin_scope(),
            &empty_filter(),
            &Page {
                limit: 5,
                cursor: None,
            },
            &Sort::CreatedAtDesc,
        )
        .await
        .unwrap();

    assert!(result.next_cursor.is_none());
}

// encode_cursor in the service switches the timestamp field based on sort:
// UpdatedAtDesc → updated_at; everything else → created_at. We need a process
// where both fields differ to confirm the right one is picked.
#[tokio::test]
async fn get_all_cursor_uses_updated_at_for_sort_updated_at_desc() {
    let now = Utc::now();
    let created = now - Duration::hours(2);
    let updated = now - Duration::hours(1);
    let process = TransferProcess::rehydrate(
        TransferProcessId::new(p_urn(1)),
        "t".to_string(),
        TransferRole::Provider,
        created,
        updated,
        0,
        ProtocolId::Dsp2024,
        ProtocolState(CompactString::from("STARTED")),
        StateMetadata::empty(),
        empty_correlation(),
        serde_json::json!({}),
        None,
    );
    let expected = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(updated.to_rfc3339());
    let pc = process.clone();
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_all_transfer_processes()
        .returning(move |_, _, _| Ok(vec![pc.clone()]));
    proc_repo
        .expect_count_transfer_processes()
        .returning(|_| Ok(1));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_batch_process_id()
        .returning(|_| Ok(vec![]));

    let svc = make_svc(proc_repo, id_repo);
    let result = svc
        .get_all(
            &admin_scope(),
            &empty_filter(),
            &Page {
                limit: 1,
                cursor: None,
            },
            &Sort::UpdatedAtDesc,
        )
        .await
        .unwrap();

    assert_eq!(result.next_cursor, Some(expected));
}

#[tokio::test]
async fn get_all_cursor_uses_created_at_for_sort_created_at_asc() {
    let now = Utc::now();
    let created = now - Duration::hours(2);
    let updated = now - Duration::hours(1);
    let process = TransferProcess::rehydrate(
        TransferProcessId::new(p_urn(1)),
        "tenant-2".to_string(),
        TransferRole::Provider,
        created,
        updated,
        0,
        ProtocolId::Dsp2024,
        ProtocolState(CompactString::from("STARTED")),
        StateMetadata::empty(),
        empty_correlation(),
        serde_json::json!({}),
        None,
    );
    let expected = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(created.to_rfc3339());
    let pc = process.clone();
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_all_transfer_processes()
        .returning(move |_, _, _| Ok(vec![pc.clone()]));
    proc_repo
        .expect_count_transfer_processes()
        .returning(|_| Ok(1));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_batch_process_id()
        .returning(|_| Ok(vec![]));

    let svc = make_svc(proc_repo, id_repo);
    let result = svc
        .get_all(
            &admin_scope(),
            &empty_filter(),
            &Page {
                limit: 1,
                cursor: None,
            },
            &Sort::CreatedAtAsc,
        )
        .await
        .unwrap();

    assert_eq!(result.next_cursor, Some(expected));
}

// TransferProcessView::assemble promotes "consumerPid" / "providerPid" keys from
// the identifiers map into the top-level correlation fields if those fields are
// not already set on the process itself.
#[tokio::test]
async fn get_all_merges_identifiers_and_promotes_consumer_pid() {
    let p = make_process(1);
    let id = make_identifier(p_urn(1), "consumerPid", "cpid-1");
    let (pc, idc) = (p.clone(), id.clone());
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_all_transfer_processes()
        .returning(move |_, _, _| Ok(vec![pc.clone()]));
    proc_repo
        .expect_count_transfer_processes()
        .returning(|_| Ok(1));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_batch_process_id()
        .returning(move |_| Ok(vec![idc.clone()]));

    let svc = make_svc(proc_repo, id_repo);
    let result = svc
        .get_all(
            &admin_scope(),
            &empty_filter(),
            &default_page(),
            &Sort::CreatedAtDesc,
        )
        .await
        .unwrap();

    let view = &result.items[0];
    assert_eq!(
        view.correlation
            .identifiers
            .get("consumerPid")
            .map(String::as_str),
        Some("cpid-1")
    );
    assert_eq!(view.correlation.consumer_pid.as_deref(), Some("cpid-1"));
}

#[tokio::test]
async fn get_all_total_forwarded_from_count() {
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_all_transfer_processes()
        .returning(|_, _, _| Ok(vec![]));
    proc_repo
        .expect_count_transfer_processes()
        .returning(|_| Ok(42));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_batch_process_id()
        .returning(|_| Ok(vec![]));

    let svc = make_svc(proc_repo, id_repo);
    let result = svc
        .get_all(
            &admin_scope(),
            &empty_filter(),
            &default_page(),
            &Sort::CreatedAtDesc,
        )
        .await
        .unwrap();

    assert_eq!(result.total, Some(42));
}

#[tokio::test]
async fn get_all_filter_by_protocol_passed_through() {
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_all_transfer_processes()
        .withf(|f, _, _| f.protocol == Some(ProtocolId::Dsp2025_1))
        .returning(|_, _, _| Ok(vec![]));
    proc_repo
        .expect_count_transfer_processes()
        .returning(|_| Ok(0));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_batch_process_id()
        .returning(|_| Ok(vec![]));

    let filter = TransferProcessFilter {
        protocol: Some(ProtocolId::Dsp2025_1),
        ..empty_filter()
    };
    let svc = make_svc(proc_repo, id_repo);
    svc.get_all(
        &admin_scope(),
        &filter,
        &default_page(),
        &Sort::CreatedAtDesc,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn get_all_filter_by_role_passed_through() {
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_all_transfer_processes()
        .withf(|f, _, _| f.role == Some(TransferRole::Consumer))
        .returning(|_, _, _| Ok(vec![]));
    proc_repo
        .expect_count_transfer_processes()
        .returning(|_| Ok(0));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_batch_process_id()
        .returning(|_| Ok(vec![]));

    let filter = TransferProcessFilter {
        role: Some(TransferRole::Consumer),
        ..empty_filter()
    };
    let svc = make_svc(proc_repo, id_repo);
    svc.get_all(
        &admin_scope(),
        &filter,
        &default_page(),
        &Sort::CreatedAtDesc,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn get_all_filter_by_state_passed_through() {
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_all_transfer_processes()
        .withf(|f, _, _| f.state.as_ref().map(|s| s.0.as_str()) == Some("COMPLETED"))
        .returning(|_, _, _| Ok(vec![]));
    proc_repo
        .expect_count_transfer_processes()
        .returning(|_| Ok(0));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_batch_process_id()
        .returning(|_| Ok(vec![]));

    let filter = TransferProcessFilter {
        state: Some(ProtocolState(CompactString::from("COMPLETED"))),
        ..empty_filter()
    };
    let svc = make_svc(proc_repo, id_repo);
    svc.get_all(
        &admin_scope(),
        &filter,
        &default_page(),
        &Sort::CreatedAtDesc,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn get_all_filter_by_tenant_id_passed_through() {
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_all_transfer_processes()
        .withf(|f, _, _| f.tenant_id.as_ref().map(|t| t.as_str()) == Some("acme"))
        .returning(|_, _, _| Ok(vec![]));
    proc_repo
        .expect_count_transfer_processes()
        .returning(|_| Ok(0));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_batch_process_id()
        .returning(|_| Ok(vec![]));

    let filter = TransferProcessFilter {
        tenant_id: Some("acme".to_string()),
        ..empty_filter()
    };
    let svc = make_svc(proc_repo, id_repo);
    svc.get_all(
        &admin_scope(),
        &filter,
        &default_page(),
        &Sort::CreatedAtDesc,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn get_all_filter_by_date_range_passed_through() {
    let after = Utc::now() - Duration::days(7);
    let before = Utc::now();
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_all_transfer_processes()
        .withf(move |f, _, _| f.created_after == Some(after) && f.created_before == Some(before))
        .returning(|_, _, _| Ok(vec![]));
    proc_repo
        .expect_count_transfer_processes()
        .returning(|_| Ok(0));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_batch_process_id()
        .returning(|_| Ok(vec![]));

    let filter = TransferProcessFilter {
        created_after: Some(after),
        created_before: Some(before),
        ..empty_filter()
    };
    let svc = make_svc(proc_repo, id_repo);
    svc.get_all(
        &admin_scope(),
        &filter,
        &default_page(),
        &Sort::CreatedAtDesc,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn get_all_page_cursor_passed_through() {
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_all_transfer_processes()
        .withf(|_, p, _| p.limit == 10 && p.cursor.as_deref() == Some("tok"))
        .returning(|_, _, _| Ok(vec![]));
    proc_repo
        .expect_count_transfer_processes()
        .returning(|_| Ok(0));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_batch_process_id()
        .returning(|_| Ok(vec![]));

    let svc = make_svc(proc_repo, id_repo);
    svc.get_all(
        &admin_scope(),
        &empty_filter(),
        &Page {
            limit: 10,
            cursor: Some("tok".to_string()),
        },
        &Sort::CreatedAtDesc,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn get_all_propagates_process_repo_error() {
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_all_transfer_processes()
        .returning(|_, _, _| {
            Err(TransferProcessRepoErrors::ErrorFetchingTransferProcess(io_err()).into_errors())
        });
    proc_repo
        .expect_count_transfer_processes()
        .returning(|_| Ok(0));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_batch_process_id()
        .returning(|_| Ok(vec![]));

    let svc = make_svc(proc_repo, id_repo);
    assert!(
        svc.get_all(
            &admin_scope(),
            &empty_filter(),
            &default_page(),
            &Sort::CreatedAtDesc
        )
        .await
        .is_err()
    );
}

#[tokio::test]
async fn get_all_propagates_count_repo_error() {
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_all_transfer_processes()
        .returning(|_, _, _| Ok(vec![]));
    proc_repo.expect_count_transfer_processes().returning(|_| {
        Err(TransferProcessRepoErrors::ErrorFetchingTransferProcess(io_err()).into_errors())
    });
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_batch_process_id()
        .returning(|_| Ok(vec![]));

    let svc = make_svc(proc_repo, id_repo);
    assert!(
        svc.get_all(
            &admin_scope(),
            &empty_filter(),
            &default_page(),
            &Sort::CreatedAtDesc
        )
        .await
        .is_err()
    );
}

#[tokio::test]
async fn get_all_propagates_identifier_repo_error() {
    let pc = make_process(1);
    let pcc = pc.clone();
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_all_transfer_processes()
        .returning(move |_, _, _| Ok(vec![pcc.clone()]));
    proc_repo
        .expect_count_transfer_processes()
        .returning(|_| Ok(1));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_batch_process_id()
        .returning(|_| {
            Err(
                TransferIdentifierRepoErrors::ErrorFetchingTransferIdentifier(io_err())
                    .into_errors(),
            )
        });

    let svc = make_svc(proc_repo, id_repo);
    assert!(
        svc.get_all(
            &admin_scope(),
            &empty_filter(),
            &default_page(),
            &Sort::CreatedAtDesc
        )
        .await
        .is_err()
    );
}

// ─── get_one ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn get_one_returns_view_with_identifiers() {
    let p = make_process(1);
    let id = make_identifier(p_urn(1), "key", "value");
    let (pc, idc) = (p.clone(), id.clone());
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_transfer_process_by_id()
        .returning(move |_| Ok(Some(pc.clone())));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_process_id()
        .returning(move |_| Ok(vec![idc.clone()]));

    let svc = make_svc(proc_repo, id_repo);
    let view = svc.get_one(&admin_scope(), p.id().as_urn()).await.unwrap();

    assert_eq!(&view.id, p.id());
    assert_eq!(
        view.correlation.identifiers.get("key").map(String::as_str),
        Some("value")
    );
}

#[tokio::test]
async fn get_one_promotes_consumer_pid_and_provider_pid_from_identifiers() {
    let p = make_process(1);
    let ids = vec![
        make_identifier(p_urn(1), "consumerPid", "cpid-abc"),
        make_identifier(p_urn(1), "providerPid", "ppid-xyz"),
    ];
    let (pc, idsc) = (p.clone(), ids.clone());
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_transfer_process_by_id()
        .returning(move |_| Ok(Some(pc.clone())));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_process_id()
        .returning(move |_| Ok(idsc.clone()));

    let svc = make_svc(proc_repo, id_repo);
    let view = svc.get_one(&admin_scope(), p.id().as_urn()).await.unwrap();

    assert_eq!(view.correlation.consumer_pid.as_deref(), Some("cpid-abc"));
    assert_eq!(view.correlation.provider_pid.as_deref(), Some("ppid-xyz"));
}

#[tokio::test]
async fn get_one_without_identifiers_returns_empty_map() {
    let p = make_process(1);
    let pc = p.clone();
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_transfer_process_by_id()
        .returning(move |_| Ok(Some(pc.clone())));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_process_id()
        .returning(|_| Ok(vec![]));

    let svc = make_svc(proc_repo, id_repo);
    let view = svc.get_one(&admin_scope(), p.id().as_urn()).await.unwrap();

    assert!(view.correlation.identifiers.is_empty());
    assert!(view.correlation.consumer_pid.is_none());
    assert!(view.correlation.provider_pid.is_none());
}

#[tokio::test]
async fn get_one_not_found_returns_error() {
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_transfer_process_by_id()
        .returning(|_| Ok(None));
    let id_repo = MockTransferIdentifierRepoTrait::new();

    let svc = make_svc(proc_repo, id_repo);
    assert!(svc.get_one(&admin_scope(), &p_urn(999)).await.is_err());
}

#[tokio::test]
async fn get_one_propagates_process_repo_error() {
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_transfer_process_by_id()
        .returning(|_| {
            Err(TransferProcessRepoErrors::ErrorFetchingTransferProcess(io_err()).into_errors())
        });
    let id_repo = MockTransferIdentifierRepoTrait::new();

    let svc = make_svc(proc_repo, id_repo);
    assert!(svc.get_one(&admin_scope(), &p_urn(1)).await.is_err());
}

#[tokio::test]
async fn get_one_propagates_identifier_repo_error() {
    let p = make_process(1);
    let pc = p.clone();
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_transfer_process_by_id()
        .returning(move |_| Ok(Some(pc.clone())));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_process_id()
        .returning(|_| {
            Err(
                TransferIdentifierRepoErrors::ErrorFetchingTransferIdentifier(io_err())
                    .into_errors(),
            )
        });

    let svc = make_svc(proc_repo, id_repo);
    assert!(svc.get_one(&admin_scope(), p.id().as_urn()).await.is_err());
}

// ─── tenant isolation (AccessScope) ───────────────────────────────────────────
// These exercise the rules the service now owns on behalf of both transports.

#[tokio::test]
async fn get_one_foreign_tenant_returns_not_found() {
    // Process belongs to tenant-1; an owner scoped to tenant-2 must get a 404.
    let p = make_process(1); // tenant-1
    let pc = p.clone();
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_transfer_process_by_id()
        .returning(move |_| Ok(Some(pc.clone())));
    let id_repo = MockTransferIdentifierRepoTrait::new();

    let svc = make_svc(proc_repo, id_repo);
    assert!(
        svc.get_one(&tenant_scope("tenant-2"), p.id().as_urn())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn create_forces_caller_tenant_for_non_admin() {
    // Even though the body asks for tenant-1, an owner scoped to tenant-2 must
    // have the record created under tenant-2.
    let pc = make_process(1);
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_create_transfer_process()
        .withf(|cmd| cmd.tenant_id.as_ref().map(|t| t.as_str()) == Some("tenant-2"))
        .returning(move |_| Ok(pc.clone()));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo.expect_upsert_identifier().times(0);

    let svc = make_svc(proc_repo, id_repo);
    // make_new_cmd sets tenant_id = tenant-1; the scope must override it.
    svc.create(&tenant_scope("tenant-2"), &make_new_cmd(None))
        .await
        .unwrap();
}

#[tokio::test]
async fn edit_foreign_tenant_returns_not_found_without_mutating() {
    // ensure_access fetches the record, sees tenant-1, and rejects the tenant-2
    // owner before put_transfer_process is ever called.
    let pc = make_process(1); // tenant-1
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_transfer_process_by_id()
        .returning(move |_| Ok(Some(pc.clone())));
    proc_repo.expect_put_transfer_process().times(0);
    let id_repo = MockTransferIdentifierRepoTrait::new();

    let svc = make_svc(proc_repo, id_repo);
    assert!(
        svc.edit(
            &tenant_scope("tenant-2"),
            &p_urn(1),
            &make_edit_cmd(None, None)
        )
        .await
        .is_err()
    );
}

#[tokio::test]
async fn batch_filters_out_foreign_tenant_records() {
    // The repo returns a tenant-1 record; a tenant-2 owner must see none of it.
    let p = make_process(1); // tenant-1
    let pc = p.clone();
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_batch_transfer_processes()
        .returning(move |_| Ok(vec![pc.clone()]));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_batch_process_id()
        .returning(|_| Ok(vec![]));

    let svc = make_svc(proc_repo, id_repo);
    let views = svc
        .batch(
            &tenant_scope("tenant-2"),
            &BatchRequests {
                ids: vec![p_urn(1)],
            },
        )
        .await
        .unwrap();
    assert!(views.is_empty());
}

// ─── batch ────────────────────────────────────────────────────────────────────
// batch fetches all processes in a single repo call, then groups the flat
// identifier list by transfer_process_id before assembling each view.

#[tokio::test]
async fn batch_empty_ids_returns_empty_vec() {
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_batch_transfer_processes()
        .returning(|_| Ok(vec![]));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_batch_process_id()
        .returning(|_| Ok(vec![]));

    let svc = make_svc(proc_repo, id_repo);
    let views = svc
        .batch(&admin_scope(), &BatchRequests { ids: vec![] })
        .await
        .unwrap();

    assert!(views.is_empty());
}

#[tokio::test]
async fn batch_returns_one_view_per_process() {
    let p1 = make_process(1);
    let p2 = make_process(2);
    let (p1c, p2c) = (p1.clone(), p2.clone());
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_batch_transfer_processes()
        .returning(move |_| Ok(vec![p1c.clone(), p2c.clone()]));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_batch_process_id()
        .returning(|_| Ok(vec![]));

    let svc = make_svc(proc_repo, id_repo);
    let views = svc
        .batch(
            &admin_scope(),
            &BatchRequests {
                ids: vec![p_urn(1), p_urn(2)],
            },
        )
        .await
        .unwrap();

    assert_eq!(views.len(), 2);
}

// Identifiers arrive in a single flat vec; the service must group them by URN
// so that each view only receives its own identifiers, not those of siblings.
#[tokio::test]
async fn batch_groups_identifiers_per_process() {
    let p1 = make_process(1);
    let p2 = make_process(2);
    let id1 = make_identifier(p_urn(1), "k1", "v1");
    let id2 = make_identifier(p_urn(2), "k2", "v2");
    let (p1c, p2c) = (p1.clone(), p2.clone());
    let (id1c, id2c) = (id1.clone(), id2.clone());
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_batch_transfer_processes()
        .returning(move |_| Ok(vec![p1c.clone(), p2c.clone()]));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_batch_process_id()
        .returning(move |_| Ok(vec![id1c.clone(), id2c.clone()]));

    let svc = make_svc(proc_repo, id_repo);
    let views = svc
        .batch(
            &admin_scope(),
            &BatchRequests {
                ids: vec![p_urn(1), p_urn(2)],
            },
        )
        .await
        .unwrap();

    let v1 = views.iter().find(|v| &v.id == p1.id()).unwrap();
    let v2 = views.iter().find(|v| &v.id == p2.id()).unwrap();
    assert_eq!(
        v1.correlation.identifiers.get("k1").map(String::as_str),
        Some("v1")
    );
    assert!(!v1.correlation.identifiers.contains_key("k2"));
    assert_eq!(
        v2.correlation.identifiers.get("k2").map(String::as_str),
        Some("v2")
    );
    assert!(!v2.correlation.identifiers.contains_key("k1"));
}

#[tokio::test]
async fn batch_propagates_process_repo_error() {
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_batch_transfer_processes()
        .returning(|_| {
            Err(TransferProcessRepoErrors::ErrorFetchingTransferProcess(io_err()).into_errors())
        });
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_batch_process_id()
        .returning(|_| Ok(vec![]));

    let svc = make_svc(proc_repo, id_repo);
    assert!(
        svc.batch(
            &admin_scope(),
            &BatchRequests {
                ids: vec![p_urn(1)]
            }
        )
        .await
        .is_err()
    );
}

#[tokio::test]
async fn batch_propagates_identifier_repo_error() {
    let pc = make_process(1);
    let pcc = pc.clone();
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_batch_transfer_processes()
        .returning(move |_| Ok(vec![pcc.clone()]));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_batch_process_id()
        .returning(|_| {
            Err(
                TransferIdentifierRepoErrors::ErrorFetchingTransferIdentifier(io_err())
                    .into_errors(),
            )
        });

    let svc = make_svc(proc_repo, id_repo);
    assert!(
        svc.batch(
            &admin_scope(),
            &BatchRequests {
                ids: vec![p_urn(1)]
            }
        )
        .await
        .is_err()
    );
}

// ─── create ──────────────────────────────────────────────────────────────────
// After persisting the process, create upserts each identifier individually.
// The returned view is assembled directly from cmd.identifiers — it does NOT
// re-fetch from the repo, unlike edit.

#[tokio::test]
async fn create_without_identifiers_does_not_call_upsert() {
    let pc = make_process(1);
    let pcc = pc.clone();
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_create_transfer_process()
        .times(1)
        .returning(move |_| Ok(pcc.clone()));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo.expect_upsert_identifier().times(0); // enforces "never called"

    let svc = make_svc(proc_repo, id_repo);
    let view = svc
        .create(&admin_scope(), &make_new_cmd(None))
        .await
        .unwrap();

    assert!(&view.correlation.identifiers.is_empty());
}

#[tokio::test]
async fn create_with_one_identifier_upserts_once() {
    let pc = make_process(1);
    let pcc = pc.clone();
    let mut ids = HashMap::new();
    ids.insert("consumerPid".to_string(), "cpid-1".to_string());
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_create_transfer_process()
        .returning(move |_| Ok(pcc.clone()));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_upsert_identifier()
        .times(1)
        .returning(|_, _| {
            Ok(TransferProcessIdentifier::new(
                p_urn(0),
                "placeholder",
                Some("placeholder".to_string()),
            ))
        });

    let svc = make_svc(proc_repo, id_repo);
    let view = svc
        .create(&admin_scope(), &make_new_cmd(Some(ids)))
        .await
        .unwrap();

    assert_eq!(
        view.correlation
            .identifiers
            .get("consumerPid")
            .map(String::as_str),
        Some("cpid-1")
    );
}

#[tokio::test]
async fn create_with_multiple_identifiers_upserts_each() {
    let pc = make_process(1);
    let pcc = pc.clone();
    let mut ids = HashMap::new();
    ids.insert("consumerPid".to_string(), "cpid-1".to_string());
    ids.insert("contractId".to_string(), "ctr-99".to_string());
    ids.insert("datasetId".to_string(), "ds-42".to_string());
    let id_count = ids.len();
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_create_transfer_process()
        .returning(move |_| Ok(pcc.clone()));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_upsert_identifier()
        .times(id_count)
        .returning(|_, _| {
            Ok(TransferProcessIdentifier::new(
                p_urn(0),
                "placeholder",
                Some("placeholder".to_string()),
            ))
        });

    let svc = make_svc(proc_repo, id_repo);
    let view = svc
        .create(&admin_scope(), &make_new_cmd(Some(ids.clone())))
        .await
        .unwrap();

    for (k, v) in &ids {
        assert_eq!(
            view.correlation.identifiers.get(k).map(String::as_str),
            Some(v.as_str())
        );
    }
}

#[tokio::test]
async fn create_propagates_process_repo_error() {
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo.expect_create_transfer_process().returning(|_| {
        Err(TransferProcessRepoErrors::ErrorCreatingTransferProcess(io_err()).into_errors())
    });
    let id_repo = MockTransferIdentifierRepoTrait::new();

    let svc = make_svc(proc_repo, id_repo);
    assert!(
        svc.create(&admin_scope(), &make_new_cmd(None))
            .await
            .is_err()
    );
}

#[tokio::test]
async fn create_propagates_identifier_upsert_error() {
    let pc = make_process(1);
    let pcc = pc.clone();
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_create_transfer_process()
        .returning(move |_| Ok(pcc.clone()));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo.expect_upsert_identifier().returning(|_, _| {
        Err(TransferIdentifierRepoErrors::ErrorUpsertingTransferIdentifier(io_err()).into_errors())
    });

    let mut ids = HashMap::new();
    ids.insert("k".to_string(), "v".to_string());
    let svc = make_svc(proc_repo, id_repo);
    assert!(
        svc.create(&admin_scope(), &make_new_cmd(Some(ids)))
            .await
            .is_err()
    );
}

// ─── edit ─────────────────────────────────────────────────────────────────────
// edit: put the process → (optionally) upsert each identifier → re-fetch ALL
// identifiers from the repo to build the view. This differs from create, where
// the view is built from the command without a subsequent fetch.

#[tokio::test]
async fn edit_with_state_change() {
    let p = make_process(1);
    let pc = p.clone();
    let urn = p_urn(1);
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_put_transfer_process()
        .withf(|_, cmd| cmd.state.as_ref().map(|s| s.0.as_str()) == Some("COMPLETED"))
        .returning(move |_, _| Ok(pc.clone()));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_process_id()
        .returning(|_| Ok(vec![]));

    let svc = make_svc(proc_repo, id_repo);
    let view = svc
        .edit(
            &admin_scope(),
            &urn,
            &make_edit_cmd(Some("COMPLETED"), None),
        )
        .await
        .unwrap();

    assert_eq!(&view.id, p.id());
}

#[tokio::test]
async fn edit_without_identifiers_skips_upsert() {
    let p = make_process(1);
    let pc = p.clone();
    let urn = p_urn(1);
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_put_transfer_process()
        .returning(move |_, _| Ok(pc.clone()));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo.expect_upsert_identifier().times(0);
    id_repo
        .expect_get_identifiers_by_process_id()
        .returning(|_| Ok(vec![]));

    let svc = make_svc(proc_repo, id_repo);
    svc.edit(&admin_scope(), &urn, &make_edit_cmd(None, None))
        .await
        .unwrap();
}

#[tokio::test]
async fn edit_with_identifiers_upserts_then_fetches() {
    let p = make_process(1);
    let pc = p.clone();
    let urn = p_urn(1);
    let fetched_id = make_identifier(urn.clone(), "newKey", "newVal");
    let fidc = fetched_id.clone();
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_put_transfer_process()
        .returning(move |_, _| Ok(pc.clone()));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_upsert_identifier()
        .times(1)
        .returning(|_, _| {
            Ok(TransferProcessIdentifier::new(
                p_urn(0),
                "placeholder",
                Some("placeholder".to_string()),
            ))
        });
    id_repo
        .expect_get_identifiers_by_process_id()
        .returning(move |_| Ok(vec![fidc.clone()]));

    let mut ids = HashMap::new();
    ids.insert("newKey".to_string(), "newVal".to_string());
    let svc = make_svc(proc_repo, id_repo);
    let view = svc
        .edit(&admin_scope(), &urn, &make_edit_cmd(None, Some(ids)))
        .await
        .unwrap();

    assert_eq!(
        view.correlation
            .identifiers
            .get("newKey")
            .map(String::as_str),
        Some("newVal")
    );
}

// The re-fetch after upsert returns the full current state, so pre-existing
// identifiers that were NOT part of this edit command also appear in the view.
#[tokio::test]
async fn edit_view_identifiers_come_from_repo_after_upsert() {
    let p = make_process(1);
    let pc = p.clone();
    let urn = p_urn(1);
    // repo returns two identifiers (pre-existing + new)
    let existing = make_identifier(urn.clone(), "existingKey", "existingVal");
    let upserted = make_identifier(urn.clone(), "newKey", "newVal");
    let (ec, uc) = (existing.clone(), upserted.clone());
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_put_transfer_process()
        .returning(move |_, _| Ok(pc.clone()));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_upsert_identifier()
        .times(1)
        .returning(|_, _| {
            Ok(TransferProcessIdentifier::new(
                p_urn(0),
                "placeholder",
                Some("placeholder".to_string()),
            ))
        });
    id_repo
        .expect_get_identifiers_by_process_id()
        .returning(move |_| Ok(vec![ec.clone(), uc.clone()]));

    let mut ids = HashMap::new();
    ids.insert("newKey".to_string(), "newVal".to_string());
    let svc = make_svc(proc_repo, id_repo);
    let view = svc
        .edit(&admin_scope(), &urn, &make_edit_cmd(None, Some(ids)))
        .await
        .unwrap();

    assert!(view.correlation.identifiers.contains_key("existingKey"));
    assert!(view.correlation.identifiers.contains_key("newKey"));
}

#[tokio::test]
async fn edit_propagates_process_repo_error() {
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo.expect_put_transfer_process().returning(|_, _| {
        Err(TransferProcessRepoErrors::ErrorUpdatingTransferProcess(io_err()).into_errors())
    });
    let id_repo = MockTransferIdentifierRepoTrait::new();

    let svc = make_svc(proc_repo, id_repo);
    assert!(
        svc.edit(&admin_scope(), &p_urn(1), &make_edit_cmd(None, None))
            .await
            .is_err()
    );
}

#[tokio::test]
async fn edit_propagates_identifier_upsert_error() {
    let pc = make_process(1);
    let pcc = pc.clone();
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_put_transfer_process()
        .returning(move |_, _| Ok(pcc.clone()));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo.expect_upsert_identifier().returning(|_, _| {
        Err(TransferIdentifierRepoErrors::ErrorUpsertingTransferIdentifier(io_err()).into_errors())
    });

    let mut ids = HashMap::new();
    ids.insert("k".to_string(), "v".to_string());
    let svc = make_svc(proc_repo, id_repo);
    assert!(
        svc.edit(&admin_scope(), &p_urn(1), &make_edit_cmd(None, Some(ids)))
            .await
            .is_err()
    );
}

// This covers the second identifier call (get_identifiers_by_process_id),
// which happens after put and any upserts. Even a nil-identifier edit reaches it.
#[tokio::test]
async fn edit_propagates_identifier_fetch_error() {
    let pc = make_process(1);
    let pcc = pc.clone();
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_put_transfer_process()
        .returning(move |_, _| Ok(pcc.clone()));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_process_id()
        .returning(|_| {
            Err(
                TransferIdentifierRepoErrors::ErrorFetchingTransferIdentifier(io_err())
                    .into_errors(),
            )
        });

    let svc = make_svc(proc_repo, id_repo);
    assert!(
        svc.edit(&admin_scope(), &p_urn(1), &make_edit_cmd(None, None))
            .await
            .is_err()
    );
}

// ─── delete ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn delete_happy_path() {
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_delete_transfer_process()
        .times(1)
        .returning(|_| Ok(()));
    let id_repo = MockTransferIdentifierRepoTrait::new();

    let svc = make_svc(proc_repo, id_repo);
    assert!(svc.delete(&admin_scope(), &p_urn(1)).await.is_ok());
}

#[tokio::test]
async fn delete_propagates_error() {
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo.expect_delete_transfer_process().returning(|_| {
        Err(TransferProcessRepoErrors::ErrorDeletingTransferProcess(io_err()).into_errors())
    });
    let id_repo = MockTransferIdentifierRepoTrait::new();

    let svc = make_svc(proc_repo, id_repo);
    assert!(svc.delete(&admin_scope(), &p_urn(1)).await.is_err());
}
