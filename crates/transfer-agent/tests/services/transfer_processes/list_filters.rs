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

//! Listing processes: filters and cursor passed through, and repository errors.

use super::*;

/// The protocol filter reaches the repository.
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
        &TestScopes::admin_of("tenant-1"),
        &filter,
        &default_page(),
        &Sort::CreatedAtDesc,
    )
    .await
    .unwrap();
}

/// The role filter reaches the repository.
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
        &TestScopes::admin_of("tenant-1"),
        &filter,
        &default_page(),
        &Sort::CreatedAtDesc,
    )
    .await
    .unwrap();
}

/// The state filter reaches the repository.
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
        &TestScopes::admin_of("tenant-1"),
        &filter,
        &default_page(),
        &Sort::CreatedAtDesc,
    )
    .await
    .unwrap();
}

/// The tenant filter reaches the repository.
#[tokio::test]
async fn get_all_filter_by_tenant_id_passed_through() {
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_all_transfer_processes()
        .withf(|f, _, _| f.tenant_id.as_deref() == Some("acme"))
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
        &TestScopes::admin_of("tenant-1"),
        &filter,
        &default_page(),
        &Sort::CreatedAtDesc,
    )
    .await
    .unwrap();
}

/// The date range reaches the repository.
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
        &TestScopes::admin_of("tenant-1"),
        &filter,
        &default_page(),
        &Sort::CreatedAtDesc,
    )
    .await
    .unwrap();
}

/// The page cursor reaches the repository.
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
        &TestScopes::admin_of("tenant-1"),
        &empty_filter(),
        &Page::new(10, Some("tok".to_string())),
        &Sort::CreatedAtDesc,
    )
    .await
    .unwrap();
}

/// A process repository error while listing is returned.
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
            &TestScopes::admin_of("tenant-1"),
            &empty_filter(),
            &default_page(),
            &Sort::CreatedAtDesc
        )
        .await
        .is_err()
    );
}

/// A repository error while counting is returned.
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
            &TestScopes::admin_of("tenant-1"),
            &empty_filter(),
            &default_page(),
            &Sort::CreatedAtDesc
        )
        .await
        .is_err()
    );
}

/// An identifier repository error while listing is returned.
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
            &TestScopes::admin_of("tenant-1"),
            &empty_filter(),
            &default_page(),
            &Sort::CreatedAtDesc
        )
        .await
        .is_err()
    );
}
