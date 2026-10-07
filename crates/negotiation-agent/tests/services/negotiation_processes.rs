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

//! NegotiationProcessService with mocked repositories: tenant isolation and role checks.

use chrono::Utc;
use common::batch_requests::BatchRequests;
use common::query::{Page, Sort};
use common::oauth::OwnerScope;
use common::test_utils::scopes::TestUsers;
use negotiation_agent::data::entities::negotiation_process::Model as NegotiationProcessModel;
use negotiation_agent::data::repo_traits::agreement_repo::MockAgreementRepoTrait;
use negotiation_agent::data::repo_traits::negotiation_message_repo::MockNegotiationMessageRepoTrait;
use negotiation_agent::data::repo_traits::negotiation_process_identifiers_repo::MockNegotiationIdentifierRepoTrait;
use negotiation_agent::data::repo_traits::negotiation_process_repo::{
    MockNegotiationProcessRepoTrait, NegotiationProcessRepoErrors,
};
use negotiation_agent::data::repo_traits::offer_repo::MockOfferRepoTrait;
use negotiation_agent::entities::filters::NegotiationProcessFilter;
use negotiation_agent::entities::negotiation_process::{
    EditNegotiationProcessDto, NewNegotiationProcessDto,
};
use negotiation_agent::services::negotiation_process::NegotiationProcessServiceTrait;
use negotiation_agent::services::negotiation_process::service::NegotiationProcessService;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::RepoIntoErrors;

fn test_urn(n: u32) -> Urn {
    Urn::from_str(&format!(
        "urn:uuid:11111111-1111-1111-1111-1111111111{n:02}"
    ))
    .unwrap()
}

fn make_process_model(id: &Urn, tenant_id: &str) -> NegotiationProcessModel {
    NegotiationProcessModel {
        id: id.to_string(),
        user_id: tenant_id.to_string(),
        user_role: common::oauth::RolePath::root(),
        visibility: common::oauth::Visibility::Private,
        state: "REQUESTED".to_string(),
        state_attribute: None,
        associated_agent_peer: "urn:peer:1".to_string(),
        protocol: "DSP_2025_1".to_string(),
        callback_address: Some("http://localhost".to_string()),
        role: "CONSUMER".to_string(),
        properties: serde_json::json!({}),
        error_details: None,
        created_at: Utc::now().into(),
        updated_at: None,
    }
}

fn make_service(
    process_repo: MockNegotiationProcessRepoTrait,
    identifiers_repo: MockNegotiationIdentifierRepoTrait,
    messages_repo: MockNegotiationMessageRepoTrait,
    offers_repo: MockOfferRepoTrait,
    agreements_repo: MockAgreementRepoTrait,
) -> NegotiationProcessService {
    NegotiationProcessService::new(
        Arc::new(process_repo),
        Arc::new(identifiers_repo),
        Arc::new(messages_repo),
        Arc::new(offers_repo),
        Arc::new(agreements_repo),
    )
}

