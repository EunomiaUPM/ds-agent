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

//! Tenant isolation and role checks.

use super::*;

/// Reading a record of another tenant is not found: the lookup only searches the
/// caller's tenant.
#[tokio::test]
async fn get_one_foreign_tenant_returns_not_found() {
    // Process belongs to tenant-1; an owner scoped to tenant-2 passes tenant-2 to repo,
    // which filters it out in the query and returns None, yielding a 404.
    let p = make_process(1); // tenant-1
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_transfer_process_by_id()
        .withf(|tenant, id| tenant.as_deref() == Some("tenant-2") && id == &p_urn(1))
        .returning(|_, _| Ok(None));
    let id_repo = MockTransferIdentifierRepoTrait::new();

    let svc = make_svc(proc_repo, id_repo);
    assert!(
        svc.get_one(&TestScopes::owner("tenant-2"), p.id().as_urn())
            .await
            .is_err()
    );
}

/// A non-admin listing another tenant is rejected before touching the repository.
#[tokio::test]
async fn get_all_foreign_tenant_query_rejected_with_forbidden() {
    let proc_repo = MockTransferProcessRepoTrait::new();
    let id_repo = MockTransferIdentifierRepoTrait::new();
    let svc = make_svc(proc_repo, id_repo);

    let filter = TransferProcessFilter {
        tenant_id: Some("tenant-foreign".to_string()),
        ..Default::default()
    };

    let result = svc
        .get_all(
            &TestScopes::owner("tenant-1"),
            &filter,
            &Page::default(),
            &Sort::default(),
        )
        .await;

    assert!(result.is_err());
}

/// A non-admin always creates in its own tenant, whatever the command says.
#[tokio::test]
async fn create_forces_caller_tenant_for_non_admin() {
    // Even though the body asks for tenant-1, an owner scoped to tenant-2 must
    // have the record created under tenant-2.
    let pc = make_process(1);
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_create_transfer_process()
        .withf(|cmd| cmd.tenant_id.as_deref() == Some("tenant-2"))
        .returning(move |_| Ok(pc.clone()));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo.expect_upsert_identifier().times(0);

    let svc = make_svc(proc_repo, id_repo);
    // make_new_cmd sets tenant_id = tenant-1; the scope must override it.
    svc.create(&TestScopes::owner("tenant-2"), &make_new_cmd(None))
        .await
        .unwrap();
}

/// Editing a record of another tenant is not found and changes nothing.
#[tokio::test]
async fn edit_foreign_tenant_returns_not_found_without_mutating() {
    // Foreign tenant filter is passed directly to repo, which returns TransferProcessNotFound.
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_put_transfer_process()
        .withf(|tenant, id, _| tenant.as_deref() == Some("tenant-2") && id == &p_urn(1))
        .returning(|_, _, _| Err(TransferProcessRepoErrors::TransferProcessNotFound.into_errors()));
    let id_repo = MockTransferIdentifierRepoTrait::new();

    let svc = make_svc(proc_repo, id_repo);
    assert!(
        svc.edit(
            &TestScopes::owner("tenant-2"),
            &p_urn(1),
            &make_edit_cmd(None, None)
        )
        .await
        .is_err()
    );
}

/// Deleting a record of another tenant is not found.
#[tokio::test]
async fn delete_foreign_tenant_returns_not_found() {
    // Foreign tenant filter is passed directly to repo, which returns TransferProcessNotFound.
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_delete_transfer_process()
        .withf(|tenant, id| tenant.as_deref() == Some("tenant-2") && id == &p_urn(1))
        .returning(|_, _| Err(TransferProcessRepoErrors::TransferProcessNotFound.into_errors()));
    let id_repo = MockTransferIdentifierRepoTrait::new();

    let svc = make_svc(proc_repo, id_repo);
    assert!(
        svc.delete(&TestScopes::owner("tenant-2"), &p_urn(1))
            .await
            .is_err()
    );
}

/// A batch read only returns records of the caller's tenant.
#[tokio::test]
async fn batch_filters_out_foreign_tenant_records() {
    // Foreign tenant filter is passed to the repo, returning empty list.
    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_batch_transfer_processes()
        .withf(|tenant, ids| tenant.as_deref() == Some("tenant-2") && ids == [p_urn(1)])
        .returning(|_, _| Ok(vec![]));
    let mut id_repo = MockTransferIdentifierRepoTrait::new();
    id_repo
        .expect_get_identifiers_by_batch_process_id()
        .returning(|_| Ok(vec![]));

    let svc = make_svc(proc_repo, id_repo);
    let views = svc
        .batch(
            &TestScopes::owner("tenant-2"),
            &BatchRequests {
                ids: vec![p_urn(1)],
            },
        )
        .await
        .unwrap();
    assert!(views.is_empty());
}
