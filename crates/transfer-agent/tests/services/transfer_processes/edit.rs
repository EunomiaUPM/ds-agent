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

//! Editing a process: state changes, identifier upserts and the read back.

use super::*;

/// An edit changes the state.
#[tokio::test]
async fn edit_with_state_change() {
    let p = make_process(1);
    let pc = p.clone();
    let urn = p_urn(1);
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_put_transfer_process()
        .withf(|_, _, cmd| cmd.state.as_ref().map(|s| s.0.as_str()) == Some("COMPLETED"))
        .returning(move |_, _, _| Ok(pc.clone()));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_process_id()
        .returning(|_| Ok(vec![]));

    let svc = make_svc(proc_repo, id_repo);
    let view = svc
        .edit(
            &TestScopes::admin_of("tenant-1"),
            &urn,
            &make_edit_cmd(Some("COMPLETED"), None),
        )
        .await
        .unwrap();

    assert_eq!(&view.id, p.id());
}

/// An edit without identifiers stores none.
#[tokio::test]
async fn edit_without_identifiers_skips_upsert() {
    let p = make_process(1);
    let pc = p.clone();
    let urn = p_urn(1);
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_put_transfer_process()
        .returning(move |_, _, _| Ok(pc.clone()));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo.expect_upsert_identifier().times(0);
    id_repo
        .expect_get_identifiers_by_process_id()
        .returning(|_| Ok(vec![]));

    let svc = make_svc(proc_repo, id_repo);
    svc.edit(
        &TestScopes::admin_of("tenant-1"),
        &urn,
        &make_edit_cmd(None, None),
    )
    .await
    .unwrap();
}

/// An edit stores its identifiers and then reads them back.
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
        .returning(move |_, _, _| Ok(pc.clone()));
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
        .edit(
            &TestScopes::admin_of("tenant-1"),
            &urn,
            &make_edit_cmd(None, Some(ids)),
        )
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

/// The edited view shows every identifier of the process, not only the edited ones.
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
        .returning(move |_, _, _| Ok(pc.clone()));
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
        .edit(
            &TestScopes::admin_of("tenant-1"),
            &urn,
            &make_edit_cmd(None, Some(ids)),
        )
        .await
        .unwrap();

    assert!(view.correlation.identifiers.contains_key("existingKey"));
    assert!(view.correlation.identifiers.contains_key("newKey"));
}

/// A process repository error while editing is returned.
#[tokio::test]
async fn edit_propagates_process_repo_error() {
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_put_transfer_process()
        .returning(|_, _, _| {
            Err(TransferProcessRepoErrors::ErrorUpdatingTransferProcess(io_err()).into_errors())
        });
    let id_repo = MockTransferIdentifierRepoTrait::new();

    let svc = make_svc(proc_repo, id_repo);
    assert!(
        svc.edit(
            &TestScopes::admin_of("tenant-1"),
            &p_urn(1),
            &make_edit_cmd(None, None)
        )
        .await
        .is_err()
    );
}

/// An error storing an identifier on edit is returned.
#[tokio::test]
async fn edit_propagates_identifier_upsert_error() {
    let pc = make_process(1);
    let pcc = pc.clone();
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_put_transfer_process()
        .returning(move |_, _, _| Ok(pcc.clone()));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo.expect_upsert_identifier().returning(|_, _| {
        Err(TransferIdentifierRepoErrors::ErrorUpsertingTransferIdentifier(io_err()).into_errors())
    });

    let mut ids = HashMap::new();
    ids.insert("k".to_string(), "v".to_string());
    let svc = make_svc(proc_repo, id_repo);
    assert!(
        svc.edit(
            &TestScopes::admin_of("tenant-1"),
            &p_urn(1),
            &make_edit_cmd(None, Some(ids))
        )
        .await
        .is_err()
    );
}

/// An error reading identifiers back after an edit is returned, even with no
/// identifiers in the edit.
#[tokio::test]
async fn edit_propagates_identifier_fetch_error() {
    let pc = make_process(1);
    let pcc = pc.clone();
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_put_transfer_process()
        .returning(move |_, _, _| Ok(pcc.clone()));
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
        svc.edit(
            &TestScopes::admin_of("tenant-1"),
            &p_urn(1),
            &make_edit_cmd(None, None)
        )
        .await
        .is_err()
    );
}