/// Reading a record of another tenant is not found: the lookup only searches the
/// caller's tenant.
#[tokio::test]
async fn get_one_foreign_tenant_returns_not_found() {
    let mut process_repo = MockNegotiationProcessRepoTrait::new();
    let id = test_urn(1);
    process_repo
        .expect_get_negotiation_process_by_id()
        .withf(move |scope, pid| *scope == OwnerScope::seeing(&TestUsers::alone("tenant-2")) && pid == &test_urn(1))
        .returning(|_, _| Ok(None));

    let svc = make_service(
        process_repo,
        MockNegotiationIdentifierRepoTrait::new(),
        MockNegotiationMessageRepoTrait::new(),
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
    let mut repo = MockNegotiationProcessRepoTrait::new();
    repo.expect_get_all_negotiation_processes()
        .withf(|scope, f, _, _| {
            *scope == OwnerScope::seeing(&TestUsers::alone("tenant-1"))
                && f.user_id.as_deref() == Some("tenant-foreign")
        })
        .returning(|_, _, _, _| Ok((vec![], Some(0))));
    let svc = make_service(
        repo,
        MockNegotiationIdentifierRepoTrait::new(),
        MockNegotiationMessageRepoTrait::new(),
        MockOfferRepoTrait::new(),
        MockAgreementRepoTrait::new(),
    );

    let filter = NegotiationProcessFilter {
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

/// Editing a record of another tenant is not found and changes nothing.
#[tokio::test]
async fn edit_foreign_tenant_returns_not_found_without_mutating() {
    let mut process_repo = MockNegotiationProcessRepoTrait::new();
    let id = test_urn(1);
    process_repo
        .expect_put_negotiation_process()
        .withf(move |scope, pid, _| *scope == OwnerScope::acting(&TestUsers::alone("tenant-2")) && pid == &test_urn(1))
        .returning(|_, _, _| {
            Err(NegotiationProcessRepoErrors::NegotiationProcessNotFound.into_errors())
        });

    let svc = make_service(
        process_repo,
        MockNegotiationIdentifierRepoTrait::new(),
        MockNegotiationMessageRepoTrait::new(),
        MockOfferRepoTrait::new(),
        MockAgreementRepoTrait::new(),
    );
    let cmd = EditNegotiationProcessDto {
        state: Some("TERMINATED".to_string()),
        state_attribute: None,
        properties: None,
        error_details: None,
        identifiers: None,
    };
    assert!(
        svc.edit(&TestUsers::user("tenant-2", "/admin/tenant-2"), &id, &cmd)
            .await
            .is_err()
    );
}

/// Deleting a record of another tenant is not found.
#[tokio::test]
async fn delete_foreign_tenant_returns_not_found() {
    let mut process_repo = MockNegotiationProcessRepoTrait::new();
    let id = test_urn(1);
    process_repo
        .expect_delete_negotiation_process()
        .withf(move |scope, pid| *scope == OwnerScope::acting(&TestUsers::alone("tenant-2")) && pid == &test_urn(1))
        .returning(|_, _| {
            Err(NegotiationProcessRepoErrors::NegotiationProcessNotFound.into_errors())
        });

    let svc = make_service(
        process_repo,
        MockNegotiationIdentifierRepoTrait::new(),
        MockNegotiationMessageRepoTrait::new(),
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
    let mut process_repo = MockNegotiationProcessRepoTrait::new();
    let id = test_urn(1);
    process_repo
        .expect_get_batch_negotiation_processes()
        .withf(move |scope, ids| *scope == OwnerScope::seeing(&TestUsers::alone("tenant-2")) && *ids == [test_urn(1)])
        .returning(|_, _| Ok(vec![]));

    let svc = make_service(
        process_repo,
        MockNegotiationIdentifierRepoTrait::new(),
        MockNegotiationMessageRepoTrait::new(),
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
    let mut process_repo = MockNegotiationProcessRepoTrait::new();
    let id = test_urn(1);
    let id_clone = id.clone();
    process_repo
        .expect_create_negotiation_process()
        .withf(|model| model.owner == TestUsers::owner("tenant-2"))
        .returning(move |model| Ok(make_process_model(&id_clone, &model.owner.user_id)));

    let svc = make_service(
        process_repo,
        MockNegotiationIdentifierRepoTrait::new(),
        MockNegotiationMessageRepoTrait::new(),
        MockOfferRepoTrait::new(),
        MockAgreementRepoTrait::new(),
    );
    let cmd = NewNegotiationProcessDto {
        id: Some(id),
        visibility: None,
        owner: Some(TestUsers::owner("tenant-1")),
        state: "REQUESTED".to_string(),
        state_attribute: None,
        associated_agent_peer: "urn:peer:1".to_string(),
        protocol: "DSP_2025_1".to_string(),
        callback_address: Some("http://localhost".to_string()),
        role: "CONSUMER".to_string(),
        properties: None,
        identifiers: None,
    };
    let view = svc
        .create(&TestUsers::user("tenant-2", "/admin/tenant-2"), &cmd)
        .await
        .unwrap();
    assert_eq!(view.inner.user_id, "tenant-2");
}
