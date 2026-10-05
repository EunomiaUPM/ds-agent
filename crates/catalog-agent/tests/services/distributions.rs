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

//! DistributionService with a mocked repository and no-op cache: tenant isolation and role
//! checks.

use std::sync::Arc;

use catalog_agent::data::entities::distribution;
use catalog_agent::data::factory_trait::MockCatalogAgentRepoTrait;
use catalog_agent::data::repo_traits::catalog_db_errors::{
    CatalogAgentRepoErrors, DistributionRepoErrors,
};
use catalog_agent::data::repo_traits::distribution_repo::MockDistributionRepositoryTrait;
use catalog_agent::entities::distributions::{EditDistributionDto, NewDistributionDto};
use catalog_agent::entities::filters::DistributionFilter;
use catalog_agent::services::distributions::service::DistributionService;
use catalog_agent::services::distributions::DistributionServiceTrait;
use chrono::Utc;
use common::paginated_spec::{Page, Sort};
use common::oauth::OwnerScope;
use common::test_utils::scopes::TestUsers;
use ymir::errors::RepoIntoErrors;

use crate::support::fixtures::{noop_cache_factory, test_urn};

fn not_found() -> ymir::errors::Errors {
    CatalogAgentRepoErrors::DistributionRepoErrors(DistributionRepoErrors::DistributionNotFound)
        .into_errors()
}

fn make_svc(repo: MockDistributionRepositoryTrait) -> DistributionService {
    let repo = Arc::new(repo);
    let mut factory = MockCatalogAgentRepoTrait::new();
    factory
        .expect_get_distribution_repo()
        .returning(move || repo.clone());
    DistributionService::new(Arc::new(factory), noop_cache_factory())
}

fn make_model(tenant: &str, n: u32) -> distribution::Model {
    distribution::Model {
        id: test_urn(n).to_string(),
        user_id: tenant.to_string(),
        user_role: common::oauth::RolePath::root(),
        visibility: common::oauth::Visibility::Private,
        dct_issued: Utc::now().into(),
        dct_modified: None,
        dct_title: Some(format!("distribution-{n}")),
        dct_description: None,
        dcat_access_service: test_urn(200).to_string(),
        dataset_id: test_urn(100).to_string(),
        dct_format: Some("http-pull".to_string()),
    }
}

fn make_new_dto() -> NewDistributionDto {
    NewDistributionDto {
        id: None,
        visibility: None,
        owner: None,
        dct_title: Some("new distribution".to_string()),
        dct_description: None,
        dct_formats: Some("http-pull".to_string()),
        dcat_access_service: test_urn(200).to_string(),
        dataset_id: test_urn(100),
    }
}

fn make_edit_dto() -> EditDistributionDto {
    EditDistributionDto {
        dct_title: Some("edited".to_string()),
        dct_description: None,
        dcat_access_service: None,
    }
}

/// Reading a record of another tenant is not found: the lookup only searches the
/// caller's tenant.
#[tokio::test]
async fn get_one_foreign_tenant_returns_not_found() {
    let mut repo = MockDistributionRepositoryTrait::new();
    repo.expect_get_distribution_by_id()
        .withf(|scope, id| *scope == OwnerScope::seeing(&TestUsers::alone("tenant-2")) && id == &test_urn(1))
        .returning(|_, _| Ok(None));

    let svc = make_svc(repo);
    assert!(svc
        .get_distribution_by_id(&TestUsers::user("tenant-2", "/admin/tenant-2"), &test_urn(1))
        .await
        .is_err());
}

/// Looking up a distribution by dataset and format in another tenant is not found.
#[tokio::test]
async fn get_by_dataset_and_format_foreign_tenant_returns_not_found() {
    let mut repo = MockDistributionRepositoryTrait::new();
    repo.expect_get_distribution_by_dataset_id_and_dct_format()
        .withf(|scope, id, fmt| {
            *scope == OwnerScope::seeing(&TestUsers::alone("tenant-2"))
                && id == &test_urn(100)
                && fmt == "http-pull"
        })
        .returning(|_, _, _| Ok(None));

    let svc = make_svc(repo);
    assert!(svc
        .get_distribution_by_dataset_id_and_dct_format(
            &TestUsers::user("tenant-2", "/admin/tenant-2"),
            &test_urn(100),
            "http-pull"
        )
        .await
        .is_err());
}

