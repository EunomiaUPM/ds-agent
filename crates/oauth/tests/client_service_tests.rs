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

//! Unit and isolation tests for ClientService with mocked repository.

use std::sync::Arc;

use common::auth::access::AccessScope;
use common::auth::claims::RbacRole;
use common::query::{Page, Sort};
use oauth::data::repositories::client::{ClientRepositoryError, MockClientRepository};
use oauth::entities::client::Client;
use oauth::entities::commands::CreateClientCommand;
use oauth::entities::query::ClientFilter;
use oauth::services::client_service::ClientServiceTrait;
use oauth::services::client_service::service::ClientService;
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

fn make_service(repo: MockClientRepository) -> ClientService {
    ClientService::new(Arc::new(repo))
}

fn make_create_cmd(tenant_id: Option<String>) -> CreateClientCommand {
    CreateClientCommand {
        client_id: "test-client".to_string(),
        tenant_id,
        client_secret: "secret-123456".to_string(),
        client_name: "Test Client".to_string(),
        role: RbacRole::Owner,
        scopes: vec!["data:read".to_string()],
    }
}

#[tokio::test]
async fn get_one_foreign_tenant_returns_not_found() {
    let mut repo = MockClientRepository::new();
    repo.expect_get_by_id()
        .withf(|tenant, id| tenant == "tenant-2" && id == "client-1")
        .returning(|_, _| Ok(None));

    let svc = make_service(repo);
    assert!(
        svc.get_client(&tenant_scope("tenant-2"), "client-1")
            .await
            .is_err()
    );
}

#[tokio::test]
async fn get_all_foreign_tenant_query_rejected_with_forbidden() {
    let repo = MockClientRepository::new();
    let svc = make_service(repo);

    let mut filter = ClientFilter::default();
    filter.tenant_id = Some("tenant-foreign".to_string());

    let result = svc
        .list_clients(
            &tenant_scope("tenant-1"),
            &filter,
            &Page::default(),
            &Sort::default(),
        )
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn delete_foreign_tenant_returns_not_found() {
    let mut repo = MockClientRepository::new();
    repo.expect_delete()
        .withf(|tenant, id| tenant == "tenant-2" && id == "client-1")
        .returning(|_, _| Err(ClientRepositoryError::NotFound.into_errors()));

    let svc = make_service(repo);
    assert!(
        svc.delete_client(&tenant_scope("tenant-2"), "client-1")
            .await
            .is_err()
    );
}

#[tokio::test]
async fn create_forces_caller_tenant_for_non_admin() {
    let mut repo = MockClientRepository::new();
    repo.expect_get_by_client_id()
        .withf(|id| id == "test-client")
        .returning(|_| Ok(None));
    repo.expect_create()
        .withf(|c| c.tenant_id == "tenant-2")
        .returning(|c| Ok(c.clone()));

    let svc = make_service(repo);
    let cmd = make_create_cmd(Some("tenant-1".to_string()));
    let view = svc
        .create_client(&tenant_scope("tenant-2"), &cmd)
        .await
        .unwrap();
    assert_eq!(view.client_id, "test-client");
}

#[tokio::test]
async fn reader_cannot_create_client() {
    let repo = MockClientRepository::new();
    let svc = make_service(repo);
    let cmd = make_create_cmd(None);

    let result = svc.create_client(&reader_scope("tenant-1"), &cmd).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn reader_cannot_delete_client() {
    let repo = MockClientRepository::new();
    let svc = make_service(repo);

    let result = svc
        .delete_client(&reader_scope("tenant-1"), "client-1")
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn admin_can_query_cross_tenant() {
    let mut repo = MockClientRepository::new();
    repo.expect_get_all()
        .withf(|f, _, _| f.tenant_id.is_none())
        .returning(|_, _, _| Ok(vec![]));
    repo.expect_count()
        .withf(|f| f.tenant_id.is_none())
        .returning(|_| Ok(0));

    let svc = make_service(repo);
    let result = svc
        .list_clients(
            &admin_scope(),
            &ClientFilter::default(),
            &Page::default(),
            &Sort::default(),
        )
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn admin_can_create_client_for_any_tenant() {
    let mut repo = MockClientRepository::new();
    repo.expect_get_by_client_id()
        .withf(|id| id == "test-client")
        .returning(|_| Ok(None));
    repo.expect_create()
        .withf(|c| c.tenant_id == "tenant-custom")
        .returning(|c| Ok(c.clone()));

    let svc = make_service(repo);
    let cmd = make_create_cmd(Some("tenant-custom".to_string()));
    let view = svc.create_client(&admin_scope(), &cmd).await.unwrap();
    assert_eq!(view.client_id, "test-client");
}
