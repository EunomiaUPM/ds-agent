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

//! Multi-tenant isolation tests for DatasetEntities with a mocked repository.

mod fixtures;

use std::sync::Arc;

use catalog_agent::data::entities::dataset;
use catalog_agent::data::factory_trait::MockCatalogAgentRepoTrait;
use catalog_agent::data::repo_traits::catalog_db_errors::{
    CatalogAgentRepoErrors, DatasetRepoErrors,
};
use catalog_agent::data::repo_traits::dataset_repo::MockDatasetRepositoryTrait;
use catalog_agent::entities::datasets::datasets::DatasetEntities;
use catalog_agent::entities::datasets::{DatasetEntityTrait, EditDatasetDto, NewDatasetDto};
use catalog_agent::entities::filters::DatasetFilter;
use chrono::Utc;
use common::paginated_spec::{Page, Sort};
use fixtures::{admin_scope, noop_cache_factory, reader_scope, tenant_scope, test_urn};
use ymir::errors::RepoIntoErrors;

fn not_found() -> ymir::errors::Errors {
    CatalogAgentRepoErrors::DatasetRepoErrors(DatasetRepoErrors::DatasetNotFound).into_errors()
}

fn make_svc(repo: MockDatasetRepositoryTrait) -> DatasetEntities {
    let repo = Arc::new(repo);
    let mut factory = MockCatalogAgentRepoTrait::new();
    factory
        .expect_get_dataset_repo()
        .returning(move || repo.clone());
    DatasetEntities::new(Arc::new(factory), noop_cache_factory())
}

fn make_model(tenant: &str, n: u32) -> dataset::Model {
    dataset::Model {
        id: test_urn(n).to_string(),
        tenant_id: tenant.to_string(),
        dct_conforms_to: None,
        dct_creator: None,
        dct_identifier: None,
        dct_issued: Utc::now().into(),
        dct_modified: None,
        dct_title: Some(format!("dataset-{n}")),
        dct_description: None,
        catalog_id: test_urn(100).to_string(),
    }
}

fn make_new_dto() -> NewDatasetDto {
    NewDatasetDto {
        id: None,
        tenant_id: None,
        dct_conforms_to: None,
        dct_creator: None,
        dct_title: Some("new dataset".to_string()),
        dct_description: None,
        catalog_id: test_urn(100),
    }
}

fn make_edit_dto() -> EditDatasetDto {
    EditDatasetDto {
        dct_conforms_to: None,
        dct_creator: None,
        dct_title: Some("edited".to_string()),
        dct_description: None,
    }
}

#[tokio::test]
async fn get_one_foreign_tenant_returns_not_found() {
    let mut repo = MockDatasetRepositoryTrait::new();
    repo.expect_get_dataset_by_id()
        .withf(|tenant, id| tenant == "tenant-2" && id == &test_urn(1))
        .returning(|_, _| Ok(None));

    let svc = make_svc(repo);
    assert!(svc
        .get_dataset_by_id(&tenant_scope("tenant-2"), &test_urn(1))
        .await
        .is_err());
}

