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

//! Unit and isolation tests for OfferService.

use chrono::Utc;
use common::auth::access::AccessScope;
use common::auth::claims::RbacRole;
use common::batch_requests::BatchRequests;
use common::query::{Page, Sort};
use negotiation_agent::data::entities::offer::Model as OfferModel;
use negotiation_agent::data::repo_traits::offer_repo::{MockOfferRepoTrait, OfferRepoErrors};
use negotiation_agent::entities::filters::OfferFilter;
use negotiation_agent::entities::offer::NewOfferDto;
use negotiation_agent::services::offer::OfferServiceTrait;
use negotiation_agent::services::offer::service::OfferService;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::RepoIntoErrors;

fn test_urn(n: u32) -> Urn {
    Urn::from_str(&format!(
        "urn:uuid:33333333-3333-3333-3333-3333333333{:02}",
        n
    ))
    .unwrap()
}

fn tenant_scope(tenant: &str) -> AccessScope {
    AccessScope::from_role(RbacRole::Owner, tenant)
}

fn reader_scope(tenant: &str) -> AccessScope {
    AccessScope::from_role(RbacRole::Reader, tenant)
}

fn make_offer_model(id: &Urn, tenant_id: &str) -> OfferModel {
    OfferModel {
        id: id.to_string(),
        tenant_id: tenant_id.to_string(),
        negotiation_agent_process_id: "urn:uuid:process-1".to_string(),
        negotiation_agent_message_id: "urn:uuid:message-1".to_string(),
        offer_id: "offer-ext-1".to_string(),
        offer_content: serde_json::json!({}),
        created_at: Utc::now().into(),
    }
}

fn make_service(offer_repo: MockOfferRepoTrait) -> OfferService {
    OfferService::new(Arc::new(offer_repo))
}

#[tokio::test]
async fn get_one_foreign_tenant_returns_not_found() {
    let mut offer_repo = MockOfferRepoTrait::new();
    let id = test_urn(1);
    offer_repo
        .expect_get_offer_by_id()
        .withf(move |tenant, oid| tenant.as_deref() == Some("tenant-2") && oid == &test_urn(1))
        .returning(|_, _| Ok(None));

    let svc = make_service(offer_repo);
    assert!(svc.get_one(&tenant_scope("tenant-2"), &id).await.is_err());
}

#[tokio::test]
async fn get_all_foreign_tenant_query_rejected_with_forbidden() {
    let svc = make_service(MockOfferRepoTrait::new());

    let mut filter = OfferFilter::default();
    filter.tenant_id = Some("tenant-foreign".to_string());

    let result = svc
        .get_all(
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
    let mut offer_repo = MockOfferRepoTrait::new();
    let id = test_urn(1);
    offer_repo
        .expect_delete_offer()
        .withf(move |tenant, oid| tenant.as_deref() == Some("tenant-2") && oid == &test_urn(1))
        .returning(|_, _| Err(OfferRepoErrors::OfferNotFound.into_errors()));

    let svc = make_service(offer_repo);
    assert!(svc.delete(&tenant_scope("tenant-2"), &id).await.is_err());
}

#[tokio::test]
async fn batch_filters_out_foreign_tenant_records() {
    let mut offer_repo = MockOfferRepoTrait::new();
    let id = test_urn(1);
    offer_repo
        .expect_get_batch_offers()
        .withf(move |tenant, ids| tenant.as_deref() == Some("tenant-2") && ids == &[test_urn(1)])
        .returning(|_, _| Ok(vec![]));

    let svc = make_service(offer_repo);
    let views = svc
        .batch(&tenant_scope("tenant-2"), &BatchRequests { ids: vec![id] })
        .await
        .unwrap();
    assert!(views.is_empty());
}

#[tokio::test]
async fn create_forces_caller_tenant_for_non_admin() {
    let mut offer_repo = MockOfferRepoTrait::new();
    let id = test_urn(1);
    let id_clone = id.clone();
    offer_repo
        .expect_create_offer()
        .withf(|model| model.tenant_id == "tenant-2")
        .returning(move |model| Ok(make_offer_model(&id_clone, &model.tenant_id)));

    let svc = make_service(offer_repo);
    let cmd = NewOfferDto {
        id: Some(id),
        tenant_id: Some("tenant-1".to_string()),
        negotiation_agent_process_id: test_urn(9),
        negotiation_agent_message_id: test_urn(8),
        offer_id: "offer-ext-1".to_string(),
        offer_content: serde_json::json!({}),
    };
    let view = svc.create(&tenant_scope("tenant-2"), &cmd).await.unwrap();
    assert_eq!(view.inner.tenant_id, "tenant-2");
}

#[tokio::test]
async fn reader_cannot_create_offer() {
    let svc = make_service(MockOfferRepoTrait::new());
    let cmd = NewOfferDto {
        id: Some(test_urn(1)),
        tenant_id: None,
        negotiation_agent_process_id: test_urn(9),
        negotiation_agent_message_id: test_urn(8),
        offer_id: "offer-ext-1".to_string(),
        offer_content: serde_json::json!({}),
    };
    assert!(svc.create(&reader_scope("tenant-1"), &cmd).await.is_err());
}
