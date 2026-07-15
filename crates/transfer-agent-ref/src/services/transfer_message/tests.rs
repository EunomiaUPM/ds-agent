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

use std::str::FromStr;
use std::sync::Arc;

use base64::Engine;
use chrono::{Duration, Utc};
use compact_str::CompactString;
use urn::Urn;

use crate::data::repo::transfer_message::{
    MockTransferMessageRepoTrait, TransferMessageRepoErrors,
};
use crate::entities::commands::NewTransferMessageCommand;
use crate::entities::filters::TransferMessageFilter;
use crate::entities::ids::{MessageId, TransferProcessId};
use crate::entities::message_envelope::MessageEnvelope;
use crate::entities::protocol::{ProtocolId, ProtocolMessageType, ProtocolState};
use crate::entities::transfer_message::{Direction, TransferMessage};
use crate::services::transfer_message::TransferMessageServiceTrait;
use crate::services::transfer_message::service::TransferMessageService;
use common::auth::access::AccessScope;
use common::auth::claims::RbacRole;
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

// Unit tests for TransferMessageService. The single repository dependency
// (TransferMessageRepoTrait) is replaced with a mockall mock.

// Fixtures ────────────────────────────────────────────────────────────────

fn p_urn(n: u32) -> Urn {
    Urn::from_str(&format!("urn:uuid:{:08x}-0000-0000-0000-000000000000", n)).expect("static URN")
}

fn make_envelope() -> MessageEnvelope {
    MessageEnvelope {
        canonical_form: None,
        canonical_hash: None,
        payload: serde_json::json!({}),
    }
}

fn make_message(n: u32) -> TransferMessage {
    TransferMessage {
        id: MessageId::new(p_urn(n + 1000)),
        transfer_process_id: TransferProcessId::new(p_urn(1)),
        tenant_id: "tenant-1".to_string(),
        direction: Direction::Inbound,
        protocol: ProtocolId::Dsp2024,
        message_type: ProtocolMessageType(CompactString::from("TransferRequestMessage")),
        state_transition_from: "INITIAL".to_string(),
        state_transition_to: "STARTED".to_string(),
        envelope: make_envelope(),
        occurred_at: Utc::now() - Duration::seconds(n as i64 * 10),
    }
}

