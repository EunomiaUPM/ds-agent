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

//! OdrlPolicyService with a mocked repository and no-op cache: tenant isolation and role
//! checks.

use std::sync::Arc;

use catalog_agent::data::entities::odrl_offer;
use catalog_agent::data::factory_trait::MockCatalogAgentRepoTrait;
use catalog_agent::data::repo_traits::catalog_db_errors::{
    CatalogAgentRepoErrors, OdrlOfferRepoErrors,
};
use catalog_agent::data::repo_traits::odrl_offer_repo::MockOdrlOfferRepositoryTrait;
use catalog_agent::entities::filters::OdrlPolicyFilter;
use catalog_agent::entities::odrl_policies::{CatalogEntityTypes, NewOdrlPolicyDto};
use catalog_agent::services::odrl_policies::service::OdrlPolicyService;
use catalog_agent::services::odrl_policies::OdrlPolicyServiceTrait;
use chrono::Utc;
use common::paginated_spec::{Page, Sort};
use common::oauth::OwnerScope;
use common::test_utils::scopes::TestUsers;
use serde_json::json;
use ymir::errors::RepoIntoErrors;

use crate::support::fixtures::{noop_cache_factory, test_urn};

fn not_found() -> ymir::errors::Errors {
    CatalogAgentRepoErrors::OdrlOfferRepoErrors(OdrlOfferRepoErrors::OdrlOfferNotFound)
        .into_errors()
}

fn make_svc(repo: MockOdrlOfferRepositoryTrait) -> OdrlPolicyService {
    let repo = Arc::new(repo);
    let mut factory = MockCatalogAgentRepoTrait::new();
    factory
        .expect_get_odrl_offer_repo()
        .returning(move || repo.clone());
    OdrlPolicyService::new(Arc::new(factory), noop_cache_factory())
}

fn make_model(tenant: &str, n: u32) -> odrl_offer::Model {
    odrl_offer::Model {
        id: test_urn(n).to_string(),
        user_id: tenant.to_string(),
        user_role: common::oauth::RolePath::root(),
        visibility: common::oauth::Visibility::Private,
        odrl_offer: json!({}),
        entity: test_urn(100).to_string(),
        entity_type: "Dataset".to_string(),
        created_at: Utc::now().into(),
        source_template_id: None,
        source_template_version: None,
        instantiation_parameters: None,
        description: None,
    }
}

fn make_new_dto() -> NewOdrlPolicyDto {
    NewOdrlPolicyDto {
        id: None,
        visibility: None,
        owner: None,
        odrl_offer: serde_json::from_value(json!({})).unwrap(),
        entity_id: test_urn(100),
        entity_type: CatalogEntityTypes::Dataset,
        source_template_id: None,
        source_template_version: None,
        instantiation_parameters: None,
        description: None,
    }
}

/// Reading a record of another tenant is not found: the lookup only searches the
/// caller's tenant.
#[tokio::test]
async fn get_one_foreign_tenant_returns_not_found() {
    let mut repo = MockOdrlOfferRepositoryTrait::new();
    repo.expect_get_odrl_offer_by_id()
        .withf(|scope, id| *scope == OwnerScope::seeing(&TestUsers::alone("tenant-2")) && id == &test_urn(1))
        .returning(|_, _| Ok(None));

    let svc = make_svc(repo);
    assert!(svc
        .get_odrl_offer_by_id(&TestUsers::user("tenant-2", "/admin/tenant-2"), &test_urn(1))
        .await
        .is_err());
}

