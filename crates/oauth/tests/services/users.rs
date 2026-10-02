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

//! UserService with a mocked repository: a user is its tenant, so every check is per tenant.

use std::sync::Arc;

use common::query::{Page, Sort};
use common::test_utils::scopes::TestScopes;
use oauth::data::repositories::user::MockUserRepository;
use oauth::entities::commands::PatchUserCommand;
use oauth::entities::query::UserFilter;
use oauth::services::user_service::UserServiceTrait;
use oauth::services::user_service::service::UserService;

fn make_service(repo: MockUserRepository) -> UserService {
    UserService::new(Arc::new(repo))
}

/// A non-admin reading another tenant's user is rejected.
#[tokio::test]
async fn get_user_foreign_tenant_rejected_with_forbidden() {
    let repo = MockUserRepository::new();
    let svc = make_service(repo);

    let result = svc
        .get_user(&TestScopes::owner("tenant-1"), "tenant-2")
        .await;
    assert!(result.is_err());
}

/// Reading its own user that does not exist is not found.
#[tokio::test]
async fn get_user_own_tenant_returns_not_found_when_missing() {
    let mut repo = MockUserRepository::new();
    repo.expect_get_by_tenant_id()
        .withf(|t| t == "tenant-1")
        .returning(|_| Ok(None));

    let svc = make_service(repo);
    let result = svc
        .get_user(&TestScopes::owner("tenant-1"), "tenant-1")
        .await;
    assert!(result.is_err());
}

/// A non-admin listing users of another tenant is rejected.
#[tokio::test]
async fn list_users_foreign_tenant_query_rejected_with_forbidden() {
    let repo = MockUserRepository::new();
    let svc = make_service(repo);

    let filter = UserFilter {
        tenant_id: Some("tenant-foreign".to_string()),
        ..Default::default()
    };

    let result = svc
        .list_users(
            &TestScopes::owner("tenant-1"),
            &filter,
            &Page::default(),
            &Sort::default(),
        )
        .await;
    assert!(result.is_err());
}

/// A non-admin patching another tenant's user is rejected.
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
        .patch_user(&TestScopes::owner("tenant-1"), "tenant-2", &cmd)
        .await;
    assert!(result.is_err());
}

/// Only an admin deletes users, not even its own.
#[tokio::test]
async fn delete_non_admin_rejected_with_forbidden() {
    let repo = MockUserRepository::new();
    let svc = make_service(repo);

    let result = svc
        .delete_user(&TestScopes::owner("tenant-1"), "tenant-1")
        .await;
    assert!(result.is_err());
}

/// An admin deletes any user.
#[tokio::test]
async fn admin_can_delete_user() {
    let mut repo = MockUserRepository::new();
    repo.expect_delete()
        .withf(|t| t == "tenant-1")
        .returning(|_| Ok(()));

    let svc = make_service(repo);
    let result = svc.delete_user(&TestScopes::admin(), "tenant-1").await;
    assert!(result.is_ok());
}

/// An admin lists and counts without a tenant filter.
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
            &TestScopes::admin(),
            &UserFilter::default(),
            &Page::default(),
            &Sort::default(),
        )
        .await;
    assert!(result.is_ok());
}
