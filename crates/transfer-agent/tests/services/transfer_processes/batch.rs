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

//! Batch reads: one view per process, each with its own identifiers.

use super::*;

/// A batch without ids returns nothing.
#[tokio::test]
async fn batch_empty_ids_returns_empty_vec() {
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_batch_transfer_processes()
        .returning(|_, _| Ok(vec![]));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_batch_process_id()
        .returning(|_| Ok(vec![]));

    let svc = make_svc(proc_repo, id_repo);
    let views = svc
        .batch(
            &TestScopes::admin_of("tenant-1"),
            &BatchRequests { ids: vec![] },
        )
        .await
        .unwrap();

    assert!(views.is_empty());
}

/// A batch returns one view per process found.
#[tokio::test]
async fn batch_returns_one_view_per_process() {
    let p1 = make_process(1);
    let p2 = make_process(2);
    let (p1c, p2c) = (p1.clone(), p2.clone());
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_batch_transfer_processes()
        .returning(move |_, _| Ok(vec![p1c.clone(), p2c.clone()]));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_batch_process_id()
        .returning(|_| Ok(vec![]));

    let svc = make_svc(proc_repo, id_repo);
    let views = svc
        .batch(
            &TestScopes::admin_of("tenant-1"),
            &BatchRequests {
                ids: vec![p_urn(1), p_urn(2)],
            },
        )
        .await
        .unwrap();

    assert_eq!(views.len(), 2);
}

/// Identifiers fetched in one query are grouped so each view gets only its own.
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
        .returning(move |_, _| Ok(vec![p1c.clone(), p2c.clone()]));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_batch_process_id()
        .returning(move |_| Ok(vec![id1c.clone(), id2c.clone()]));

    let svc = make_svc(proc_repo, id_repo);
    let views = svc
        .batch(
            &TestScopes::admin_of("tenant-1"),
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

/// A process repository error in a batch is returned.
#[tokio::test]
async fn batch_propagates_process_repo_error() {
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_batch_transfer_processes()
        .returning(|_, _| {
            Err(TransferProcessRepoErrors::ErrorFetchingTransferProcess(io_err()).into_errors())
        });
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_batch_process_id()
        .returning(|_| Ok(vec![]));

    let svc = make_svc(proc_repo, id_repo);
    assert!(
        svc.batch(
            &TestScopes::admin_of("tenant-1"),
            &BatchRequests {
                ids: vec![p_urn(1)]
            }
        )
        .await
        .is_err()
    );
}

/// An identifier repository error in a batch is returned.
#[tokio::test]
async fn batch_propagates_identifier_repo_error() {
    let pc = make_process(1);
    let pcc = pc.clone();
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_batch_transfer_processes()
        .returning(move |_, _| Ok(vec![pcc.clone()]));
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
            &TestScopes::admin_of("tenant-1"),
            &BatchRequests {
                ids: vec![p_urn(1)]
            }
        )
        .await
        .is_err()
    );
}
