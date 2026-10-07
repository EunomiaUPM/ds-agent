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

//! Reading, creating and deleting a single message.

use super::*;

/// A stored message is returned as a view.
#[tokio::test]
async fn get_one_happy_path() {
    let msg = make_message(1);
    let id_urn = msg.id.as_urn().clone();
    let mc = msg.clone();
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_transfer_message_by_id()
        .returning(move |_, _| Ok(Some(mc.clone())));

    let svc = make_svc(repo);
    let view = svc
        .get_one(&TestUsers::user("tenant-1", "/admin"), &id_urn)
        .await
        .unwrap();

    assert_eq!(&view.id, &msg.id);
    assert_eq!(view.direction, msg.direction);
    assert_eq!(view.protocol, msg.protocol);
}

/// The view carries every field of the stored message.
#[tokio::test]
async fn get_one_returns_view_fields_correctly() {
    let msg = make_message(5);
    let id_urn = msg.id.as_urn().clone();
    let mc = msg.clone();
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_transfer_message_by_id()
        .returning(move |_, _| Ok(Some(mc.clone())));

    let svc = make_svc(repo);
    let view = svc
        .get_one(&TestUsers::user("tenant-1", "/admin"), &id_urn)
        .await
        .unwrap();

    assert_eq!(view.user_id, "tenant-1");
    assert_eq!(view.state_transition_from, "INITIAL");
    assert_eq!(view.state_transition_to, "STARTED");
}

/// A missing message is an error.
#[tokio::test]
async fn get_one_not_found_returns_error() {
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_transfer_message_by_id()
        .returning(|_, _| Ok(None));

    let svc = make_svc(repo);
    assert!(
        svc.get_one(&TestUsers::user("tenant-1", "/admin"), &p_urn(999))
            .await
            .is_err()
    );
}

/// A repository error while reading is returned.
#[tokio::test]
async fn get_one_propagates_repo_error() {
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_transfer_message_by_id().returning(|_, _| {
        Err(TransferMessageRepoErrors::ErrorFetchingTransferMessage(io_err()).into_errors())
    });

    let svc = make_svc(repo);
    assert!(
        svc.get_one(&TestUsers::user("tenant-1", "/admin"), &p_urn(1))
            .await
            .is_err()
    );
}

/// A created message is returned as a view.
#[tokio::test]
async fn create_happy_path() {
    let msg = make_message(1);
    let mc = msg.clone();
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_create_transfer_message()
        .times(1)
        .returning(move |_| Ok(mc.clone()));

    let svc = make_svc(repo);
    let view = svc
        .create(&TestUsers::user("tenant-1", "/admin"), &make_cmd())
        .await
        .unwrap();

    assert_eq!(&view.id, &msg.id);
    assert_eq!(view.direction, Direction::Inbound);
    assert_eq!(view.protocol, ProtocolId::Dsp2024);
}

/// The view comes from what the repository stored, not from the command.
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
    let view = svc
        .create(&TestUsers::user("tenant-1", "/admin"), &cmd)
        .await
        .unwrap();

    assert_eq!(view.direction, Direction::Outbound);
    assert_eq!(view.protocol, ProtocolId::Dsp2025_1);
}

/// A repository error while creating is returned.
#[tokio::test]
async fn create_propagates_repo_error() {
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_create_transfer_message().returning(|_| {
        Err(TransferMessageRepoErrors::ErrorCreatingTransferMessage(io_err()).into_errors())
    });

    let svc = make_svc(repo);
    assert!(
        svc.create(&TestUsers::user("tenant-1", "/admin"), &make_cmd())
            .await
            .is_err()
    );
}

/// Deleting a stored message succeeds.
#[tokio::test]
async fn delete_happy_path() {
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_delete_transfer_message()
        .times(1)
        .returning(|_, _| Ok(common::test_utils::scopes::TestUsers::owner("tenant-1")));

    let svc = make_svc(repo);
    assert!(
        svc.delete(&TestUsers::user("tenant-1", "/admin"), &p_urn(1))
            .await
            .is_ok()
    );
}

/// A repository error while deleting is returned.
#[tokio::test]
async fn delete_propagates_error() {
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_delete_transfer_message().returning(|_, _| {
        Err(TransferMessageRepoErrors::ErrorDeletingTransferMessage(io_err()).into_errors())
    });

    let svc = make_svc(repo);
    assert!(
        svc.delete(&TestUsers::user("tenant-1", "/admin"), &p_urn(1))
            .await
            .is_err()
    );
}
