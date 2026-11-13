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

//! DataplaneTransferService with mocked repositories: tenant isolation and role checks.

use std::str::FromStr;
use std::sync::{Arc, Mutex};

use chrono::Utc;
use events::{EventEnvelope, EventSeverity, EventSinkTrait};
use ymir::batch_requests::BatchRequests;
use ymir::query::{Page, Sort};
use ymir::oauth::OwnerScope;
use common::test_utils::scopes::TestUsers;
use dataplane::cache::NoopCache;
use dataplane::data::factory_trait::MockDataplaneRepoTrait;
use dataplane::data::repo::dataplane_field::{
    DataplaneFieldRepoTrait, MockDataplaneFieldRepoTrait,
};
use dataplane::data::repo::dataplane_transfer::{
    DataplaneTransfersRepo, DataplaneTransfersRepoErrors, MockDataplaneTransfersRepo,
};
use dataplane::data::repo::dataplane_transfer_log::{
    DataplaneTransferLogsRepo, MockDataplaneTransferLogsRepo,
};
use dataplane::data::sea_orm::orm::dataplane_transfer_logs::Model as LogModel;
use dataplane::data::sea_orm::orm::dataplane_transfers::{
    InteractionMode, Model as TransferModel, TransferRole, TransferState,
};
use dataplane::entities::dataplane_transfers::{EditDataplaneTransferDto, NewDataplaneTransferDto};
use dataplane::entities::filters::DataplaneTransferFilter;
use dataplane::services::dataplane_transfers::{
    DataplaneTransferService, DataplaneTransferServiceTrait,
};
use urn::Urn;
use ymir::errors::RepoIntoErrors;

fn test_urn(n: u32) -> Urn {
    Urn::from_str(&format!("urn:uuid:{n:08x}-0000-0000-0000-000000000000")).expect("valid URN")
}

fn make_transfer_model(n: u32, tenant: &str) -> TransferModel {
    TransferModel {
        id: test_urn(n).to_string(),
        user_id: tenant.to_string(),
        user_role: ymir::types::oauth::RolePath::root(),
        visibility: ymir::types::participants::Visibility::Private,
        transfer_process_id: format!("urn:uuid:{n:08x}-0000-0000-0000-000000000001"),
        role: TransferRole::Provider,
        interaction_mode: InteractionMode::Pull,
        state: TransferState::Init,
        connector_instance_id: None,
        ingress_config: serde_json::json!({}),
        egress_config: serde_json::json!({}),
        flow_control: None,
        created_at: Utc::now().into(),
        updated_at: None,
    }
}

fn make_new_dto() -> NewDataplaneTransferDto {
    NewDataplaneTransferDto {
        id: Some(test_urn(10)),
        owner: Some(TestUsers::owner("tenant-default")),
        transfer_process_id: "urn:uuid:00000010-0000-0000-0000-000000000001".to_string(),
        role: TransferRole::Provider,
        interaction_mode: InteractionMode::Pull,
        state: TransferState::Init,
        connector_instance_id: None,
        ingress_config: serde_json::json!({}),
        egress_config: serde_json::json!({}),
        reason: None,
    }
}

fn default_field_repo() -> MockDataplaneFieldRepoTrait {
    let mut repo = MockDataplaneFieldRepoTrait::new();
    repo.expect_get_all_dataplane_fields_by_process_id()
        .returning(|_, _| Ok(vec![]));
    repo
}

fn default_logs_repo() -> MockDataplaneTransferLogsRepo {
    let mut repo = MockDataplaneTransferLogsRepo::new();
    repo.expect_get_transfer_logs_by_dataplane_process_id()
        .returning(|_, _| Ok(vec![]));
    repo.expect_create_log().returning(|cmd| {
        Ok(LogModel {
            id: test_urn(999).to_string(),
            user_id: cmd.owner.user_id.clone(),
            user_role: ymir::types::oauth::RolePath::root(),
            visibility: ymir::types::participants::Visibility::Private,
            dataplane_process_id: cmd.dataplane_process_id,
            previous_state: None,
            new_state: cmd.new_state,
            trigger: cmd.trigger,
            reason: cmd.reason,
            created_at: Utc::now().into(),
        })
    });
    repo
}

