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

//! Unit and isolation tests for NegotiationMessageService.

use chrono::Utc;
use common::auth::access::AccessScope;
use common::auth::claims::RbacRole;
use common::batch_requests::BatchRequests;
use common::query::{Page, Sort};
use negotiation_agent::data::entities::negotiation_message::Model as NegotiationMessageModel;
use negotiation_agent::data::repo_traits::agreement_repo::MockAgreementRepoTrait;
use negotiation_agent::data::repo_traits::negotiation_message_repo::{
    MockNegotiationMessageRepoTrait, NegotiationMessageRepoErrors,
};
use negotiation_agent::data::repo_traits::offer_repo::MockOfferRepoTrait;
use negotiation_agent::entities::filters::NegotiationMessageFilter;
use negotiation_agent::entities::negotiation_message::NewNegotiationMessageDto;
use negotiation_agent::services::negotiation_message::NegotiationMessageServiceTrait;
use negotiation_agent::services::negotiation_message::service::NegotiationMessageService;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::RepoIntoErrors;

fn test_urn(n: u32) -> Urn {
    Urn::from_str(&format!(
        "urn:uuid:22222222-2222-2222-2222-2222222222{:02}",
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

fn make_message_model(id: &Urn, tenant_id: &str) -> NegotiationMessageModel {
    NegotiationMessageModel {
        id: id.to_string(),
        tenant_id: tenant_id.to_string(),
        negotiation_agent_process_id: "urn:uuid:process-1".to_string(),
        direction: "SENT".to_string(),
        protocol: "DSP_2025_1".to_string(),
        message_type: "ContractRequestMessage".to_string(),
        state_transition_from: "REQUESTED".to_string(),
        state_transition_to: "OFFERED".to_string(),
        payload: serde_json::json!({}),
        created_at: Utc::now().into(),
    }
}

fn make_service(
    message_repo: MockNegotiationMessageRepoTrait,
    offer_repo: MockOfferRepoTrait,
    agreement_repo: MockAgreementRepoTrait,
) -> NegotiationMessageService {
    NegotiationMessageService::new(
        Arc::new(message_repo),
        Arc::new(offer_repo),
        Arc::new(agreement_repo),
    )
}

#[tokio::test]
async fn get_one_foreign_tenant_returns_not_found() {
    let mut message_repo = MockNegotiationMessageRepoTrait::new();
    let id = test_urn(1);
    message_repo
        .expect_get_negotiation_message_by_id()
        .withf(move |tenant, mid| tenant == "tenant-2" && mid == &test_urn(1))
        .returning(|_, _| Ok(None));

    let svc = make_service(
        message_repo,
        MockOfferRepoTrait::new(),
        MockAgreementRepoTrait::new(),
    );
    assert!(svc.get_one(&tenant_scope("tenant-2"), &id).await.is_err());
}

#[tokio::test]
async fn get_all_foreign_tenant_query_rejected_with_forbidden() {
    let svc = make_service(
        MockNegotiationMessageRepoTrait::new(),
        MockOfferRepoTrait::new(),
        MockAgreementRepoTrait::new(),
    );

    let mut filter = NegotiationMessageFilter::default();
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
    let mut message_repo = MockNegotiationMessageRepoTrait::new();
    let id = test_urn(1);
    message_repo
        .expect_delete_negotiation_message()
        .withf(move |tenant, mid| tenant == "tenant-2" && mid == &test_urn(1))
        .returning(|_, _| {
            Err(NegotiationMessageRepoErrors::NegotiationMessageNotFound.into_errors())
        });

    let svc = make_service(
        message_repo,
        MockOfferRepoTrait::new(),
        MockAgreementRepoTrait::new(),
    );
    assert!(svc.delete(&tenant_scope("tenant-2"), &id).await.is_err());
}

#[tokio::test]
async fn batch_filters_out_foreign_tenant_records() {
    let mut message_repo = MockNegotiationMessageRepoTrait::new();
    let id = test_urn(1);
    message_repo
        .expect_get_batch_negotiation_messages()
        .withf(move |tenant, ids| tenant == "tenant-2" && ids == &[test_urn(1)])
        .returning(|_, _| Ok(vec![]));

    let svc = make_service(
        message_repo,
        MockOfferRepoTrait::new(),
        MockAgreementRepoTrait::new(),
    );
    let views = svc
        .batch(&tenant_scope("tenant-2"), &BatchRequests { ids: vec![id] })
        .await
        .unwrap();
    assert!(views.is_empty());
}

#[tokio::test]
async fn create_forces_caller_tenant_for_non_admin() {
    let mut message_repo = MockNegotiationMessageRepoTrait::new();
    let id = test_urn(1);
    let id_clone = id.clone();
    message_repo
        .expect_create_negotiation_message()
        .withf(|model| model.tenant_id == "tenant-2")
        .returning(move |model| Ok(make_message_model(&id_clone, &model.tenant_id)));

    let svc = make_service(
        message_repo,
        MockOfferRepoTrait::new(),
        MockAgreementRepoTrait::new(),
    );
    let cmd = NewNegotiationMessageDto {
        id: Some(id),
        tenant_id: Some("tenant-1".to_string()),
        negotiation_agent_process_id: test_urn(9),
        direction: "SENT".to_string(),
        protocol: "DSP_2025_1".to_string(),
        message_type: "ContractRequestMessage".to_string(),
        state_transition_from: "REQUESTED".to_string(),
        state_transition_to: "OFFERED".to_string(),
        payload: serde_json::json!({}),
    };
    let view = svc.create(&tenant_scope("tenant-2"), &cmd).await.unwrap();
    assert_eq!(view.inner.tenant_id, "tenant-2");
}

#[tokio::test]
async fn reader_cannot_create_message() {
    let svc = make_service(
        MockNegotiationMessageRepoTrait::new(),
        MockOfferRepoTrait::new(),
        MockAgreementRepoTrait::new(),
    );
    let cmd = NewNegotiationMessageDto {
        id: Some(test_urn(1)),
        tenant_id: None,
        negotiation_agent_process_id: test_urn(9),
        direction: "SENT".to_string(),
        protocol: "DSP_2025_1".to_string(),
        message_type: "ContractRequestMessage".to_string(),
        state_transition_from: "REQUESTED".to_string(),
        state_transition_to: "OFFERED".to_string(),
        payload: serde_json::json!({}),
    };
    assert!(svc.create(&reader_scope("tenant-1"), &cmd).await.is_err());
}
