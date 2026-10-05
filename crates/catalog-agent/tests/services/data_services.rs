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

//! DataServiceService with a mocked repository and no-op cache: tenant isolation and role
//! checks.

use std::sync::Arc;

use catalog_agent::data::entities::dataservice;
use catalog_agent::data::factory_trait::MockCatalogAgentRepoTrait;
use catalog_agent::data::repo_traits::catalog_db_errors::{
    CatalogAgentRepoErrors, DataServiceRepoErrors,
};
use catalog_agent::data::repo_traits::dataservice_repo::MockDataServiceRepositoryTrait;
use catalog_agent::entities::data_services::{EditDataServiceDto, NewDataServiceDto};
use catalog_agent::entities::filters::DataServiceFilter;
use catalog_agent::services::data_services::service::DataServiceService;
use catalog_agent::services::data_services::DataServiceServiceTrait;
use chrono::Utc;
use common::paginated_spec::{Page, Sort};
use common::oauth::{Owner, OwnerScope};
use common::test_utils::scopes::TestUsers;
use ymir::errors::RepoIntoErrors;

use crate::support::fixtures::{noop_cache_factory, test_urn};

fn not_found() -> ymir::errors::Errors {
    CatalogAgentRepoErrors::DataServiceRepoErrors(DataServiceRepoErrors::DataServiceNotFound)
        .into_errors()
}

fn make_svc(repo: MockDataServiceRepositoryTrait) -> DataServiceService {
    let repo = Arc::new(repo);
    let mut factory = MockCatalogAgentRepoTrait::new();
    factory
        .expect_get_dataservice_repo()
        .returning(move || repo.clone());
    DataServiceService::new(Arc::new(factory), noop_cache_factory())
}

fn make_model(tenant: &str, n: u32) -> dataservice::Model {
    dataservice::Model {
        id: test_urn(n).to_string(),
        user_id: tenant.to_string(),
        user_role: common::oauth::RolePath::root(),
        visibility: common::oauth::Visibility::Private,
        dcat_endpoint_description: None,
        dcat_endpoint_url: "https://example.org/api".to_string(),
        dct_conforms_to: None,
        dct_creator: None,
        dct_identifier: None,
        dct_issued: Utc::now().into(),
        dct_modified: None,
        dct_title: Some(format!("data-service-{n}")),
        dct_description: None,
        catalog_id: test_urn(100).to_string(),
        dspace_main_data_service: false,
    }
}

fn make_new_dto() -> NewDataServiceDto {
    NewDataServiceDto {
        dcat_endpoint_url: "https://example.org/api".to_string(),
        dct_title: Some("new data service".to_string()),
        catalog_id: test_urn(100),
        ..Default::default()
    }
}

fn make_edit_dto() -> EditDataServiceDto {
    EditDataServiceDto {
        dcat_endpoint_description: None,
        dcat_endpoint_url: None,
        dct_conforms_to: None,
        dct_creator: None,
        dct_title: Some("edited".to_string()),
        dct_description: None,
    }
}

/// Reading a record of another tenant is not found: the lookup only searches the
/// caller's tenant.
#[tokio::test]
async fn get_one_foreign_tenant_returns_not_found() {
    let mut repo = MockDataServiceRepositoryTrait::new();
    repo.expect_get_data_service_by_id()
        .withf(|scope, id| *scope == OwnerScope::seeing(&TestUsers::alone("tenant-2")) && id == &test_urn(1))
        .returning(|_, _| Ok(None));

    let svc = make_svc(repo);
    assert!(svc
        .get_data_service_by_id(&TestUsers::user("tenant-2", "/admin/tenant-2"), &test_urn(1))
        .await
        .is_err());
}

/// The main data service is the connector's, whoever asks for it.
#[tokio::test]
async fn get_main_is_the_connectors() {
    let mut repo = MockDataServiceRepositoryTrait::new();
    repo.expect_get_main_data_service()
        .returning(|| Ok(Some(make_model("tenant-1", 1))));

    let svc = make_svc(repo);
    let dto = svc
        .get_main_data_service(&TestUsers::user("tenant-1", "/admin/tenant-1"))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(dto.inner.user_id, "tenant-1");
}

/// Narrowing a listing to another user stays within what the caller sees.
#[tokio::test]
async fn get_all_of_another_user_stays_within_what_the_caller_sees() {
    let mut repo = MockDataServiceRepositoryTrait::new();
    repo.expect_get_all_data_services()
        .withf(|scope, f, _, _| {
            *scope == OwnerScope::seeing(&TestUsers::alone("tenant-1"))
                && f.user_id.as_deref() == Some("tenant-foreign")
        })
        .returning(|_, _, _, _| Ok((vec![], Some(0))));
    let svc = make_svc(repo);
    let filter = DataServiceFilter {
        user_id: Some("tenant-foreign".to_string()),
        ..Default::default()
    };
    let page = svc
        .get_all_data_services(
            &TestUsers::user("tenant-1", "/admin/tenant-1"),
            &filter,
            &Page::default(),
            &Sort::default()
        )
        .await
        .unwrap();
    assert!(page.items.is_empty());
}

