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

//! Multi-tenant isolation tests for DataServiceEntities with a mocked repository.

mod fixtures;

use std::sync::Arc;

use catalog_agent::data::entities::dataservice;
use catalog_agent::data::factory_trait::MockCatalogAgentRepoTrait;
use catalog_agent::data::repo_traits::catalog_db_errors::{
    CatalogAgentRepoErrors, DataServiceRepoErrors,
};
use catalog_agent::data::repo_traits::dataservice_repo::MockDataServiceRepositoryTrait;
use catalog_agent::entities::data_services::data_services::DataServiceEntities;
use catalog_agent::entities::data_services::{
    DataServiceEntityTrait, EditDataServiceDto, NewDataServiceDto,
};
use catalog_agent::entities::filters::DataServiceFilter;
use chrono::Utc;
use common::paginated_spec::{Page, Sort};
use fixtures::{admin_scope, noop_cache_factory, reader_scope, tenant_scope, test_urn};
use ymir::errors::RepoIntoErrors;

fn not_found() -> ymir::errors::Errors {
    CatalogAgentRepoErrors::DataServiceRepoErrors(DataServiceRepoErrors::DataServiceNotFound)
        .into_errors()
}

fn make_svc(repo: MockDataServiceRepositoryTrait) -> DataServiceEntities {
    let repo = Arc::new(repo);
    let mut factory = MockCatalogAgentRepoTrait::new();
    factory
        .expect_get_dataservice_repo()
        .returning(move || repo.clone());
    DataServiceEntities::new(Arc::new(factory), noop_cache_factory())
}

