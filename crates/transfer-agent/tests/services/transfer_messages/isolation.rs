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
    let msg = make_message(1); // tenant-1
    let id_urn = msg.id.as_urn().clone();
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_get_transfer_message_by_id()
        .withf(|tenant, id| tenant.as_deref() == Some("tenant-2") && id == &p_urn(1001))
        .returning(|_, _| Ok(None));

    let svc = make_svc(repo);
    assert!(
        svc.get_one(&TestScopes::owner("tenant-2"), &id_urn)
            .await
            .is_err()
    );
}

/// A non-admin listing another tenant is rejected before touching the repository.
#[tokio::test]
async fn get_all_foreign_tenant_query_rejected_with_forbidden() {
    let repo = MockTransferMessageRepoTrait::new();
    let svc = make_svc(repo);

    let filter = TransferMessageFilter {
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

/// A non-admin listing another tenant's process messages is rejected.
#[tokio::test]
async fn get_all_by_process_foreign_tenant_query_rejected_with_forbidden() {
    let repo = MockTransferMessageRepoTrait::new();
    let svc = make_svc(repo);

    let filter = TransferMessageFilter {
        tenant_id: Some("tenant-foreign".to_string()),
        ..Default::default()
    };

    let result = svc
        .get_all_by_process(
            &TestScopes::owner("tenant-1"),
            &p_urn(1),
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
    let msg = make_message(1);
    let mc = msg.clone();
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_create_transfer_message()
        .withf(|cmd| cmd.tenant_id.as_deref() == Some("tenant-2"))
        .returning(move |_| Ok(mc.clone()));

    let svc = make_svc(repo);
    // make_cmd sets tenant_id = tenant-1; the scope must override it.
    svc.create(&TestScopes::owner("tenant-2"), &make_cmd())
        .await
        .unwrap();
}

/// Deleting a record of another tenant is not found.
#[tokio::test]
async fn delete_foreign_tenant_returns_not_found() {
    let msg = make_message(1); // tenant-1
    let id_urn = msg.id.as_urn().clone();
    let mut repo = MockTransferMessageRepoTrait::new();
    repo.expect_delete_transfer_message()
        .withf(|tenant, id| tenant.as_deref() == Some("tenant-2") && id == &p_urn(1001))
        .returning(|_, _| Err(TransferMessageRepoErrors::TransferMessageNotFound.into_errors()));

    let svc = make_svc(repo);
    assert!(
        svc.delete(&TestScopes::owner("tenant-2"), &id_urn)
            .await
            .is_err()
    );
}