/// An admin without a tenant filter lists every tenant.
#[tokio::test]
async fn get_all_admin_without_tenant_queries_cross_tenant() {
    let mut repo = MockDataServiceRepositoryTrait::new();
    repo.expect_get_all_data_services()
        .withf(|scope, _, _, _| *scope == OwnerScope::All)
        .returning(|_, _, _, _| {
            Ok((
                vec![make_model("tenant-1", 1), make_model("tenant-2", 2)],
                Some(2),
            ))
        });

    let svc = make_svc(repo);
    let page = svc
        .get_all_data_services(
            &TestUsers::user("admin-tenant", "/admin"),
            &DataServiceFilter::default(),
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
    let mut repo = MockDataServiceRepositoryTrait::new();
    repo.expect_put_data_service_by_id()
        .withf(|scope, id, _| *scope == OwnerScope::acting(&TestUsers::alone("tenant-2")) && id == &test_urn(1))
        .returning(|_, _, _| Err(not_found()));

    let svc = make_svc(repo);
    assert!(svc
        .put_data_service_by_id(
            &TestUsers::user("tenant-2", "/admin/tenant-2"),
            &test_urn(1),
            &make_edit_dto()
        )
        .await
        .is_err());
}

/// Deleting a record of another tenant is not found.
#[tokio::test]
async fn delete_foreign_tenant_returns_not_found() {
    let mut repo = MockDataServiceRepositoryTrait::new();
    repo.expect_delete_data_service_by_id()
        .withf(|scope, id| *scope == OwnerScope::acting(&TestUsers::alone("tenant-2")) && id == &test_urn(1))
        .returning(|_, _| Err(not_found()));

    let svc = make_svc(repo);
    assert!(svc
        .delete_data_service_by_id(&TestUsers::user("tenant-2", "/admin/tenant-2"), &test_urn(1))
        .await
        .is_err());
}

/// Deleting a record of the caller's own tenant succeeds.
#[tokio::test]
async fn delete_own_tenant_returns_deleted_row_and_succeeds() {
    let mut repo = MockDataServiceRepositoryTrait::new();
    repo.expect_delete_data_service_by_id()
        .withf(|scope, id| *scope == OwnerScope::acting(&TestUsers::alone("tenant-1")) && id == &test_urn(1))
        .returning(|_, _| Ok(make_model("tenant-1", 1)));

    let svc = make_svc(repo);
    assert!(svc
        .delete_data_service_by_id(&TestUsers::user("tenant-1", "/admin/tenant-1"), &test_urn(1))
        .await
        .is_ok());
}

/// A batch read only returns records of the caller's tenant.
#[tokio::test]
async fn batch_filters_out_foreign_tenant_records() {
    let mut repo = MockDataServiceRepositoryTrait::new();
    repo.expect_get_batch_data_services()
        .withf(|scope, ids| *scope == OwnerScope::seeing(&TestUsers::alone("tenant-2")) && ids == [test_urn(1)])
        .returning(|_, _| Ok(vec![]));

    let svc = make_svc(repo);
    let views = svc
        .get_batch_data_services(&TestUsers::user("tenant-2", "/admin/tenant-2"), &[test_urn(1)])
        .await
        .unwrap();
    assert!(views.is_empty());
}

/// A non-admin always creates in its own tenant, whatever the DTO says.
#[tokio::test]
async fn create_forces_caller_tenant_for_non_admin() {
    let mut repo = MockDataServiceRepositoryTrait::new();
    repo.expect_create_data_service()
        .withf(|cmd| cmd.owner == TestUsers::owner("tenant-2"))
        .returning(|cmd| Ok(make_model(&cmd.owner.user_id, 1)));

    let svc = make_svc(repo);
    let mut cmd = make_new_dto();
    cmd.owner = Some(TestUsers::owner("tenant-1"));
    let dto = svc
        .create_data_service(&TestUsers::user("tenant-2", "/admin/tenant-2"), &cmd)
        .await
        .unwrap();
    assert_eq!(dto.inner.user_id, "tenant-2");
}

/// Only the root creates the main data service, and it is the connector's.
#[tokio::test]
async fn create_main_is_the_roots_and_the_connectors() {
    let mut repo = MockDataServiceRepositoryTrait::new();
    repo.expect_create_main_data_service()
        .withf(|cmd| cmd.owner == Owner::connector())
        .times(1)
        .returning(|cmd| Ok(make_model(&cmd.owner.user_id, 1)));

    let svc = make_svc(repo);
    let cmd = make_new_dto();
    assert!(svc
        .create_main_data_service(&TestUsers::user("tenant-2", "/admin/tenant-2"), &cmd)
        .await
        .is_err());
    let dto = svc.create_main_data_service(&TestUsers::root(), &cmd).await.unwrap();
    assert_eq!(dto.inner.user_id, "system");
}
