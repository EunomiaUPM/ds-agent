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

//! Creating a process with its identifiers, and deleting it.

use super::*;

/// Creating without identifiers stores none.
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
        .create(&TestUsers::user("tenant-1", "/admin"), &make_new_cmd(None))
        .await
        .unwrap();

    assert!(&view.correlation.identifiers.is_empty());
}

/// Creating with one identifier stores it once.
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
        .create(&TestUsers::user("tenant-1", "/admin"), &make_new_cmd(Some(ids)))
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

/// Creating with several identifiers stores each of them.
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
        .create(
            &TestUsers::user("tenant-1", "/admin"),
            &make_new_cmd(Some(ids.clone())),
        )
        .await
        .unwrap();

    for (k, v) in &ids {
        assert_eq!(
            view.correlation.identifiers.get(k).map(String::as_str),
            Some(v.as_str())
        );
    }
}

/// A process repository error while creating is returned.
#[tokio::test]
async fn create_propagates_process_repo_error() {
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo.expect_create_transfer_process().returning(|_| {
        Err(TransferProcessRepoErrors::ErrorCreatingTransferProcess(io_err()).into_errors())
    });
    let id_repo = MockTransferIdentifierRepoTrait::new();

    let svc = make_svc(proc_repo, id_repo);
    assert!(
        svc.create(&TestUsers::user("tenant-1", "/admin"), &make_new_cmd(None))
            .await
            .is_err()
    );
}

/// An error storing an identifier on create is returned.
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
        svc.create(&TestUsers::user("tenant-1", "/admin"), &make_new_cmd(Some(ids)))
            .await
            .is_err()
    );
}

/// Deleting a stored process succeeds.
#[tokio::test]
async fn delete_happy_path() {
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_delete_transfer_process()
        .times(1)
        .returning(|_, _| Ok(common::test_utils::scopes::TestUsers::owner("tenant-1")));
    let id_repo = MockTransferIdentifierRepoTrait::new();

    let svc = make_svc(proc_repo, id_repo);
    assert!(
        svc.delete(&TestUsers::user("tenant-1", "/admin"), &p_urn(1))
            .await
            .is_ok()
    );
}

/// A repository error while deleting is returned.
#[tokio::test]
async fn delete_propagates_error() {
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_delete_transfer_process()
        .returning(|_, _| {
            Err(TransferProcessRepoErrors::ErrorDeletingTransferProcess(io_err()).into_errors())
        });
    let id_repo = MockTransferIdentifierRepoTrait::new();

    let svc = make_svc(proc_repo, id_repo);
    assert!(
        svc.delete(&TestUsers::user("tenant-1", "/admin"), &p_urn(1))
            .await
            .is_err()
    );
}