/// Narrowing a listing to another user stays within what the caller sees.
#[tokio::test]
async fn get_all_of_another_user_stays_within_what_the_caller_sees() {
    let mut repo = MockOdrlOfferRepositoryTrait::new();
    repo.expect_get_all_odrl_offers()
        .withf(|scope, f, _, _| {
            *scope == OwnerScope::seeing(&TestUsers::alone("tenant-1"))
                && f.user_id.as_deref() == Some("tenant-foreign")
        })
        .returning(|_, _, _, _| Ok((vec![], Some(0))));
    let svc = make_svc(repo);
    let filter = OdrlPolicyFilter {
        user_id: Some("tenant-foreign".to_string()),
        ..Default::default()
    };
    let page = svc
        .get_all_odrl_offers(
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
    let mut repo = MockOdrlOfferRepositoryTrait::new();
    repo.expect_get_all_odrl_offers()
        .withf(|scope, _, _, _| *scope == OwnerScope::All)
        .returning(|_, _, _, _| {
            Ok((
                vec![make_model("tenant-1", 1), make_model("tenant-2", 2)],
                Some(2),
            ))
        });

    let svc = make_svc(repo);
    let page = svc
        .get_all_odrl_offers(
            &TestUsers::user("admin-tenant", "/admin"),
            &OdrlPolicyFilter::default(),
            &Page::default(),
            &Sort::default(),
        )
        .await
        .unwrap();
    assert_eq!(page.items.len(), 2);
}

/// Offers of an entity are looked up in the caller's tenant.
#[tokio::test]
async fn by_entity_is_tenant_scoped() {
    let mut repo = MockOdrlOfferRepositoryTrait::new();
    repo.expect_get_all_odrl_offers_by_entity()
        .withf(|scope, entity| *scope == OwnerScope::seeing(&TestUsers::alone("tenant-2")) && entity == &test_urn(100))
        .returning(|_, _| Ok(vec![]));

    let svc = make_svc(repo);
    let dtos = svc
        .get_all_odrl_offers_by_entity(&TestUsers::user("tenant-2", "/admin/tenant-2"), &test_urn(100))
        .await
        .unwrap();
    assert!(dtos.is_empty());
}

/// Deleting a record of another tenant is not found.
#[tokio::test]
async fn delete_foreign_tenant_returns_not_found() {
    let mut repo = MockOdrlOfferRepositoryTrait::new();
    repo.expect_delete_odrl_offer_by_id()
        .withf(|scope, id| *scope == OwnerScope::acting(&TestUsers::alone("tenant-2")) && id == &test_urn(1))
        .returning(|_, _| Err(not_found()));

    let svc = make_svc(repo);
    assert!(svc
        .delete_odrl_offer_by_id(&TestUsers::user("tenant-2", "/admin/tenant-2"), &test_urn(1))
        .await
        .is_err());
}

/// Deleting the offers of an entity in another tenant is not found.
#[tokio::test]
async fn delete_by_entity_foreign_tenant_returns_not_found() {
    let mut repo = MockOdrlOfferRepositoryTrait::new();
    repo.expect_delete_odrl_offers_by_entity()
        .withf(|scope, entity| *scope == OwnerScope::acting(&TestUsers::alone("tenant-2")) && entity == &test_urn(100))
        .returning(|_, _| Err(not_found()));

    let svc = make_svc(repo);
    assert!(svc
        .delete_odrl_offers_by_entity(&TestUsers::user("tenant-2", "/admin/tenant-2"), &test_urn(100))
        .await
        .is_err());
}

/// Deleting the offers of an entity in the caller's own tenant succeeds.
#[tokio::test]
async fn delete_by_entity_own_tenant_returns_deleted_rows_and_succeeds() {
    let mut repo = MockOdrlOfferRepositoryTrait::new();
    repo.expect_delete_odrl_offers_by_entity()
        .withf(|scope, entity| *scope == OwnerScope::acting(&TestUsers::alone("tenant-1")) && entity == &test_urn(100))
        .returning(|_, _| {
            Ok(vec![
                make_model("tenant-1", 1),
                make_model("tenant-1", 2),
            ])
        });

    let svc = make_svc(repo);
    assert!(svc
        .delete_odrl_offers_by_entity(&TestUsers::user("tenant-1", "/admin/tenant-1"), &test_urn(100))
        .await
        .is_ok());
}

/// A batch read only returns records of the caller's tenant.
#[tokio::test]
async fn batch_filters_out_foreign_tenant_records() {
    let mut repo = MockOdrlOfferRepositoryTrait::new();
    repo.expect_get_batch_odrl_offers()
        .withf(|scope, ids| *scope == OwnerScope::seeing(&TestUsers::alone("tenant-2")) && ids == [test_urn(1)])
        .returning(|_, _| Ok(vec![]));

    let svc = make_svc(repo);
    let views = svc
        .get_batch_odrl_offers(&TestUsers::user("tenant-2", "/admin/tenant-2"), &[test_urn(1)])
        .await
        .unwrap();
    assert!(views.is_empty());
}

/// A non-admin always creates in its own tenant, whatever the DTO says.
#[tokio::test]
async fn create_forces_caller_tenant_for_non_admin() {
    let mut repo = MockOdrlOfferRepositoryTrait::new();
    repo.expect_create_odrl_offer()
        .withf(|cmd| cmd.owner == TestUsers::owner("tenant-2"))
        .returning(|cmd| Ok(make_model(&cmd.owner.user_id, 1)));

    let svc = make_svc(repo);
    let mut cmd = make_new_dto();
    cmd.owner = Some(TestUsers::owner("tenant-1"));
    let dto = svc
        .create_odrl_offer(&TestUsers::user("tenant-2", "/admin/tenant-2"), &cmd)
        .await
        .unwrap();
    assert_eq!(dto.inner.user_id, "tenant-2");
}
