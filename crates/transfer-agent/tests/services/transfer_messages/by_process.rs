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

//! Listing the messages of one process.

use super::*;

/// Messages of one process are listed.
#[tokio::test]
async fn get_all_by_process_happy_path() {
    let process_urn = p_urn(1);
    let msg = make_message(1);
    let mc = msg.clone();
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_messages_by_process_id()
        .returning(move |_, _, _, _| Ok(vec![mc.clone()]));
    repo.expect_count_transfer_messages().returning(|_| Ok(1));

    let svc = make_svc(repo);
    let result = svc
        .get_all_by_process(
            &TestScopes::admin_of("tenant-1"),
            &process_urn,
            &empty_filter(),
            &default_page(),
            &Sort::CreatedAtDesc,
        )
        .await
        .unwrap();

    assert_eq!(result.items.len(), 1);
    assert_eq!(result.total, Some(1));
}

/// A process without messages gives an empty page.
#[tokio::test]
async fn get_all_by_process_empty() {
    let process_urn = p_urn(1);
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_messages_by_process_id()
        .returning(|_, _, _, _| Ok(vec![]));
    repo.expect_count_transfer_messages().returning(|_| Ok(0));

    let svc = make_svc(repo);
    let result = svc
        .get_all_by_process(
            &TestScopes::admin_of("tenant-1"),
            &process_urn,
            &empty_filter(),
            &default_page(),
            &Sort::CreatedAtDesc,
        )
        .await
        .unwrap();

    assert!(result.items.is_empty());
    assert!(result.next_cursor.is_none());
}

/// A full page of a process's messages gets a next cursor.
#[tokio::test]
async fn get_all_by_process_full_page_produces_cursor() {
    let process_urn = p_urn(1);
    let msg = make_message(1);
    let expected =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(msg.occurred_at().to_rfc3339());
    let mc = msg.clone();
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_messages_by_process_id()
        .returning(move |_, _, _, _| Ok(vec![mc.clone()]));
    repo.expect_count_transfer_messages().returning(|_| Ok(1));

    let svc = make_svc(repo);
    let result = svc
        .get_all_by_process(
            &TestScopes::admin_of("tenant-1"),
            &process_urn,
            &empty_filter(),
            &Page::new(1, None),
            &Sort::CreatedAtDesc,
        )
        .await
        .unwrap();

    assert_eq!(result.next_cursor.as_deref(), Some(expected.as_str()));
}

/// A short page of a process's messages has no next cursor.
#[tokio::test]
async fn get_all_by_process_partial_page_no_cursor() {
    let process_urn = p_urn(1);
    let mc = make_message(1);
    let mcc = mc.clone();
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_messages_by_process_id()
        .returning(move |_, _, _, _| Ok(vec![mcc.clone()]));
    repo.expect_count_transfer_messages().returning(|_| Ok(1));

    let svc = make_svc(repo);
    let result = svc
        .get_all_by_process(
            &TestScopes::admin_of("tenant-1"),
            &process_urn,
            &empty_filter(),
            &Page::new(5, None),
            &Sort::CreatedAtDesc,
        )
        .await
        .unwrap();

    assert!(result.next_cursor.is_none());
}

/// The process id reaches the repository as the filter.
#[tokio::test]
async fn get_all_by_process_process_id_passed_through() {
    let process_urn = p_urn(42);
    let puc = process_urn.clone();
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_messages_by_process_id()
        .withf(move |id, _, _, _| *id == puc)
        .returning(|_, _, _, _| Ok(vec![]));
    repo.expect_count_transfer_messages().returning(|_| Ok(0));

    let svc = make_svc(repo);
    svc.get_all_by_process(
        &TestScopes::admin_of("tenant-1"),
        &process_urn,
        &empty_filter(),
        &default_page(),
        &Sort::CreatedAtDesc,
    )
    .await
    .unwrap();
}

/// The direction filter is kept when listing by process.
#[tokio::test]
async fn get_all_by_process_filter_direction_passed_through() {
    let process_urn = p_urn(1);
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_messages_by_process_id()
        .withf(|_, f, _, _| f.direction == Some(Direction::Inbound))
        .returning(|_, _, _, _| Ok(vec![]));
    repo.expect_count_transfer_messages().returning(|_| Ok(0));

    let filter = TransferMessageFilter {
        direction: Some(Direction::Inbound),
        ..empty_filter()
    };
    let svc = make_svc(repo);
    svc.get_all_by_process(
        &TestScopes::admin_of("tenant-1"),
        &process_urn,
        &filter,
        &default_page(),
        &Sort::CreatedAtDesc,
    )
    .await
    .unwrap();
}

/// A repository error while listing by process is returned.
#[tokio::test]
async fn get_all_by_process_propagates_repo_error() {
    let process_urn = p_urn(1);
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_messages_by_process_id()
        .returning(|_, _, _, _| {
            Err(TransferMessageRepoErrors::ErrorFetchingTransferMessage(io_err()).into_errors())
        });
    repo.expect_count_transfer_messages().returning(|_| Ok(0));

    let svc = make_svc(repo);
    assert!(
        svc.get_all_by_process(
            &TestScopes::admin_of("tenant-1"),
            &process_urn,
            &empty_filter(),
            &default_page(),
            &Sort::CreatedAtDesc
        )
        .await
        .is_err()
    );
}