fn empty_filter() -> TransferMessageFilter {
    TransferMessageFilter {
        tenant_id: None,
        direction: None,
        protocol: None,
        state_transition_to: None,
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

fn make_cmd() -> NewTransferMessageCommand {
    NewTransferMessageCommand {
        id: None,
        transfer_process_id: TransferProcessId::new(p_urn(1)),
        tenant_id: Some("tenant-1".to_string()),
        direction: Direction::Inbound,
        protocol: ProtocolId::Dsp2024,
        message_type: ProtocolMessageType(CompactString::from("TransferRequestMessage")),
        state_transition_from: ProtocolState(CompactString::from("INITIAL")),
        state_transition_to: ProtocolState(CompactString::from("STARTED")),
        envelope: make_envelope(),
    }
}

fn make_svc(repo: MockTransferMessageRepoTrait) -> TransferMessageService {
    TransferMessageService::new(Arc::new(repo))
}

fn io_err() -> Box<dyn std::error::Error + Send + Sync> {
    Box::new(std::io::Error::from(std::io::ErrorKind::Other))
}

// get_all ─────────────────────────────────────────────────────────────────
// Like the process service, get_all fires get_all_transfer_messages and
// count_transfer_messages concurrently via tokio::try_join!. The cursor for
// messages is always based on occurred_at, regardless of the sort field.

#[tokio::test]
async fn get_all_empty_result() {
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_all_transfer_messages()
        .returning(|_, _, _| Ok(vec![]));
    repo.expect_count_transfer_messages().returning(|_| Ok(0));

    let svc = make_svc(repo);
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

// Cursor is always base64url(occurred_at.to_rfc3339()) for messages,
// independent of which sort field was requested.
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

    assert_eq!(result.items.len(), 1);
    assert_eq!(result.next_cursor.as_deref(), Some(expected.as_str()));
}

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

#[tokio::test]
async fn get_all_total_forwarded_from_count() {
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_all_transfer_messages()
        .returning(|_, _, _| Ok(vec![]));
    repo.expect_count_transfer_messages().returning(|_| Ok(77));

    let svc = make_svc(repo);
    let result = svc
        .get_all(
            &admin_scope(),
            &empty_filter(),
            &default_page(),
            &Sort::CreatedAtDesc,
        )
        .await
        .unwrap();

    assert_eq!(result.total, Some(77));
}

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
        &admin_scope(),
        &filter,
        &default_page(),
        &Sort::CreatedAtDesc,
    )
    .await
    .unwrap();
}

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
        &admin_scope(),
        &filter,
        &default_page(),
        &Sort::CreatedAtDesc,
    )
    .await
    .unwrap();
}

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
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_all_transfer_messages()
        .withf(|f, _, _| f.tenant_id.as_ref().map(|t| t.as_str()) == Some("t1"))
        .returning(|_, _, _| Ok(vec![]));
    repo.expect_count_transfer_messages().returning(|_| Ok(0));

    let filter = TransferMessageFilter {
        tenant_id: Some("t1".to_string()),
        ..empty_filter()
    };
    let svc = make_svc(repo);
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
        &admin_scope(),
        &filter,
        &default_page(),
        &Sort::CreatedAtDesc,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn get_all_sort_created_at_asc_passed_through() {
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_all_transfer_messages()
        .withf(|_, _, s| matches!(s, Sort::CreatedAtAsc))
        .returning(|_, _, _| Ok(vec![]));
    repo.expect_count_transfer_messages().returning(|_| Ok(0));

    let svc = make_svc(repo);
    svc.get_all(
        &admin_scope(),
        &empty_filter(),
        &default_page(),
        &Sort::CreatedAtAsc,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn get_all_sort_updated_at_desc_passed_through() {
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_all_transfer_messages()
        .withf(|_, _, s| matches!(s, Sort::UpdatedAtDesc))
        .returning(|_, _, _| Ok(vec![]));
    repo.expect_count_transfer_messages().returning(|_| Ok(0));

    let svc = make_svc(repo);
    svc.get_all(
        &admin_scope(),
        &empty_filter(),
        &default_page(),
        &Sort::UpdatedAtDesc,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn get_all_page_limit_and_cursor_passed_through() {
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_all_transfer_messages()
        .withf(|_, p, _| p.limit == 5 && p.cursor.as_deref() == Some("abc"))
        .returning(|_, _, _| Ok(vec![]));
    repo.expect_count_transfer_messages().returning(|_| Ok(0));

    let svc = make_svc(repo);
    svc.get_all(
        &admin_scope(),
        &empty_filter(),
        &Page {
            limit: 5,
            cursor: Some("abc".to_string()),
        },
        &Sort::CreatedAtDesc,
    )
    .await
    .unwrap();
}

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
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_all_transfer_messages()
        .returning(|_, _, _| Ok(vec![]));
    repo.expect_count_transfer_messages().returning(|_| {
        Err(TransferMessageRepoErrors::ErrorFetchingTransferMessage(io_err()).into_errors())
    });

    let svc = make_svc(repo);
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

// get_all_by_process ──────────────────────────────────────────────────────
// get_all_by_process delegates to get_messages_by_process_id for the items, but
// still calls count_transfer_messages (without process_id) for the total. The
// cursor logic and filter pass-through are identical to get_all.

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
            &admin_scope(),
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
            &admin_scope(),
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
            &admin_scope(),
            &process_urn,
            &empty_filter(),
            &Page {
                limit: 1,
                cursor: None,
            },
            &Sort::CreatedAtDesc,
        )
        .await
        .unwrap();

    assert_eq!(result.next_cursor.as_deref(), Some(expected.as_str()));
}

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
            &admin_scope(),
            &process_urn,
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
        &admin_scope(),
        &process_urn,
        &empty_filter(),
        &default_page(),
        &Sort::CreatedAtDesc,
    )
    .await
    .unwrap();
}

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
        &admin_scope(),
        &process_urn,
        &filter,
        &default_page(),
        &Sort::CreatedAtDesc,
    )
    .await
    .unwrap();
}

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
            &admin_scope(),
            &process_urn,
            &empty_filter(),
            &default_page(),
            &Sort::CreatedAtDesc
        )
        .await
        .is_err()
    );
}

// get_one ─────────────────────────────────────────────────────────────────
// get_one converts Option::None from the repo into a not-found error using
// TransferMessageRepoErrors::TransferMessageNotFound.

#[tokio::test]
async fn get_one_happy_path() {
    let msg = make_message(1);
    let id_urn = msg.id.as_urn().clone();
    let mc = msg.clone();
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_transfer_message_by_id()
        .returning(move |_| Ok(Some(mc.clone())));

    let svc = make_svc(repo);
    let view = svc.get_one(&admin_scope(), &id_urn).await.unwrap();

    assert_eq!(&view.id, &msg.id);
    assert_eq!(view.direction, msg.direction);
    assert_eq!(view.protocol, msg.protocol);
}

#[tokio::test]
async fn get_one_returns_view_fields_correctly() {
    let msg = make_message(5);
    let id_urn = msg.id.as_urn().clone();
    let mc = msg.clone();
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_transfer_message_by_id()
        .returning(move |_| Ok(Some(mc.clone())));

    let svc = make_svc(repo);
    let view = svc.get_one(&admin_scope(), &id_urn).await.unwrap();

    assert_eq!(view.tenant_id, "tenant-1");
    assert_eq!(view.state_transition_from, "INITIAL");
    assert_eq!(view.state_transition_to, "STARTED");
}

