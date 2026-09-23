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

//! Multi-tenant isolation tests for OdrlPolicyService with a mocked repository.

mod fixtures;

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
use fixtures::{admin_scope, noop_cache_factory, reader_scope, tenant_scope, test_urn};
use serde_json::json;
use ymir::errors::RepoIntoErrors;

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
        tenant_id: tenant.to_string(),
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
        tenant_id: None,
        odrl_offer: serde_json::from_value(json!({})).unwrap(),
        entity_id: test_urn(100),
        entity_type: CatalogEntityTypes::Dataset,
        source_template_id: None,
        source_template_version: None,
        instantiation_parameters: None,
        description: None,
    }
}

#[tokio::test]
async fn get_one_foreign_tenant_returns_not_found() {
    let mut repo = MockOdrlOfferRepositoryTrait::new();
    repo.expect_get_odrl_offer_by_id()
        .withf(|tenant, id| tenant.as_deref() == Some("tenant-2") && id == &test_urn(1))
        .returning(|_, _| Ok(None));

    let svc = make_svc(repo);
    assert!(svc
        .get_odrl_offer_by_id(&tenant_scope("tenant-2"), &test_urn(1))
        .await
        .is_err());
}

#[tokio::test]
async fn get_all_foreign_tenant_query_rejected_with_forbidden() {
    let svc = make_svc(MockOdrlOfferRepositoryTrait::new());
    let filter = OdrlPolicyFilter {
        tenant_id: Some("tenant-foreign".to_string()),
        ..Default::default()
    };
    assert!(svc
        .get_all_odrl_offers(
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
    let mut repo = MockOdrlOfferRepositoryTrait::new();
    repo.expect_get_all_odrl_offers()
        .withf(|f, _, _| f.tenant_id.is_none())
        .returning(|_, _, _| {
            Ok((
                vec![make_model("tenant-1", 1), make_model("tenant-2", 2)],
                Some(2),
            ))
        });

    let svc = make_svc(repo);
    let page = svc
        .get_all_odrl_offers(
            &admin_scope(),
            &OdrlPolicyFilter::default(),
            &Page::default(),
            &Sort::default(),
        )
        .await
        .unwrap();
    assert_eq!(page.items.len(), 2);
}

#[tokio::test]
async fn by_entity_is_tenant_scoped() {
    let mut repo = MockOdrlOfferRepositoryTrait::new();
    repo.expect_get_all_odrl_offers_by_entity()
        .withf(|tenant, entity| tenant.as_deref() == Some("tenant-2") && entity == &test_urn(100))
        .returning(|_, _| Ok(vec![]));

    let svc = make_svc(repo);
    let dtos = svc
        .get_all_odrl_offers_by_entity(&tenant_scope("tenant-2"), &test_urn(100))
        .await
        .unwrap();
    assert!(dtos.is_empty());
}

#[tokio::test]
async fn delete_foreign_tenant_returns_not_found() {
    let mut repo = MockOdrlOfferRepositoryTrait::new();
    repo.expect_delete_odrl_offer_by_id()
        .withf(|tenant, id| tenant.as_deref() == Some("tenant-2") && id == &test_urn(1))
        .returning(|_, _| Err(not_found()));

    let svc = make_svc(repo);
    assert!(svc
        .delete_odrl_offer_by_id(&tenant_scope("tenant-2"), &test_urn(1))
        .await
        .is_err());
}

#[tokio::test]
async fn delete_by_entity_foreign_tenant_returns_not_found() {
    let mut repo = MockOdrlOfferRepositoryTrait::new();
    repo.expect_delete_odrl_offers_by_entity()
        .withf(|tenant, entity| tenant.as_deref() == Some("tenant-2") && entity == &test_urn(100))
        .returning(|_, _| Err(not_found()));

    let svc = make_svc(repo);
    assert!(svc
        .delete_odrl_offers_by_entity(&tenant_scope("tenant-2"), &test_urn(100))
        .await
        .is_err());
}

#[tokio::test]
async fn delete_by_entity_own_tenant_returns_deleted_rows_and_succeeds() {
    let mut repo = MockOdrlOfferRepositoryTrait::new();
    repo.expect_delete_odrl_offers_by_entity()
        .withf(|tenant, entity| tenant.as_deref() == Some("tenant-1") && entity == &test_urn(100))
        .returning(|tenant, _| {
            Ok(vec![
                make_model(tenant.as_deref().unwrap(), 1),
                make_model(tenant.as_deref().unwrap(), 2),
            ])
        });

    let svc = make_svc(repo);
    assert!(svc
        .delete_odrl_offers_by_entity(&tenant_scope("tenant-1"), &test_urn(100))
        .await
        .is_ok());
}

#[tokio::test]
async fn delete_reader_is_forbidden_before_reaching_repo() {
    let svc = make_svc(MockOdrlOfferRepositoryTrait::new());
    assert!(svc
        .delete_odrl_offer_by_id(&reader_scope("tenant-1"), &test_urn(1))
        .await
        .is_err());
    assert!(svc
        .delete_odrl_offers_by_entity(&reader_scope("tenant-1"), &test_urn(100))
        .await
        .is_err());
}

#[tokio::test]
async fn batch_filters_out_foreign_tenant_records() {
    let mut repo = MockOdrlOfferRepositoryTrait::new();
    repo.expect_get_batch_odrl_offers()
        .withf(|tenant, ids| tenant.as_deref() == Some("tenant-2") && ids == [test_urn(1)])
        .returning(|_, _| Ok(vec![]));

    let svc = make_svc(repo);
    let views = svc
        .get_batch_odrl_offers(&tenant_scope("tenant-2"), &[test_urn(1)])
        .await
        .unwrap();
    assert!(views.is_empty());
}

#[tokio::test]
async fn create_forces_caller_tenant_for_non_admin() {
    let mut repo = MockOdrlOfferRepositoryTrait::new();
    repo.expect_create_odrl_offer()
        .withf(|cmd| cmd.tenant_id == "tenant-2")
        .returning(|cmd| Ok(make_model(&cmd.tenant_id, 1)));

    let svc = make_svc(repo);
    let mut cmd = make_new_dto();
    cmd.tenant_id = Some("tenant-1".to_string());
    let dto = svc
        .create_odrl_offer(&tenant_scope("tenant-2"), &cmd)
        .await
        .unwrap();
    assert_eq!(dto.inner.tenant_id, "tenant-2");
}

#[tokio::test]
async fn create_reader_is_forbidden() {
    let svc = make_svc(MockOdrlOfferRepositoryTrait::new());
    assert!(svc
        .create_odrl_offer(&reader_scope("tenant-1"), &make_new_dto())
        .await
        .is_err());
}