fn make_test_svc(transfer_repo: MockDataplaneTransfersRepo) -> DataplaneTransferService {
    let mut factory = MockDataplaneRepoTrait::new();
    let transfer_arc: Arc<dyn DataplaneTransfersRepo> = Arc::new(transfer_repo);
    let field_arc: Arc<dyn DataplaneFieldRepoTrait> = Arc::new(default_field_repo());
    let logs_arc: Arc<dyn DataplaneTransferLogsRepo> = Arc::new(default_logs_repo());

    factory
        .expect_get_dataplane_transfers_repo()
        .return_const(transfer_arc);
    factory
        .expect_get_dataplane_fields_repo()
        .return_const(field_arc);
    factory
        .expect_get_dataplane_transfer_logs_repo()
        .return_const(logs_arc);

    DataplaneTransferService::new(Arc::new(factory), Arc::new(NoopCache::new()))
}

/// Reading a record of another tenant is not found: the lookup only searches the
/// caller's tenant.
#[tokio::test]
async fn get_one_foreign_tenant_returns_not_found() {
    let mut transfer_repo = MockDataplaneTransfersRepo::new();
    transfer_repo
        .expect_get_dataplane_transfers_by_id()
        .withf(|scope, id| *scope == OwnerScope::seeing(&TestUsers::alone("tenant-2")) && id == &test_urn(1))
        .returning(|_, _| Ok(None));

    let svc = make_test_svc(transfer_repo);
    let result = svc
        .get_one(&TestUsers::user("tenant-2", "/admin/tenant-2"), &test_urn(1))
        .await;
    assert!(result.is_err());
}

