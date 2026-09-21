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

//! Unit and isolation tests for UserService with mocked repository.

use std::sync::Arc;

use common::auth::access::AccessScope;
use common::auth::claims::RbacRole;
use common::query::{Page, Sort};
use oauth::data::repositories::user::MockUserRepository;
use oauth::entities::commands::PatchUserCommand;
use oauth::entities::query::UserFilter;
use oauth::services::user_service::UserServiceTrait;
use oauth::services::user_service::service::UserService;

fn admin_scope() -> AccessScope {
    AccessScope::from_role(RbacRole::Admin, "admin-tenant")
}

fn tenant_scope(tenant: &str) -> AccessScope {
    AccessScope::from_role(RbacRole::Owner, tenant)
}

fn reader_scope(tenant: &str) -> AccessScope {
    AccessScope::from_role(RbacRole::Reader, tenant)
}

fn make_service(repo: MockUserRepository) -> UserService {
    UserService::new(Arc::new(repo))
}

#[tokio::test]
async fn get_user_foreign_tenant_rejected_with_forbidden() {
    let repo = MockUserRepository::new();
    let svc = make_service(repo);

    let result = svc.get_user(&tenant_scope("tenant-1"), "tenant-2").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn get_user_own_tenant_returns_not_found_when_missing() {
    let mut repo = MockUserRepository::new();
    repo.expect_get_by_tenant_id()
        .withf(|t| t == "tenant-1")
        .returning(|_| Ok(None));

    let svc = make_service(repo);
    let result = svc.get_user(&tenant_scope("tenant-1"), "tenant-1").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn list_users_foreign_tenant_query_rejected_with_forbidden() {
    let repo = MockUserRepository::new();
    let svc = make_service(repo);

    let mut filter = UserFilter::default();
    filter.tenant_id = Some("tenant-foreign".to_string());

    let result = svc
        .list_users(
            &tenant_scope("tenant-1"),
            &filter,
            &Page::default(),
            &Sort::default(),
        )
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn patch_foreign_tenant_rejected_with_forbidden() {
    let repo = MockUserRepository::new();
    let svc = make_service(repo);

    let cmd = PatchUserCommand {
        email: Some("new@example.com".into()),
        role: None,
        extra_fields: None,
    };

    let result = svc
        .patch_user(&tenant_scope("tenant-1"), "tenant-2", &cmd)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn delete_non_admin_rejected_with_forbidden() {
    let repo = MockUserRepository::new();
    let svc = make_service(repo);

    let result = svc.delete_user(&tenant_scope("tenant-1"), "tenant-1").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn admin_can_delete_user() {
    let mut repo = MockUserRepository::new();
    repo.expect_delete()
        .withf(|t| t == "tenant-1")
        .returning(|_| Ok(()));

    let svc = make_service(repo);
    let result = svc.delete_user(&admin_scope(), "tenant-1").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn admin_can_query_cross_tenant() {
    let mut repo = MockUserRepository::new();
    repo.expect_get_all()
        .withf(|f, _, _| f.tenant_id.is_none())
        .returning(|_, _, _| Ok(vec![]));
    repo.expect_count()
        .withf(|f| f.tenant_id.is_none())
        .returning(|_| Ok(0));

    let svc = make_service(repo);
    let result = svc
        .list_users(
            &admin_scope(),
            &UserFilter::default(),
            &Page::default(),
            &Sort::default(),
        )
        .await;
    assert!(result.is_ok());
}
