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

//! Reading one process with its identifiers.

use super::*;

/// A stored process is returned with its identifiers.
#[tokio::test]
async fn get_one_returns_view_with_identifiers() {
    let p = make_process(1);
    let id = make_identifier(p_urn(1), "key", "value");
    let (pc, idc) = (p.clone(), id.clone());
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_transfer_process_by_id()
        .returning(move |_, _| Ok(Some(pc.clone())));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_process_id()
        .returning(move |_| Ok(vec![idc.clone()]));

    let svc = make_svc(proc_repo, id_repo);
    let view = svc
        .get_one(&TestScopes::admin_of("tenant-1"), p.id().as_urn())
        .await
        .unwrap();

    assert_eq!(&view.id, p.id());
    assert_eq!(
        view.correlation.identifiers.get("key").map(String::as_str),
        Some("value")
    );
}

/// consumerPid and providerPid identifiers become top-level fields of the view.
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
        .returning(move |_, _| Ok(Some(pc.clone())));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_process_id()
        .returning(move |_| Ok(idsc.clone()));

    let svc = make_svc(proc_repo, id_repo);
    let view = svc
        .get_one(&TestScopes::admin_of("tenant-1"), p.id().as_urn())
        .await
        .unwrap();

    assert_eq!(view.correlation.consumer_pid.as_deref(), Some("cpid-abc"));
    assert_eq!(view.correlation.provider_pid.as_deref(), Some("ppid-xyz"));
}

/// A process without identifiers has an empty identifier map.
#[tokio::test]
async fn get_one_without_identifiers_returns_empty_map() {
    let p = make_process(1);
    let pc = p.clone();
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_transfer_process_by_id()
        .returning(move |_, _| Ok(Some(pc.clone())));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_process_id()
        .returning(|_| Ok(vec![]));

    let svc = make_svc(proc_repo, id_repo);
    let view = svc
        .get_one(&TestScopes::admin_of("tenant-1"), p.id().as_urn())
        .await
        .unwrap();

    assert!(view.correlation.identifiers.is_empty());
    assert!(view.correlation.consumer_pid.is_none());
    assert!(view.correlation.provider_pid.is_none());
}

/// A missing process is an error.
#[tokio::test]
async fn get_one_not_found_returns_error() {
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_transfer_process_by_id()
        .returning(|_, _| Ok(None));
    let id_repo = MockTransferIdentifierRepoTrait::new();

    let svc = make_svc(proc_repo, id_repo);
    assert!(
        svc.get_one(&TestScopes::admin_of("tenant-1"), &p_urn(999))
            .await
            .is_err()
    );
}

/// A process repository error while reading is returned.
#[tokio::test]
async fn get_one_propagates_process_repo_error() {
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_transfer_process_by_id()
        .returning(|_, _| {
            Err(TransferProcessRepoErrors::ErrorFetchingTransferProcess(io_err()).into_errors())
        });
    let id_repo = MockTransferIdentifierRepoTrait::new();

    let svc = make_svc(proc_repo, id_repo);
    assert!(
        svc.get_one(&TestScopes::admin_of("tenant-1"), &p_urn(1))
            .await
            .is_err()
    );
}

/// An identifier repository error while reading is returned.
#[tokio::test]
async fn get_one_propagates_identifier_repo_error() {
    let p = make_process(1);
    let pc = p.clone();
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_transfer_process_by_id()
        .returning(move |_, _| Ok(Some(pc.clone())));
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
        svc.get_one(&TestScopes::admin_of("tenant-1"), p.id().as_urn())
            .await
            .is_err()
    );
}
