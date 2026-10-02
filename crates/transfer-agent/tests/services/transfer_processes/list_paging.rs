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

//! Listing processes: pages, cursors per sort, totals and identifier merging.

use super::*;

/// An empty repository gives an empty page without cursor.
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
            &TestScopes::admin_of("tenant-1"),
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

/// A page that fills the limit gets the last item's timestamp as next cursor; a short
/// one gets none.
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
            &TestScopes::admin_of("tenant-1"),
            &empty_filter(),
            &Page::new(1, None),
            &Sort::CreatedAtDesc,
        )
        .await
        .unwrap();

    let expected =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(process.created_at().to_rfc3339());
    assert_eq!(result.items.len(), 1);
    assert_eq!(result.next_cursor.as_deref(), Some(expected.as_str()));
}

/// A page shorter than the limit has no next cursor.
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
            &TestScopes::admin_of("tenant-1"),
            &empty_filter(),
            &Page::new(5, None),
            &Sort::CreatedAtDesc,
        )
        .await
        .unwrap();

    assert!(result.next_cursor.is_none());
}

/// Sorting by updated_at descending builds the cursor from updated_at.
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
            &TestScopes::admin_of("tenant-1"),
            &empty_filter(),
            &Page::new(1, None),
            &Sort::UpdatedAtDesc,
        )
        .await
        .unwrap();

    assert_eq!(result.next_cursor, Some(expected));
}

/// Sorting by created_at builds the cursor from created_at.
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
            &TestScopes::admin_of("tenant-1"),
            &empty_filter(),
            &Page::new(1, None),
            &Sort::CreatedAtAsc,
        )
        .await
        .unwrap();

    assert_eq!(result.next_cursor, Some(expected));
}

/// Identifiers are merged into each view, and consumerPid and providerPid are
/// promoted when the process lacks them.
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
            &TestScopes::admin_of("tenant-1"),
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

/// The page total is the repository count.
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
            &TestScopes::admin_of("tenant-1"),
            &empty_filter(),
            &default_page(),
            &Sort::CreatedAtDesc,
        )
        .await
        .unwrap();

    assert_eq!(result.total, Some(42));
}