#[tokio::test]
async fn get_one_not_found_returns_error() {
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_transfer_message_by_id()
        .returning(|_| Ok(None));

    let svc = make_svc(repo);
    assert!(svc.get_one(&admin_scope(), &p_urn(999)).await.is_err());
}

#[tokio::test]
async fn get_one_propagates_repo_error() {
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_transfer_message_by_id().returning(|_| {
        Err(TransferMessageRepoErrors::ErrorFetchingTransferMessage(io_err()).into_errors())
    });

    let svc = make_svc(repo);
    assert!(svc.get_one(&admin_scope(), &p_urn(1)).await.is_err());
}

// tenant isolation (AccessScope) ───────────────────────────────────────────

#[tokio::test]
async fn get_one_foreign_tenant_returns_not_found() {
    let msg = make_message(1); // tenant-1
    let id_urn = msg.id.as_urn().clone();
    let mc = msg.clone();
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_transfer_message_by_id()
        .returning(move |_| Ok(Some(mc.clone())));

    let svc = make_svc(repo);
    assert!(
        svc.get_one(&tenant_scope("tenant-2"), &id_urn)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn create_forces_caller_tenant_for_non_admin() {
    let msg = make_message(1);
    let mc = msg.clone();
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_create_transfer_message()
        .withf(|cmd| cmd.tenant_id.as_ref().map(|t| t.as_str()) == Some("tenant-2"))
        .returning(move |_| Ok(mc.clone()));

    let svc = make_svc(repo);
    // make_cmd sets tenant_id = tenant-1; the scope must override it.
    svc.create(&tenant_scope("tenant-2"), &make_cmd())
        .await
        .unwrap();
}

#[tokio::test]
async fn delete_foreign_tenant_returns_not_found() {
    let msg = make_message(1); // tenant-1
    let id_urn = msg.id.as_urn().clone();
    let mc = msg.clone();
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_transfer_message_by_id()
        .returning(move |_| Ok(Some(mc.clone())));
    repo.expect_delete_transfer_message().times(0);

    let svc = make_svc(repo);
    assert!(
        svc.delete(&tenant_scope("tenant-2"), &id_urn)
            .await
            .is_err()
    );
}

// create ──────────────────────────────────────────────────────────────────
// create delegates entirely to the repo and assembles the view from whatever
// the repo returns — not from the command fields.

#[tokio::test]
async fn create_happy_path() {
    let msg = make_message(1);
    let mc = msg.clone();
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_create_transfer_message()
        .times(1)
        .returning(move |_| Ok(mc.clone()));

    let svc = make_svc(repo);
    let view = svc.create(&admin_scope(), &make_cmd()).await.unwrap();

    assert_eq!(&view.id, &msg.id);
    assert_eq!(view.direction, Direction::Inbound);
    assert_eq!(view.protocol, ProtocolId::Dsp2024);
}

// The view is assembled from what the repo returns, not from cmd directly.
// Here we diverge cmd and the returned message to confirm the repo wins.
#[tokio::test]
async fn create_view_fields_assembled_from_stored_message() {
    let mut msg = make_message(1);
    msg.direction = Direction::Outbound;
    msg.protocol = ProtocolId::Dsp2025_1;
    let mc = msg.clone();
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_create_transfer_message()
        .returning(move |_| Ok(mc.clone()));

    let mut cmd = make_cmd();
    cmd.direction = Direction::Outbound;
    cmd.protocol = ProtocolId::Dsp2025_1;
    let svc = make_svc(repo);
    let view = svc.create(&admin_scope(), &cmd).await.unwrap();

    assert_eq!(view.direction, Direction::Outbound);
    assert_eq!(view.protocol, ProtocolId::Dsp2025_1);
}

#[tokio::test]
async fn create_propagates_repo_error() {
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_create_transfer_message().returning(|_| {
        Err(TransferMessageRepoErrors::ErrorCreatingTransferMessage(io_err()).into_errors())
    });

    let svc = make_svc(repo);
    assert!(svc.create(&admin_scope(), &make_cmd()).await.is_err());
}

// delete ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn delete_happy_path() {
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_delete_transfer_message()
        .times(1)
        .returning(|_| Ok(()));

    let svc = make_svc(repo);
    assert!(svc.delete(&admin_scope(), &p_urn(1)).await.is_ok());
}

#[tokio::test]
async fn delete_propagates_error() {
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_delete_transfer_message().returning(|_| {
        Err(TransferMessageRepoErrors::ErrorDeletingTransferMessage(io_err()).into_errors())
    });

    let svc = make_svc(repo);
    assert!(svc.delete(&admin_scope(), &p_urn(1)).await.is_err());
}