/// Narrowing a listing to another user stays within what the caller sees.
#[tokio::test]
async fn get_all_of_another_user_stays_within_what_the_caller_sees() {
    let mut transfer_repo = MockDataplaneTransfersRepo::new();
    transfer_repo.expect_get_all_dataplane_transfers()
        .withf(|scope, f, _, _| {
            *scope == OwnerScope::seeing(&TestUsers::alone("tenant-1"))
                && f.user_id.as_deref() == Some("tenant-foreign")
        })
        .returning(|_, _, _, _| Ok(vec![]));
    transfer_repo.expect_count_dataplane_transfers().returning(|_, _| Ok(0));
    let svc = make_test_svc(transfer_repo);

    let filter = DataplaneTransferFilter {
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
    let mut transfer_repo = MockDataplaneTransfersRepo::new();
    transfer_repo
        .expect_get_dataplane_transfers_by_id()
        .withf(|scope, id| *scope == OwnerScope::acting(&TestUsers::alone("tenant-2")) && id == &test_urn(1))
        .returning(|_, _| Ok(None));
    transfer_repo.expect_put_dataplane_transfers().never();

    let svc = make_test_svc(transfer_repo);
    let result = svc
        .edit(
            &TestUsers::user("tenant-2", "/admin/tenant-2"),
            &test_urn(1),
            &EditDataplaneTransferDto::default(),
        )
        .await;
    assert!(result.is_err());
}

/// Deleting a record of another tenant is not found.
#[tokio::test]
async fn delete_foreign_tenant_returns_not_found() {
    let mut transfer_repo = MockDataplaneTransfersRepo::new();
    transfer_repo
        .expect_delete_dataplane_transfers()
        .withf(|scope, id| *scope == OwnerScope::acting(&TestUsers::alone("tenant-2")) && id == &test_urn(1))
        .returning(|_, _| {
            Err(DataplaneTransfersRepoErrors::DataplaneTransferNotFound.into_errors())
        });

    let svc = make_test_svc(transfer_repo);
    let result = svc
        .delete(&TestUsers::user("tenant-2", "/admin/tenant-2"), &test_urn(1))
        .await;
    assert!(result.is_err());
}

/// A batch read only returns records of the caller's tenant.
#[tokio::test]
async fn batch_filters_out_foreign_tenant_records() {
    let mut transfer_repo = MockDataplaneTransfersRepo::new();
    transfer_repo
        .expect_get_batch_dataplane_transfers()
        .withf(|scope, ids| *scope == OwnerScope::seeing(&TestUsers::alone("tenant-2")) && ids == [test_urn(1)])
        .returning(|_, _| Ok(vec![]));

    let svc = make_test_svc(transfer_repo);
    let views = svc
        .batch(
            &TestUsers::user("tenant-2", "/admin/tenant-2"),
            &BatchRequests {
                ids: vec![test_urn(1)],
            },
        )
        .await
        .unwrap();
    assert!(views.is_empty());
}

/// A non-admin always creates in its own tenant, whatever the DTO says.
#[tokio::test]
async fn create_forces_caller_tenant_for_non_admin() {
    let mut transfer_repo = MockDataplaneTransfersRepo::new();
    transfer_repo
        .expect_create_dataplane_transfers()
        .withf(|cmd| cmd.owner == TestUsers::owner("tenant-2"))
        .returning(|cmd| {
            Ok(TransferModel {
                id: test_urn(10).to_string(),
                user_id: cmd.owner.user_id.clone(),
                user_role: ymir::types::oauth::RolePath::root(),
                visibility: ymir::types::participants::Visibility::Private,
                transfer_process_id: cmd.transfer_process_id.clone(),
                role: cmd.role.clone(),
                interaction_mode: cmd.interaction_mode.clone(),
                state: cmd.state.clone(),
                connector_instance_id: None,
                ingress_config: serde_json::json!({}),
                egress_config: serde_json::json!({}),
                flow_control: None,
                created_at: Utc::now().into(),
                updated_at: None,
            })
        });

    let svc = make_test_svc(transfer_repo);
    let mut cmd = make_new_dto();
    cmd.owner = Some(TestUsers::owner("tenant-foreign"));

    let created = svc
        .create(&TestUsers::user("tenant-2", "/admin/tenant-2"), &cmd)
        .await
        .unwrap();
    assert_eq!(created.inner.user_id, "tenant-2");
}

/// Looking up by transfer process in another tenant is not found.
#[tokio::test]
async fn get_by_process_id_foreign_tenant_returns_not_found() {
    let mut transfer_repo = MockDataplaneTransfersRepo::new();
    transfer_repo
        .expect_get_by_transfer_process_id()
        .withf(|scope, id| *scope == OwnerScope::seeing(&TestUsers::alone("tenant-2")) && id == &test_urn(1))
        .returning(|_, _| Ok(None));

    let svc = make_test_svc(transfer_repo);
    let result = svc
        .get_by_process_id(&TestUsers::user("tenant-2", "/admin/tenant-2"), &test_urn(1))
        .await;
    assert!(result.is_err());
}

/// An admin without a tenant filter lists every tenant.
#[tokio::test]
async fn admin_can_query_all_tenants() {
    let mut transfer_repo = MockDataplaneTransfersRepo::new();
    transfer_repo
        .expect_get_all_dataplane_transfers()
        .withf(|scope, _, _, _| *scope == OwnerScope::All)
        .returning(|_, _, _, _| Ok(vec![make_transfer_model(1, "tenant-1")]));
    transfer_repo
        .expect_count_dataplane_transfers()
        .returning(|_, _| Ok(1));

    let svc = make_test_svc(transfer_repo);
    let paginated = svc
        .get_all(
            &TestUsers::user("admin-tenant", "/admin"),
            &DataplaneTransferFilter::default(),
            &Page::default(),
            &Sort::default(),
        )
        .await
        .unwrap();

    assert_eq!(paginated.items.len(), 1);
}

/// An admin may filter on one specific tenant.
#[tokio::test]
async fn admin_can_filter_specific_tenant() {
    let mut transfer_repo = MockDataplaneTransfersRepo::new();
    transfer_repo
        .expect_get_all_dataplane_transfers()
        .withf(|scope, filter, _, _| *scope == OwnerScope::All && filter.user_id.as_deref() == Some("tenant-3"))
        .returning(|_, _, _, _| Ok(vec![make_transfer_model(3, "tenant-3")]));
    transfer_repo
        .expect_count_dataplane_transfers()
        .returning(|_, _| Ok(1));

    let svc = make_test_svc(transfer_repo);
    let filter = DataplaneTransferFilter {
        user_id: Some("tenant-3".to_string()),
        ..Default::default()
    };

    let paginated = svc
        .get_all(
            &TestUsers::user("admin-tenant", "/admin"),
            &filter,
            &Page::default(),
            &Sort::default(),
        )
        .await
        .unwrap();

    assert_eq!(paginated.items.len(), 1);
}

struct CaptureSink(Mutex<Vec<EventEnvelope>>);

impl CaptureSink {
    fn new() -> Arc<Self> {
        Arc::new(Self(Mutex::new(Vec::new())))
    }
}

#[async_trait::async_trait]
impl EventSinkTrait for CaptureSink {
    async fn emit_envelope(&self, envelope: EventEnvelope) -> ymir::errors::Outcome<()> {
        self.0.lock().unwrap().push(envelope);
        Ok(())
    }
}

/// Creating and editing a transfer normalizes reasons into transfer logs and transition events.
#[tokio::test]
async fn create_and_edit_propagate_normalized_reason_to_transfer_logs_and_events() {
    let recorded_logs = Arc::new(Mutex::new(Vec::new()));
    let captured_logs = recorded_logs.clone();

    let mut transfer_repo = MockDataplaneTransfersRepo::new();
    transfer_repo
        .expect_create_dataplane_transfers()
        .times(1)
        .returning(|_| Ok(make_transfer_model(40, "tenant-reason")));
    let current_model = make_transfer_model(40, "tenant-reason");
    transfer_repo
        .expect_get_dataplane_transfers_by_id()
        .times(3)
        .returning(move |_, _| Ok(Some(current_model.clone())));
    transfer_repo
        .expect_put_dataplane_transfers()
        .times(3)
        .returning(|_, _, cmd| {
            let mut updated = make_transfer_model(40, "tenant-reason");
            if let Some(st) = &cmd.state {
                updated.state = st.clone();
            }
            Ok(updated)
        });

    let mut logs_repo = MockDataplaneTransferLogsRepo::new();
    logs_repo
        .expect_get_transfer_logs_by_dataplane_process_id()
        .returning(|_, _| Ok(vec![]));
    logs_repo.expect_create_log().returning(move |cmd| {
        captured_logs
            .lock()
            .unwrap()
            .push((cmd.previous_state.clone(), cmd.new_state.clone(), cmd.reason.clone()));
        Ok(LogModel {
            id: test_urn(900).to_string(),
            user_id: cmd.owner.user_id,
            user_role: ymir::types::oauth::RolePath::root(),
            visibility: ymir::types::participants::Visibility::Private,
            dataplane_process_id: cmd.dataplane_process_id,
            previous_state: cmd.previous_state,
            new_state: cmd.new_state,
            trigger: cmd.trigger,
            reason: cmd.reason,
            created_at: Utc::now().into(),
        })
    });

    let mut factory = MockDataplaneRepoTrait::new();
    let transfer_arc: Arc<dyn DataplaneTransfersRepo> = Arc::new(transfer_repo);
    let field_arc: Arc<dyn DataplaneFieldRepoTrait> = Arc::new(default_field_repo());
    let logs_arc: Arc<dyn DataplaneTransferLogsRepo> = Arc::new(logs_repo);
    factory
        .expect_get_dataplane_transfers_repo()
        .return_const(transfer_arc);
    factory
        .expect_get_dataplane_fields_repo()
        .return_const(field_arc);
    factory
        .expect_get_dataplane_transfer_logs_repo()
        .return_const(logs_arc);

    let sink = CaptureSink::new();
    let svc = DataplaneTransferService::new(Arc::new(factory), Arc::new(NoopCache::new()))
        .with_events(sink.clone());
    let owner = TestUsers::user("tenant-reason", "/admin/tenant-reason");

    let mut create_cmd = make_new_dto();
    create_cmd.id = Some(test_urn(40));
    create_cmd.owner = Some(TestUsers::owner("tenant-reason"));
    create_cmd.reason = Some("  Initialized provider pull  ".to_string());
    svc.create(&owner, &create_cmd).await.unwrap();

    // Unchanged state must not write a transfer log even if a reason is provided.
    svc.edit(
        &owner,
        &test_urn(40),
        &EditDataplaneTransferDto {
            state: Some(TransferState::Init),
            reason: Some("ignored".to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    svc.edit(
        &owner,
        &test_urn(40),
        &EditDataplaneTransferDto {
            state: Some(TransferState::Started),
            reason: Some("Transfer active".to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let term_reason = "Transfer terminated (DSP peer: TransferTerminationMessage) [code=DATAPLANE_FORBIDDEN_TARGET]: private IP blocked";
    svc.edit(
        &owner,
        &test_urn(40),
        &EditDataplaneTransferDto {
            state: Some(TransferState::Terminated),
            reason: Some(term_reason.to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let logs = recorded_logs.lock().unwrap().clone();
    assert_eq!(
        logs,
        vec![
            (None, TransferState::Init, Some("Initialized provider pull".to_string())),
            (
                Some(TransferState::Init),
                TransferState::Started,
                Some("Transfer active".to_string())
            ),
            (
                Some(TransferState::Init),
                TransferState::Terminated,
                Some(term_reason.to_string())
            ),
        ]
    );

    let events = sink.0.lock().unwrap().clone();
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].severity, EventSeverity::Info);
    assert!(events[0].payload["message"]
        .as_str()
        .unwrap()
        .contains("reason=Initialized provider pull"));
    assert!(events[0].payload.get("error_reason").is_none());
    assert_eq!(events[1].severity, EventSeverity::Info);
    assert_eq!(events[2].severity, EventSeverity::Error);
    assert_eq!(events[2].payload["error_reason"].as_str(), Some(term_reason));
}
