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

//! Listing messages: paging, cursors, filters and sorts passed through, and repository errors.

use super::*;

/// An empty repository gives an empty page without cursor.
#[tokio::test]
async fn get_all_empty_result() {
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_all_transfer_messages()
        .returning(|_, _, _| Ok(vec![]));
    repo.expect_count_transfer_messages().returning(|_| Ok(0));

    let svc = make_svc(repo);
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

/// A full page gets a cursor built from the last message's occurred_at, whatever the
/// sort.
#[tokio::test]
async fn get_all_full_page_produces_cursor_from_occurred_at() {
    let msg = make_message(1);
    let expected =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(msg.occurred_at().to_rfc3339());
    let mc = msg.clone();
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_all_transfer_messages()
        .returning(move |_, _, _| Ok(vec![mc.clone()]));
    repo.expect_count_transfer_messages().returning(|_| Ok(1));

    let svc = make_svc(repo);
    let result = svc
        .get_all(
            &TestScopes::admin_of("tenant-1"),
            &empty_filter(),
            &Page::new(1, None),
            &Sort::CreatedAtDesc,
        )
        .await
        .unwrap();

    assert_eq!(result.items.len(), 1);
    assert_eq!(result.next_cursor.as_deref(), Some(expected.as_str()));
}

/// A page shorter than the limit has no next cursor.
#[tokio::test]
async fn get_all_partial_page_no_cursor() {
    let mc = make_message(1);
    let mcc = mc.clone();
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_all_transfer_messages()
        .returning(move |_, _, _| Ok(vec![mcc.clone()]));
    repo.expect_count_transfer_messages().returning(|_| Ok(1));

    let svc = make_svc(repo);
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

/// The page total is the repository count.
#[tokio::test]
async fn get_all_total_forwarded_from_count() {
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_all_transfer_messages()
        .returning(|_, _, _| Ok(vec![]));
    repo.expect_count_transfer_messages().returning(|_| Ok(77));

    let svc = make_svc(repo);
    let result = svc
        .get_all(
            &TestScopes::admin_of("tenant-1"),
            &empty_filter(),
            &default_page(),
            &Sort::CreatedAtDesc,
        )
        .await
        .unwrap();

    assert_eq!(result.total, Some(77));
}

/// The direction filter reaches the repository.
#[tokio::test]
async fn get_all_filter_by_direction_passed_through() {
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_all_transfer_messages()
        .withf(|f, _, _| f.direction == Some(Direction::Outbound))
        .returning(|_, _, _| Ok(vec![]));
    repo.expect_count_transfer_messages().returning(|_| Ok(0));

    let filter = TransferMessageFilter {
        direction: Some(Direction::Outbound),
        ..empty_filter()
    };
    let svc = make_svc(repo);
    svc.get_all(
        &TestScopes::admin_of("tenant-1"),
        &filter,
        &default_page(),
        &Sort::CreatedAtDesc,
    )
    .await
    .unwrap();
}

/// The protocol filter reaches the repository.
#[tokio::test]
async fn get_all_filter_by_protocol_passed_through() {
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_all_transfer_messages()
        .withf(|f, _, _| f.protocol == Some(ProtocolId::Dsp2025_1))
        .returning(|_, _, _| Ok(vec![]));
    repo.expect_count_transfer_messages().returning(|_| Ok(0));

    let filter = TransferMessageFilter {
        protocol: Some(ProtocolId::Dsp2025_1),
        ..empty_filter()
    };
    let svc = make_svc(repo);
    svc.get_all(
        &TestScopes::admin_of("tenant-1"),
        &filter,
        &default_page(),
        &Sort::CreatedAtDesc,
    )
    .await
    .unwrap();
}

/// The target state filter reaches the repository.
#[tokio::test]
async fn get_all_filter_by_state_transition_to_passed_through() {
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_all_transfer_messages()
        .withf(|f, _, _| f.state_transition_to.as_ref().map(|s| s.0.as_str()) == Some("COMPLETED"))
        .returning(|_, _, _| Ok(vec![]));
    repo.expect_count_transfer_messages().returning(|_| Ok(0));

    let filter = TransferMessageFilter {
        state_transition_to: Some(ProtocolState(CompactString::from("COMPLETED"))),
        ..empty_filter()
    };
    let svc = make_svc(repo);
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
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_all_transfer_messages()
        .withf(|f, _, _| f.tenant_id.as_deref() == Some("t1"))
        .returning(|_, _, _| Ok(vec![]));
    repo.expect_count_transfer_messages().returning(|_| Ok(0));

    let filter = TransferMessageFilter {
        tenant_id: Some("t1".to_string()),
        ..empty_filter()
    };
    let svc = make_svc(repo);
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
    let after = Utc::now() - Duration::days(3);
    let before = Utc::now();
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_all_transfer_messages()
        .withf(move |f, _, _| f.created_after == Some(after) && f.created_before == Some(before))
        .returning(|_, _, _| Ok(vec![]));
    repo.expect_count_transfer_messages().returning(|_| Ok(0));

    let filter = TransferMessageFilter {
        created_after: Some(after),
        created_before: Some(before),
        ..empty_filter()
    };
    let svc = make_svc(repo);
    svc.get_all(
        &TestScopes::admin_of("tenant-1"),
        &filter,
        &default_page(),
        &Sort::CreatedAtDesc,
    )
    .await
    .unwrap();
}

/// A created_at ascending sort reaches the repository.
#[tokio::test]
async fn get_all_sort_created_at_asc_passed_through() {
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_all_transfer_messages()
        .withf(|_, _, s| matches!(s, Sort::CreatedAtAsc))
        .returning(|_, _, _| Ok(vec![]));
    repo.expect_count_transfer_messages().returning(|_| Ok(0));

    let svc = make_svc(repo);
    svc.get_all(
        &TestScopes::admin_of("tenant-1"),
        &empty_filter(),
        &default_page(),
        &Sort::CreatedAtAsc,
    )
    .await
    .unwrap();
}

/// An updated_at descending sort reaches the repository.
#[tokio::test]
async fn get_all_sort_updated_at_desc_passed_through() {
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_all_transfer_messages()
        .withf(|_, _, s| matches!(s, Sort::UpdatedAtDesc))
        .returning(|_, _, _| Ok(vec![]));
    repo.expect_count_transfer_messages().returning(|_| Ok(0));

    let svc = make_svc(repo);
    svc.get_all(
        &TestScopes::admin_of("tenant-1"),
        &empty_filter(),
        &default_page(),
        &Sort::UpdatedAtDesc,
    )
    .await
    .unwrap();
}

/// The page limit and cursor reach the repository.
#[tokio::test]
async fn get_all_page_limit_and_cursor_passed_through() {
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_all_transfer_messages()
        .withf(|_, p, _| p.limit == 5 && p.cursor.as_deref() == Some("abc"))
        .returning(|_, _, _| Ok(vec![]));
    repo.expect_count_transfer_messages().returning(|_| Ok(0));

    let svc = make_svc(repo);
    svc.get_all(
        &TestScopes::admin_of("tenant-1"),
        &empty_filter(),
        &Page::new(5, Some("abc".to_string())),
        &Sort::CreatedAtDesc,
    )
    .await
    .unwrap();
}

/// A repository error while listing is returned.
#[tokio::test]
async fn get_all_propagates_message_repo_error() {
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_all_transfer_messages()
        .returning(|_, _, _| {
            Err(TransferMessageRepoErrors::ErrorFetchingTransferMessage(io_err()).into_errors())
        });
    repo.expect_count_transfer_messages().returning(|_| Ok(0));

    let svc = make_svc(repo);
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
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_all_transfer_messages()
        .returning(|_, _, _| Ok(vec![]));
    repo.expect_count_transfer_messages().returning(|_| {
        Err(TransferMessageRepoErrors::ErrorFetchingTransferMessage(io_err()).into_errors())
    });

    let svc = make_svc(repo);
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