#[tokio::test]
async fn get_all_foreign_tenant_query_rejected_with_forbidden() {
    let svc = make_svc(MockDatasetRepositoryTrait::new());
    let filter = DatasetFilter {
        tenant_id: Some("tenant-foreign".to_string()),
        ..Default::default()
    };
    assert!(svc
        .get_all_datasets(
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
    let mut repo = MockDatasetRepositoryTrait::new();
    repo.expect_get_all_datasets()
        .withf(|f, _, _| f.tenant_id.is_none())
        .returning(|_, _, _| {
            Ok((
                vec![make_model("tenant-1", 1), make_model("tenant-2", 2)],
                Some(2),
            ))
        });

    let svc = make_svc(repo);
    let page = svc
        .get_all_datasets(
            &admin_scope(),
            &DatasetFilter::default(),
            &Page::default(),
            &Sort::default(),
        )
        .await
        .unwrap();
    assert_eq!(page.items.len(), 2);
}

#[tokio::test]
async fn by_catalog_id_is_tenant_scoped() {
    let mut repo = MockDatasetRepositoryTrait::new();
    repo.expect_get_datasets_by_catalog_id()
        .withf(|tenant, id| tenant == "tenant-1" && id == &test_urn(100))
        .returning(|tenant, _| Ok(vec![make_model(tenant, 1)]));

    let svc = make_svc(repo);
    let dtos = svc
        .get_datasets_by_catalog_id(&tenant_scope("tenant-1"), &test_urn(100))
        .await
        .unwrap();
    assert_eq!(dtos.len(), 1);
}

#[tokio::test]
async fn edit_foreign_tenant_returns_not_found_without_mutating() {
    let mut repo = MockDatasetRepositoryTrait::new();
    repo.expect_put_dataset_by_id()
        .withf(|tenant, id, _| tenant == "tenant-2" && id == &test_urn(1))
        .returning(|_, _, _| Err(not_found()));

    let svc = make_svc(repo);
    assert!(svc
        .put_dataset_by_id(&tenant_scope("tenant-2"), &test_urn(1), &make_edit_dto())
        .await
        .is_err());
}

#[tokio::test]
async fn delete_foreign_tenant_returns_not_found() {
    let mut repo = MockDatasetRepositoryTrait::new();
    repo.expect_delete_dataset_by_id()
        .withf(|tenant, id| tenant == "tenant-2" && id == &test_urn(1))
        .returning(|_, _| Err(not_found()));

    let svc = make_svc(repo);
    assert!(svc
        .delete_dataset_by_id(&tenant_scope("tenant-2"), &test_urn(1))
        .await
        .is_err());
}

#[tokio::test]
async fn delete_own_tenant_returns_deleted_row_and_succeeds() {
    let mut repo = MockDatasetRepositoryTrait::new();
    repo.expect_delete_dataset_by_id()
        .withf(|tenant, id| tenant == "tenant-1" && id == &test_urn(1))
        .returning(|tenant, _| Ok(make_model(tenant, 1)));

    let svc = make_svc(repo);
    assert!(svc
        .delete_dataset_by_id(&tenant_scope("tenant-1"), &test_urn(1))
        .await
        .is_ok());
}

#[tokio::test]
async fn delete_reader_is_forbidden_before_reaching_repo() {
    let svc = make_svc(MockDatasetRepositoryTrait::new());
    assert!(svc
        .delete_dataset_by_id(&reader_scope("tenant-1"), &test_urn(1))
        .await
        .is_err());
}

#[tokio::test]
async fn batch_filters_out_foreign_tenant_records() {
    let mut repo = MockDatasetRepositoryTrait::new();
    repo.expect_get_batch_datasets()
        .withf(|tenant, ids| tenant == "tenant-2" && ids == [test_urn(1)])
        .returning(|_, _| Ok(vec![]));

    let svc = make_svc(repo);
    let views = svc
        .get_batch_datasets(&tenant_scope("tenant-2"), &[test_urn(1)])
        .await
        .unwrap();
    assert!(views.is_empty());
}

#[tokio::test]
async fn create_forces_caller_tenant_for_non_admin() {
    let mut repo = MockDatasetRepositoryTrait::new();
    repo.expect_create_dataset()
        .withf(|cmd| cmd.tenant_id == "tenant-2")
        .returning(|cmd| Ok(make_model(&cmd.tenant_id, 1)));

    let svc = make_svc(repo);
    let mut cmd = make_new_dto();
    cmd.tenant_id = Some("tenant-1".to_string());
    let dto = svc
        .create_dataset(&tenant_scope("tenant-2"), &cmd)
        .await
        .unwrap();
    assert_eq!(dto.inner.tenant_id, "tenant-2");
}

#[tokio::test]
async fn create_reader_is_forbidden() {
    let svc = make_svc(MockDatasetRepositoryTrait::new());
    assert!(svc
        .create_dataset(&reader_scope("tenant-1"), &make_new_dto())
        .await
        .is_err());
}
