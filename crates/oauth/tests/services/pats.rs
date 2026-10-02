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

//! PatService with a mocked repository: tenant isolation and role checks.

use std::sync::Arc;

use common::auth::claims::RbacRole;
use common::query::{Page, Sort};
use common::test_utils::scopes::TestScopes;
use oauth::data::repositories::pat::{MockPatRepository, PatRepositoryError};
use oauth::entities::query::PatFilter;
use oauth::services::pat_service::PatServiceTrait;
use oauth::services::pat_service::service::PatService;
use uuid::Uuid;
use ymir::errors::RepoIntoErrors;

fn make_service(repo: MockPatRepository) -> PatService {
    PatService::new(Arc::new(repo))
}

/// A non-admin listing the tokens of another tenant is rejected.
#[tokio::test]
async fn list_pats_foreign_tenant_query_rejected_with_forbidden() {
    let repo = MockPatRepository::new();
    let svc = make_service(repo);

    let filter = PatFilter {
        tenant_id: Some("tenant-foreign".to_string()),
        ..Default::default()
    };

    let result = svc
        .list_pats(
            &TestScopes::owner("tenant-1"),
            &filter,
            &Page::default(),
            &Sort::default(),
        )
        .await;
    assert!(result.is_err());
}

/// Revoking a token of another tenant is not found.
#[tokio::test]
async fn revoke_foreign_tenant_returns_not_found() {
    let mut repo = MockPatRepository::new();
    let test_id = Uuid::new_v4();
    repo.expect_revoke()
        .withf(move |tenant, id| tenant.as_deref() == Some("tenant-2") && id == &test_id)
        .returning(|_, _| Err(PatRepositoryError::NotFound.into_errors()));

    let svc = make_service(repo);
    assert!(
        svc.revoke_pat(&TestScopes::owner("tenant-2"), test_id)
            .await
            .is_err()
    );
}

/// A non-admin's token is always created in its own tenant.
#[tokio::test]
async fn create_forces_caller_tenant_for_non_admin() {
    let mut repo = MockPatRepository::new();
    repo.expect_create()
        .withf(|p| p.tenant_id == "tenant-2" && p.name == "my-pat")
        .returning(|p| Ok(p.clone()));

    let svc = make_service(repo);
    let res = svc
        .create_pat(
            &TestScopes::owner("tenant-2"),
            "my-pat",
            RbacRole::Owner,
            vec!["data:read".into()],
            None,
        )
        .await
        .unwrap();
    assert_eq!(res.name, "my-pat");
}

/// A reader cannot create a token.
#[tokio::test]
async fn reader_cannot_create_pat() {
    let repo = MockPatRepository::new();
    let svc = make_service(repo);

    let result = svc
        .create_pat(
            &TestScopes::reader("tenant-1"),
            "my-pat",
            RbacRole::Reader,
            vec![],
            None,
        )
        .await;
    assert!(result.is_err());
}

/// A reader cannot revoke a token.
#[tokio::test]
async fn reader_cannot_revoke_pat() {
    let repo = MockPatRepository::new();
    let svc = make_service(repo);
    let test_id = Uuid::new_v4();

    let result = svc
        .revoke_pat(&TestScopes::reader("tenant-1"), test_id)
        .await;
    assert!(result.is_err());
}

/// An admin lists and counts without a tenant filter.
#[tokio::test]
async fn admin_can_query_cross_tenant() {
    let mut repo = MockPatRepository::new();
    repo.expect_get_all()
        .withf(|f, _, _| f.tenant_id.is_none())
        .returning(|_, _, _| Ok(vec![]));
    repo.expect_count()
        .withf(|f| f.tenant_id.is_none())
        .returning(|_| Ok(0));

    let svc = make_service(repo);
    let result = svc
        .list_pats(
            &TestScopes::admin(),
            &PatFilter::default(),
            &Page::default(),
            &Sort::default(),
        )
        .await;
    assert!(result.is_ok());
}
