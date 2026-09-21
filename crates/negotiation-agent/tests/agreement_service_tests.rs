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

//! Unit and isolation tests for AgreementService.

use chrono::Utc;
use common::auth::access::AccessScope;
use common::auth::claims::RbacRole;
use common::batch_requests::BatchRequests;
use common::query::{Page, Sort};
use negotiation_agent::data::entities::agreement::Model as AgreementModel;
use negotiation_agent::data::repo_traits::agreement_repo::{
    AgreementRepoErrors, MockAgreementRepoTrait,
};
use negotiation_agent::entities::agreement::{EditAgreementDto, NewAgreementDto};
use negotiation_agent::entities::filters::AgreementFilter;
use negotiation_agent::services::agreement::AgreementServiceTrait;
use negotiation_agent::services::agreement::service::AgreementService;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::RepoIntoErrors;

fn test_urn(n: u32) -> Urn {
    Urn::from_str(&format!(
        "urn:uuid:44444444-4444-4444-4444-4444444444{:02}",
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

fn make_agreement_model(id: &Urn, tenant_id: &str) -> AgreementModel {
    AgreementModel {
        id: id.to_string(),
        tenant_id: tenant_id.to_string(),
        negotiation_agent_process_id: "urn:uuid:process-1".to_string(),
        negotiation_agent_message_id: "urn:uuid:message-1".to_string(),
        consumer_participant_id: "urn:uuid:consumer-1".to_string(),
        provider_participant_id: "urn:uuid:provider-1".to_string(),
        agreement_content: serde_json::json!({}),
        target: "urn:uuid:target-1".to_string(),
        state: "AGREED".to_string(),
        created_at: Utc::now().into(),
        updated_at: None,
    }
}

fn make_service(agreement_repo: MockAgreementRepoTrait) -> AgreementService {
    AgreementService::new(Arc::new(agreement_repo))
}

#[tokio::test]
async fn get_one_foreign_tenant_returns_not_found() {
    let mut agreement_repo = MockAgreementRepoTrait::new();
    let id = test_urn(1);
    agreement_repo
        .expect_get_agreement_by_id()
        .withf(move |tenant, aid| tenant == "tenant-2" && aid == &test_urn(1))
        .returning(|_, _| Ok(None));

    let svc = make_service(agreement_repo);
    assert!(svc.get_one(&tenant_scope("tenant-2"), &id).await.is_err());
}

#[tokio::test]
async fn get_all_foreign_tenant_query_rejected_with_forbidden() {
    let svc = make_service(MockAgreementRepoTrait::new());

    let mut filter = AgreementFilter::default();
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
async fn edit_foreign_tenant_returns_not_found_without_mutating() {
    let mut agreement_repo = MockAgreementRepoTrait::new();
    let id = test_urn(1);
    agreement_repo
        .expect_put_agreement()
        .withf(move |tenant, aid, _| tenant == "tenant-2" && aid == &test_urn(1))
        .returning(|_, _, _| Err(AgreementRepoErrors::AgreementNotFound.into_errors()));

    let svc = make_service(agreement_repo);
    let cmd = EditAgreementDto {
        state: Some("FINALIZED".to_string()),
    };
    assert!(
        svc.edit(&tenant_scope("tenant-2"), &id, &cmd)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn delete_foreign_tenant_returns_not_found() {
    let mut agreement_repo = MockAgreementRepoTrait::new();
    let id = test_urn(1);
    agreement_repo
        .expect_delete_agreement()
        .withf(move |tenant, aid| tenant == "tenant-2" && aid == &test_urn(1))
        .returning(|_, _| Err(AgreementRepoErrors::AgreementNotFound.into_errors()));

    let svc = make_service(agreement_repo);
    assert!(svc.delete(&tenant_scope("tenant-2"), &id).await.is_err());
}

#[tokio::test]
async fn batch_filters_out_foreign_tenant_records() {
    let mut agreement_repo = MockAgreementRepoTrait::new();
    let id = test_urn(1);
    agreement_repo
        .expect_get_batch_agreements()
        .withf(move |tenant, ids| tenant == "tenant-2" && ids == &[test_urn(1)])
        .returning(|_, _| Ok(vec![]));

    let svc = make_service(agreement_repo);
    let views = svc
        .batch(&tenant_scope("tenant-2"), &BatchRequests { ids: vec![id] })
        .await
        .unwrap();
    assert!(views.is_empty());
}

#[tokio::test]
async fn create_forces_caller_tenant_for_non_admin() {
    let mut agreement_repo = MockAgreementRepoTrait::new();
    let id = test_urn(1);
    let id_clone = id.clone();
    agreement_repo
        .expect_create_agreement()
        .withf(|model| model.tenant_id == "tenant-2")
        .returning(move |model| Ok(make_agreement_model(&id_clone, &model.tenant_id)));

    let svc = make_service(agreement_repo);
    let cmd = NewAgreementDto {
        id: Some(id),
        tenant_id: Some("tenant-1".to_string()),
        negotiation_agent_process_id: test_urn(9),
        negotiation_agent_message_id: test_urn(8),
        consumer_participant_id: "urn:uuid:consumer-1".to_string(),
        provider_participant_id: "urn:uuid:provider-1".to_string(),
        agreement_content: serde_json::json!({}),
        target: test_urn(7),
    };
    let view = svc.create(&tenant_scope("tenant-2"), &cmd).await.unwrap();
    assert_eq!(view.inner.tenant_id, "tenant-2");
}

#[tokio::test]
async fn reader_cannot_create_agreement() {
    let svc = make_service(MockAgreementRepoTrait::new());
    let cmd = NewAgreementDto {
        id: Some(test_urn(1)),
        tenant_id: None,
        negotiation_agent_process_id: test_urn(9),
        negotiation_agent_message_id: test_urn(8),
        consumer_participant_id: "urn:uuid:consumer-1".to_string(),
        provider_participant_id: "urn:uuid:provider-1".to_string(),
        agreement_content: serde_json::json!({}),
        target: test_urn(7),
    };
    assert!(svc.create(&reader_scope("tenant-1"), &cmd).await.is_err());
}
