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

//! Unit and isolation tests for PatService with mocked repository.

use std::sync::Arc;

use common::auth::access::AccessScope;
use common::auth::claims::RbacRole;
use common::query::{Page, Sort};
use oauth::data::repositories::pat::{MockPatRepository, PatRepositoryError};
use oauth::entities::query::PatFilter;
use oauth::services::pat_service::PatServiceTrait;
use oauth::services::pat_service::service::PatService;
use uuid::Uuid;
use ymir::errors::RepoIntoErrors;

fn admin_scope() -> AccessScope {
    AccessScope::from_role(RbacRole::Admin, "admin-tenant")
}

fn tenant_scope(tenant: &str) -> AccessScope {
    AccessScope::from_role(RbacRole::Owner, tenant)
}

fn reader_scope(tenant: &str) -> AccessScope {
    AccessScope::from_role(RbacRole::Reader, tenant)
}

fn make_service(repo: MockPatRepository) -> PatService {
    PatService::new(Arc::new(repo))
}

#[tokio::test]
async fn list_pats_foreign_tenant_query_rejected_with_forbidden() {
    let repo = MockPatRepository::new();
    let svc = make_service(repo);

    let mut filter = PatFilter::default();
    filter.tenant_id = Some("tenant-foreign".to_string());

    let result = svc
        .list_pats(
            &tenant_scope("tenant-1"),
            &filter,
            &Page::default(),
            &Sort::default(),
        )
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn revoke_foreign_tenant_returns_not_found() {
    let mut repo = MockPatRepository::new();
    let test_id = Uuid::new_v4();
    repo.expect_revoke()
        .withf(move |tenant, id| tenant.as_deref() == Some("tenant-2") && id == &test_id)
        .returning(|_, _| Err(PatRepositoryError::NotFound.into_errors()));

    let svc = make_service(repo);
    assert!(
        svc.revoke_pat(&tenant_scope("tenant-2"), test_id)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn create_forces_caller_tenant_for_non_admin() {
    let mut repo = MockPatRepository::new();
    repo.expect_create()
        .withf(|p| p.tenant_id == "tenant-2" && p.name == "my-pat")
        .returning(|p| Ok(p.clone()));

    let svc = make_service(repo);
    let res = svc
        .create_pat(
            &tenant_scope("tenant-2"),
            "my-pat",
            RbacRole::Owner,
            vec!["data:read".into()],
            None,
        )
        .await
        .unwrap();
    assert_eq!(res.name, "my-pat");
}

#[tokio::test]
async fn reader_cannot_create_pat() {
    let repo = MockPatRepository::new();
    let svc = make_service(repo);

    let result = svc
        .create_pat(
            &reader_scope("tenant-1"),
            "my-pat",
            RbacRole::Reader,
            vec![],
            None,
        )
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn reader_cannot_revoke_pat() {
    let repo = MockPatRepository::new();
    let svc = make_service(repo);
    let test_id = Uuid::new_v4();

    let result = svc.revoke_pat(&reader_scope("tenant-1"), test_id).await;
    assert!(result.is_err());
}

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
            &admin_scope(),
            &PatFilter::default(),
            &Page::default(),
            &Sort::default(),
        )
        .await;
    assert!(result.is_ok());
}