/// Narrowing a listing to another user stays within what the caller sees.
#[tokio::test]
async fn get_all_of_another_user_stays_within_what_the_caller_sees() {
    let mut repo = MockDistributionRepositoryTrait::new();
    repo.expect_get_all_distributions()
        .withf(|scope, f, _, _| {
            *scope == OwnerScope::seeing(&TestUsers::alone("tenant-1"))
                && f.user_id.as_deref() == Some("tenant-foreign")
        })
        .returning(|_, _, _, _| Ok((vec![], Some(0))));
    let svc = make_svc(repo);
    let filter = DistributionFilter {
        user_id: Some("tenant-foreign".to_string()),
        ..Default::default()
    };
    let page = svc
        .get_all_distributions(
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
    let mut repo = MockDistributionRepositoryTrait::new();
    repo.expect_get_all_distributions()
        .withf(|scope, _, _, _| *scope == OwnerScope::All)
        .returning(|_, _, _, _| {
            Ok((
                vec![make_model("tenant-1", 1), make_model("tenant-2", 2)],
                Some(2),
            ))
        });

    let svc = make_svc(repo);
    let page = svc
        .get_all_distributions(
            &TestUsers::user("admin-tenant", "/admin"),
            &DistributionFilter::default(),
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
    let mut repo = MockDistributionRepositoryTrait::new();
    repo.expect_put_distribution_by_id()
        .withf(|scope, id, _| *scope == OwnerScope::acting(&TestUsers::alone("tenant-2")) && id == &test_urn(1))
        .returning(|_, _, _| Err(not_found()));

    let svc = make_svc(repo);
    assert!(svc
        .put_distribution_by_id(
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
    let mut repo = MockDistributionRepositoryTrait::new();
    repo.expect_delete_distribution_by_id()
        .withf(|scope, id| *scope == OwnerScope::acting(&TestUsers::alone("tenant-2")) && id == &test_urn(1))
        .returning(|_, _| Err(not_found()));

    let svc = make_svc(repo);
    assert!(svc
        .delete_distribution_by_id(&TestUsers::user("tenant-2", "/admin/tenant-2"), &test_urn(1))
        .await
        .is_err());
}

/// Deleting a record of the caller's own tenant succeeds.
#[tokio::test]
async fn delete_own_tenant_returns_deleted_row_and_succeeds() {
    let mut repo = MockDistributionRepositoryTrait::new();
    repo.expect_delete_distribution_by_id()
        .withf(|scope, id| *scope == OwnerScope::acting(&TestUsers::alone("tenant-1")) && id == &test_urn(1))
        .returning(|_, _| Ok(make_model("tenant-1", 1)));

    let svc = make_svc(repo);
    assert!(svc
        .delete_distribution_by_id(&TestUsers::user("tenant-1", "/admin/tenant-1"), &test_urn(1))
        .await
        .is_ok());
}

/// A batch read only returns records of the caller's tenant.
#[tokio::test]
async fn batch_filters_out_foreign_tenant_records() {
    let mut repo = MockDistributionRepositoryTrait::new();
    repo.expect_get_batch_distributions()
        .withf(|scope, ids| *scope == OwnerScope::seeing(&TestUsers::alone("tenant-2")) && ids == [test_urn(1)])
        .returning(|_, _| Ok(vec![]));

    let svc = make_svc(repo);
    let views = svc
        .get_batch_distributions(&TestUsers::user("tenant-2", "/admin/tenant-2"), &[test_urn(1)])
        .await
        .unwrap();
    assert!(views.is_empty());
}

/// A non-admin always creates in its own tenant, whatever the DTO says.
#[tokio::test]
async fn create_forces_caller_tenant_for_non_admin() {
    let mut repo = MockDistributionRepositoryTrait::new();
    repo.expect_create_distribution()
        .withf(|cmd| cmd.owner == TestUsers::owner("tenant-2"))
        .returning(|cmd| Ok(make_model(&cmd.owner.user_id, 1)));

    let svc = make_svc(repo);
    let mut cmd = make_new_dto();
    cmd.owner = Some(TestUsers::owner("tenant-1"));
    let dto = svc
        .create_distribution(&TestUsers::user("tenant-2", "/admin/tenant-2"), &cmd)
        .await
        .unwrap();
    assert_eq!(dto.inner.user_id, "tenant-2");
}
