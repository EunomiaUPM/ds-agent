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

//! AgreementService with a mocked repository: tenant isolation and role checks.

use chrono::Utc;
use common::batch_requests::BatchRequests;
use common::query::{Page, Sort};
use common::oauth::OwnerScope;
use common::test_utils::scopes::TestUsers;
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
        "urn:uuid:44444444-4444-4444-4444-4444444444{n:02}"
    ))
    .unwrap()
}

fn make_agreement_model(id: &Urn, tenant_id: &str) -> AgreementModel {
    AgreementModel {
        id: id.to_string(),
        user_id: tenant_id.to_string(),
        user_role: common::oauth::RolePath::root(),
        visibility: common::oauth::Visibility::Private,
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

/// Reading a record of another tenant is not found: the lookup only searches the
/// caller's tenant.
#[tokio::test]
async fn get_one_foreign_tenant_returns_not_found() {
    let mut agreement_repo = MockAgreementRepoTrait::new();
    let id = test_urn(1);
    agreement_repo
        .expect_get_agreement_by_id()
        .withf(move |scope, aid| *scope == OwnerScope::seeing(&TestUsers::alone("tenant-2")) && aid == &test_urn(1))
        .returning(|_, _| Ok(None));

    let svc = make_service(agreement_repo);
    assert!(
        svc.get_one(&TestUsers::user("tenant-2", "/admin/tenant-2"), &id)
            .await
            .is_err()
    );
}

/// Narrowing a listing to another user stays within what the caller sees.
#[tokio::test]
async fn get_all_of_another_user_stays_within_what_the_caller_sees() {
    let mut repo = MockAgreementRepoTrait::new();
    repo.expect_get_all_agreements()
        .withf(|scope, f, _, _| {
            *scope == OwnerScope::seeing(&TestUsers::alone("tenant-1"))
                && f.user_id.as_deref() == Some("tenant-foreign")
        })
        .returning(|_, _, _, _| Ok((vec![], Some(0))));
    let svc = make_service(repo);

    let filter = AgreementFilter {
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
    let mut agreement_repo = MockAgreementRepoTrait::new();
    let id = test_urn(1);
    agreement_repo
        .expect_put_agreement()
        .withf(move |scope, aid, _| *scope == OwnerScope::acting(&TestUsers::alone("tenant-2")) && aid == &test_urn(1))
        .returning(|_, _, _| Err(AgreementRepoErrors::AgreementNotFound.into_errors()));

    let svc = make_service(agreement_repo);
    let cmd = EditAgreementDto {
        state: Some("FINALIZED".to_string()),
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
    let mut agreement_repo = MockAgreementRepoTrait::new();
    let id = test_urn(1);
    agreement_repo
        .expect_delete_agreement()
        .withf(move |scope, aid| *scope == OwnerScope::acting(&TestUsers::alone("tenant-2")) && aid == &test_urn(1))
        .returning(|_, _| Err(AgreementRepoErrors::AgreementNotFound.into_errors()));

    let svc = make_service(agreement_repo);
    assert!(
        svc.delete(&TestUsers::user("tenant-2", "/admin/tenant-2"), &id)
            .await
            .is_err()
    );
}

/// A batch read only returns records of the caller's tenant.
#[tokio::test]
async fn batch_filters_out_foreign_tenant_records() {
    let mut agreement_repo = MockAgreementRepoTrait::new();
    let id = test_urn(1);
    agreement_repo
        .expect_get_batch_agreements()
        .withf(move |scope, ids| *scope == OwnerScope::seeing(&TestUsers::alone("tenant-2")) && *ids == [test_urn(1)])
        .returning(|_, _| Ok(vec![]));

    let svc = make_service(agreement_repo);
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
    let mut agreement_repo = MockAgreementRepoTrait::new();
    let id = test_urn(1);
    let id_clone = id.clone();
    agreement_repo
        .expect_create_agreement()
        .withf(|model| model.owner == TestUsers::owner("tenant-2"))
        .returning(move |model| Ok(make_agreement_model(&id_clone, &model.owner.user_id)));

    let svc = make_service(agreement_repo);
    let cmd = NewAgreementDto {
        id: Some(id),
        visibility: None,
        owner: Some(TestUsers::owner("tenant-1")),
        negotiation_agent_process_id: test_urn(9),
        negotiation_agent_message_id: test_urn(8),
        consumer_participant_id: "urn:uuid:consumer-1".to_string(),
        provider_participant_id: "urn:uuid:provider-1".to_string(),
        agreement_content: serde_json::json!({}),
        target: test_urn(7),
    };
    let view = svc
        .create(&TestUsers::user("tenant-2", "/admin/tenant-2"), &cmd)
        .await
        .unwrap();
    assert_eq!(view.inner.user_id, "tenant-2");
}