fn make_model(tenant: &str, n: u32) -> dataservice::Model {
    dataservice::Model {
        id: test_urn(n).to_string(),
        tenant_id: tenant.to_string(),
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

#[tokio::test]
async fn get_one_foreign_tenant_returns_not_found() {
    let mut repo = MockDataServiceRepositoryTrait::new();
    repo.expect_get_data_service_by_id()
        .withf(|tenant, id| tenant == "tenant-2" && id == &test_urn(1))
        .returning(|_, _| Ok(None));

    let svc = make_svc(repo);
    assert!(svc
        .get_data_service_by_id(&tenant_scope("tenant-2"), &test_urn(1))
        .await
        .is_err());
}

#[tokio::test]
async fn get_main_is_tenant_scoped() {
    let mut repo = MockDataServiceRepositoryTrait::new();
    repo.expect_get_main_data_service()
        .withf(|tenant| tenant == "tenant-1")
        .returning(|tenant| Ok(Some(make_model(tenant, 1))));

    let svc = make_svc(repo);
    let dto = svc
        .get_main_data_service(&tenant_scope("tenant-1"))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(dto.inner.tenant_id, "tenant-1");
}

#[tokio::test]
async fn get_all_foreign_tenant_query_rejected_with_forbidden() {
    let svc = make_svc(MockDataServiceRepositoryTrait::new());
    let filter = DataServiceFilter {
        tenant_id: Some("tenant-foreign".to_string()),
        ..Default::default()
    };
    assert!(svc
        .get_all_data_services(
            &tenant_scope("tenant-1"),
            &filter,
            &Page::default(),
            &Sort::default()
        )
        .await
        .is_err());
}

#[tokio::test]
async fn get_all_admin_without_tenant_queries_cross_tenant() {
    let mut repo = MockDataServiceRepositoryTrait::new();
    repo.expect_get_all_data_services()
        .withf(|f, _, _| f.tenant_id.is_none())
        .returning(|_, _, _| {
            Ok((
                vec![make_model("tenant-1", 1), make_model("tenant-2", 2)],
                Some(2),
            ))
        });

    let svc = make_svc(repo);
    let page = svc
        .get_all_data_services(
            &admin_scope(),
            &DataServiceFilter::default(),
            &Page::default(),
            &Sort::default(),
        )
        .await
        .unwrap();
    assert_eq!(page.items.len(), 2);
}

#[tokio::test]
async fn edit_foreign_tenant_returns_not_found_without_mutating() {
    let mut repo = MockDataServiceRepositoryTrait::new();
    repo.expect_put_data_service_by_id()
        .withf(|tenant, id, _| tenant == "tenant-2" && id == &test_urn(1))
        .returning(|_, _, _| Err(not_found()));

    let svc = make_svc(repo);
    assert!(svc
        .put_data_service_by_id(&tenant_scope("tenant-2"), &test_urn(1), &make_edit_dto())
        .await
        .is_err());
}

#[tokio::test]
async fn delete_foreign_tenant_returns_not_found() {
    let mut repo = MockDataServiceRepositoryTrait::new();
    repo.expect_delete_data_service_by_id()
        .withf(|tenant, id| tenant == "tenant-2" && id == &test_urn(1))
        .returning(|_, _| Err(not_found()));

    let svc = make_svc(repo);
    assert!(svc
        .delete_data_service_by_id(&tenant_scope("tenant-2"), &test_urn(1))
        .await
        .is_err());
}

#[tokio::test]
async fn delete_own_tenant_returns_deleted_row_and_succeeds() {
    let mut repo = MockDataServiceRepositoryTrait::new();
    repo.expect_delete_data_service_by_id()
        .withf(|tenant, id| tenant == "tenant-1" && id == &test_urn(1))
        .returning(|tenant, _| Ok(make_model(tenant, 1)));

    let svc = make_svc(repo);
    assert!(svc
        .delete_data_service_by_id(&tenant_scope("tenant-1"), &test_urn(1))
        .await
        .is_ok());
}

#[tokio::test]
async fn delete_reader_is_forbidden_before_reaching_repo() {
    let svc = make_svc(MockDataServiceRepositoryTrait::new());
    assert!(svc
        .delete_data_service_by_id(&reader_scope("tenant-1"), &test_urn(1))
        .await
        .is_err());
}

#[tokio::test]
async fn batch_filters_out_foreign_tenant_records() {
    let mut repo = MockDataServiceRepositoryTrait::new();
    repo.expect_get_batch_data_services()
        .withf(|tenant, ids| tenant == "tenant-2" && ids == [test_urn(1)])
        .returning(|_, _| Ok(vec![]));

    let svc = make_svc(repo);
    let views = svc
        .get_batch_data_services(&tenant_scope("tenant-2"), &[test_urn(1)])
        .await
        .unwrap();
    assert!(views.is_empty());
}

#[tokio::test]
async fn create_forces_caller_tenant_for_non_admin() {
    let mut repo = MockDataServiceRepositoryTrait::new();
    repo.expect_create_data_service()
        .withf(|cmd| cmd.tenant_id == "tenant-2")
        .returning(|cmd| Ok(make_model(&cmd.tenant_id, 1)));

    let svc = make_svc(repo);
    let mut cmd = make_new_dto();
    cmd.tenant_id = Some("tenant-1".to_string());
    let dto = svc
        .create_data_service(&tenant_scope("tenant-2"), &cmd)
        .await
        .unwrap();
    assert_eq!(dto.inner.tenant_id, "tenant-2");
}

#[tokio::test]
async fn create_main_forces_caller_tenant_for_non_admin() {
    let mut repo = MockDataServiceRepositoryTrait::new();
    repo.expect_create_main_data_service()
        .withf(|cmd| cmd.tenant_id == "tenant-2")
        .returning(|cmd| Ok(make_model(&cmd.tenant_id, 1)));

    let svc = make_svc(repo);
    let mut cmd = make_new_dto();
    cmd.tenant_id = Some("tenant-1".to_string());
    let dto = svc
        .create_main_data_service(&tenant_scope("tenant-2"), &cmd)
        .await
        .unwrap();
    assert_eq!(dto.inner.tenant_id, "tenant-2");
}

#[tokio::test]
async fn create_reader_is_forbidden() {
    let svc = make_svc(MockDataServiceRepositoryTrait::new());
    assert!(svc
        .create_data_service(&reader_scope("tenant-1"), &make_new_dto())
        .await
        .is_err());
}
