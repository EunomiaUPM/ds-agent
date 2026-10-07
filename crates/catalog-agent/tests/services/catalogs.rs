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

//! CatalogService with a mocked repository and no-op cache: tenant isolation and role checks.

use std::sync::Arc;

use catalog_agent::data::entities::catalog;
use catalog_agent::data::factory_trait::MockCatalogAgentRepoTrait;
use catalog_agent::data::repo_traits::catalog_db_errors::{
    CatalogAgentRepoErrors, CatalogRepoErrors,
};
use catalog_agent::data::repo_traits::catalog_repo::MockCatalogRepositoryTrait;
use catalog_agent::entities::catalogs::{EditCatalogDto, NewCatalogDto};
use catalog_agent::entities::filters::CatalogFilter;
use catalog_agent::services::catalogs::service::CatalogService;
use catalog_agent::services::catalogs::CatalogServiceTrait;
use chrono::Utc;
use common::paginated_spec::{Page, Sort};
use common::oauth::OwnerScope;
use common::test_utils::scopes::TestUsers;
use ymir::errors::RepoIntoErrors;

use crate::support::fixtures::{noop_cache_factory, test_urn};

fn not_found() -> ymir::errors::Errors {
    CatalogAgentRepoErrors::CatalogRepoErrors(CatalogRepoErrors::CatalogNotFound).into_errors()
}

fn make_svc(repo: MockCatalogRepositoryTrait) -> CatalogService {
    let repo = Arc::new(repo);
    let mut factory = MockCatalogAgentRepoTrait::new();
    factory
        .expect_get_catalog_repo()
        .returning(move || repo.clone());
    CatalogService::new(Arc::new(factory), noop_cache_factory())
}

fn make_model(tenant: &str, n: u32) -> catalog::Model {
    catalog::Model {
        id: test_urn(n).to_string(),
        user_id: tenant.to_string(),
        user_role: common::oauth::RolePath::root(),
        visibility: common::oauth::Visibility::Private,
        foaf_home_page: None,
        dct_conforms_to: None,
        dct_creator: None,
        dct_identifier: None,
        dct_issued: Utc::now().into(),
        dct_modified: None,
        dct_title: Some(format!("catalog-{n}")),
        dspace_participant_id: None,
        dspace_main_catalog: false,
    }
}

fn make_new_dto() -> NewCatalogDto {
    NewCatalogDto {
        dct_title: Some("new catalog".to_string()),
        ..Default::default()
    }
}

fn make_edit_dto() -> EditCatalogDto {
    EditCatalogDto {
        foaf_home_page: None,
        dct_conforms_to: None,
        dct_creator: None,
        dct_title: Some("edited".to_string()),
    }
}

/// Reading a record of another tenant is not found: the lookup only searches the
/// caller's tenant.
#[tokio::test]
async fn get_one_foreign_tenant_returns_not_found() {
    let mut repo = MockCatalogRepositoryTrait::new();
    repo.expect_get_catalog_by_id()
        .withf(|scope, id| *scope == OwnerScope::seeing(&TestUsers::alone("tenant-2")) && id == &test_urn(1))
        .returning(|_, _| Ok(None));

    let svc = make_svc(repo);
    let result = svc
        .get_catalog_by_id(&TestUsers::user("tenant-2", "/admin/tenant-2"), &test_urn(1))
        .await;
    assert!(result.is_err());
}

/// Reading a catalog of the caller's own tenant returns it.
#[tokio::test]
async fn get_one_own_tenant_returns_dto() {
    let mut repo = MockCatalogRepositoryTrait::new();
    repo.expect_get_catalog_by_id()
        .withf(|scope, id| *scope == OwnerScope::seeing(&TestUsers::alone("tenant-1")) && id == &test_urn(1))
        .returning(|_, _| Ok(Some(make_model("tenant-1", 1))));

    let svc = make_svc(repo);
    let dto = svc
        .get_catalog_by_id(&TestUsers::user("tenant-1", "/admin/tenant-1"), &test_urn(1))
        .await
        .unwrap();
    assert_eq!(dto.inner.user_id, "tenant-1");
}

/// Narrowing a listing to another user stays within what the caller sees.
#[tokio::test]
async fn get_all_of_another_user_stays_within_what_the_caller_sees() {
    let mut repo = MockCatalogRepositoryTrait::new();
    repo.expect_get_all_catalogs()
        .withf(|scope, f, _, _| {
            *scope == OwnerScope::seeing(&TestUsers::alone("tenant-1"))
                && f.user_id.as_deref() == Some("tenant-foreign")
        })
        .returning(|_, _, _, _| Ok((vec![], Some(0))));
    let svc = make_svc(repo);
    let filter = CatalogFilter {
        user_id: Some("tenant-foreign".to_string()),
        ..Default::default()
    };

    let result = svc
        .get_all_catalogs(
            &TestUsers::user("tenant-1", "/admin/tenant-1"),
            &filter,
            &Page::default(),
            &Sort::default(),
        )
        .await;
    assert!(result.unwrap().items.is_empty());
}

