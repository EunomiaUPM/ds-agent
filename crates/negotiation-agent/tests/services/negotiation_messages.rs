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

//! NegotiationMessageService with mocked repositories: tenant isolation and role checks.

use chrono::Utc;
use common::batch_requests::BatchRequests;
use common::query::{Page, Sort};
use common::oauth::OwnerScope;
use common::test_utils::scopes::TestUsers;
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
        "urn:uuid:22222222-2222-2222-2222-2222222222{n:02}"
    ))
    .unwrap()
}

fn make_message_model(id: &Urn, tenant_id: &str) -> NegotiationMessageModel {
    NegotiationMessageModel {
        id: id.to_string(),
        user_id: tenant_id.to_string(),
        user_role: common::oauth::RolePath::root(),
        visibility: common::oauth::Visibility::Private,
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

/// Reading a record of another tenant is not found: the lookup only searches the
/// caller's tenant.
#[tokio::test]
async fn get_one_foreign_tenant_returns_not_found() {
    let mut message_repo = MockNegotiationMessageRepoTrait::new();
    let id = test_urn(1);
    message_repo
        .expect_get_negotiation_message_by_id()
        .withf(move |scope, mid| *scope == OwnerScope::seeing(&TestUsers::alone("tenant-2")) && mid == &test_urn(1))
        .returning(|_, _| Ok(None));

    let svc = make_service(
        message_repo,
        MockOfferRepoTrait::new(),
        MockAgreementRepoTrait::new(),
    );
    assert!(
        svc.get_one(&TestUsers::user("tenant-2", "/admin/tenant-2"), &id)
            .await
            .is_err()
    );
}

/// Narrowing a listing to another user stays within what the caller sees.
#[tokio::test]
async fn get_all_of_another_user_stays_within_what_the_caller_sees() {
    let mut repo = MockNegotiationMessageRepoTrait::new();
    repo.expect_get_all_negotiation_messages()
        .withf(|scope, f, _, _| {
            *scope == OwnerScope::seeing(&TestUsers::alone("tenant-1"))
                && f.user_id.as_deref() == Some("tenant-foreign")
        })
        .returning(|_, _, _, _| Ok((vec![], Some(0))));
    let svc = make_service(
        repo,
        MockOfferRepoTrait::new(),
        MockAgreementRepoTrait::new(),
    );

    let filter = NegotiationMessageFilter {
        user_id: Some("tenant-foreign".to_string()),
        ..Default::default()
    };

    let result = svc
        .get_all(
            &TestUsers::user("tenant-1", "/admin/tenant-1"),
            &filter,
            &Page::default(),
            &Sort::default(),
        )
        .await;
    assert!(result.unwrap().items.is_empty());
}

/// Deleting a record of another tenant is not found.
#[tokio::test]
async fn delete_foreign_tenant_returns_not_found() {
    let mut message_repo = MockNegotiationMessageRepoTrait::new();
    let id = test_urn(1);
    message_repo
        .expect_delete_negotiation_message()
        .withf(move |scope, mid| *scope == OwnerScope::acting(&TestUsers::alone("tenant-2")) && mid == &test_urn(1))
        .returning(|_, _| {
            Err(NegotiationMessageRepoErrors::NegotiationMessageNotFound.into_errors())
        });

    let svc = make_service(
        message_repo,
        MockOfferRepoTrait::new(),
        MockAgreementRepoTrait::new(),
    );
    assert!(
        svc.delete(&TestUsers::user("tenant-2", "/admin/tenant-2"), &id)
            .await
            .is_err()
    );
}

/// A batch read only returns records of the caller's tenant.
#[tokio::test]
async fn batch_filters_out_foreign_tenant_records() {
    let mut message_repo = MockNegotiationMessageRepoTrait::new();
    let id = test_urn(1);
    message_repo
        .expect_get_batch_negotiation_messages()
        .withf(move |scope, ids| *scope == OwnerScope::seeing(&TestUsers::alone("tenant-2")) && *ids == [test_urn(1)])
        .returning(|_, _| Ok(vec![]));

    let svc = make_service(
        message_repo,
        MockOfferRepoTrait::new(),
        MockAgreementRepoTrait::new(),
    );
    let views = svc
        .batch(
            &TestUsers::user("tenant-2", "/admin/tenant-2"),
            &BatchRequests { ids: vec![id] },
        )
        .await
        .unwrap();
    assert!(views.is_empty());
}

/// A non-admin always creates in its own tenant, whatever the DTO says.
#[tokio::test]
async fn create_forces_caller_tenant_for_non_admin() {
    let mut message_repo = MockNegotiationMessageRepoTrait::new();
    let id = test_urn(1);
    let id_clone = id.clone();
    message_repo
        .expect_create_negotiation_message()
        .withf(|model| model.owner == TestUsers::owner("tenant-2"))
        .returning(move |model| Ok(make_message_model(&id_clone, &model.owner.user_id)));

    let svc = make_service(
        message_repo,
        MockOfferRepoTrait::new(),
        MockAgreementRepoTrait::new(),
    );
    let cmd = NewNegotiationMessageDto {
        id: Some(id),
        visibility: None,
        owner: Some(TestUsers::owner("tenant-1")),
        negotiation_agent_process_id: test_urn(9),
        direction: "SENT".to_string(),
        protocol: "DSP_2025_1".to_string(),
        message_type: "ContractRequestMessage".to_string(),
        state_transition_from: "REQUESTED".to_string(),
        state_transition_to: "OFFERED".to_string(),
        payload: serde_json::json!({}),
    };
    let view = svc
        .create(&TestUsers::user("tenant-2", "/admin/tenant-2"), &cmd)
        .await
        .unwrap();
    assert_eq!(view.inner.user_id, "tenant-2");
}