/// A non-admin listing without filter only sees its own tenant.
#[tokio::test]
async fn get_all_owner_forces_own_tenant_filter() {
    let mut repo = MockCatalogRepositoryTrait::new();
    repo.expect_get_all_catalogs()
        .withf(|scope, f, _, _| {
            *scope == OwnerScope::seeing(&TestUsers::alone("tenant-1")) && f.user_id.is_none()
        })
        .returning(|_, _, _, _| Ok((vec![make_model("tenant-1", 1)], Some(1))));

    let svc = make_svc(repo);
    let page = svc
        .get_all_catalogs(
            &TestUsers::user("tenant-1", "/admin/tenant-1"),
            &CatalogFilter::default(),
            &Page::default(),
            &Sort::default(),
        )
        .await
        .unwrap();
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.total, Some(1));
}

/// An admin without a tenant filter lists every tenant.
#[tokio::test]
async fn get_all_admin_without_tenant_queries_cross_tenant() {
    let mut repo = MockCatalogRepositoryTrait::new();
    repo.expect_get_all_catalogs()
        .withf(|scope, _, _, _| *scope == OwnerScope::All)
        .returning(|_, _, _, _| {
            Ok((
                vec![make_model("tenant-1", 1), make_model("tenant-2", 2)],
                Some(2),
            ))
        });

    let svc = make_svc(repo);
    let page = svc
        .get_all_catalogs(
            &TestUsers::user("admin-tenant", "/admin"),
            &CatalogFilter::default(),
            &Page::default(),
            &Sort::default(),
        )
        .await
        .unwrap();
    assert_eq!(page.items.len(), 2);
}

/// Editing a record of another tenant is not found and changes nothing.
#[tokio::test]
async fn edit_foreign_tenant_returns_not_found_without_mutating() {
    let mut repo = MockCatalogRepositoryTrait::new();
    repo.expect_put_catalog_by_id()
        .withf(|scope, id, _| *scope == OwnerScope::acting(&TestUsers::alone("tenant-2")) && id == &test_urn(1))
        .returning(|_, _, _| Err(not_found()));

    let svc = make_svc(repo);
    let result = svc
        .put_catalog_by_id(
            &TestUsers::user("tenant-2", "/admin/tenant-2"),
            &test_urn(1),
            &make_edit_dto(),
        )
        .await;
    assert!(result.is_err());
}

/// Deleting a record of another tenant is not found.
#[tokio::test]
async fn delete_foreign_tenant_returns_not_found() {
    let mut repo = MockCatalogRepositoryTrait::new();
    repo.expect_delete_catalog_by_id()
        .withf(|scope, id| *scope == OwnerScope::acting(&TestUsers::alone("tenant-2")) && id == &test_urn(1))
        .returning(|_, _| Err(not_found()));

    let svc = make_svc(repo);
    let result = svc
        .delete_catalog_by_id(&TestUsers::user("tenant-2", "/admin/tenant-2"), &test_urn(1))
        .await;
    assert!(result.is_err());
}

/// A batch read only returns records of the caller's tenant.
#[tokio::test]
async fn batch_filters_out_foreign_tenant_records() {
    let mut repo = MockCatalogRepositoryTrait::new();
    repo.expect_get_batch_catalogs()
        .withf(|scope, ids| *scope == OwnerScope::seeing(&TestUsers::alone("tenant-2")) && ids == [test_urn(1)])
        .returning(|_, _| Ok(vec![]));

    let svc = make_svc(repo);
    let views = svc
        .get_batch_catalogs(&TestUsers::user("tenant-2", "/admin/tenant-2"), &[test_urn(1)])
        .await
        .unwrap();
    assert!(views.is_empty());
}

/// A non-admin always creates in its own tenant, whatever the DTO says.
#[tokio::test]
async fn create_forces_caller_tenant_for_non_admin() {
    let mut repo = MockCatalogRepositoryTrait::new();
    repo.expect_create_catalog()
        .withf(|cmd| cmd.owner == TestUsers::owner("tenant-2"))
        .returning(|cmd| Ok(make_model(&cmd.owner.user_id, 1)));

    let svc = make_svc(repo);
    let mut cmd = make_new_dto();
    cmd.owner = Some(TestUsers::owner("tenant-1"));
    let dto = svc
        .create_catalog(&TestUsers::user("tenant-2", "/admin/tenant-2"), &cmd)
        .await
        .unwrap();
    assert_eq!(dto.inner.user_id, "tenant-2");
}

/// An admin creates in the tenant named by the DTO.
#[tokio::test]
async fn create_admin_respects_requested_tenant() {
    let mut repo = MockCatalogRepositoryTrait::new();
    repo.expect_create_catalog()
        .withf(|cmd| cmd.owner == TestUsers::owner("tenant-9"))
        .returning(|cmd| Ok(make_model(&cmd.owner.user_id, 1)));

    let svc = make_svc(repo);
    let mut cmd = make_new_dto();
    cmd.owner = Some(TestUsers::owner("tenant-9"));
    let dto = svc
        .create_catalog(&TestUsers::user("admin-tenant", "/admin"), &cmd)
        .await
        .unwrap();
    assert_eq!(dto.inner.user_id, "tenant-9");
}
