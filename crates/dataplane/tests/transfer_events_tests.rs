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

//! Unit and tenant isolation tests for TransferEventsService.

use std::str::FromStr;
use std::sync::Arc;

use chrono::Utc;
use common::auth::access::AccessScope;
use common::auth::claims::RbacRole;
use common::batch_requests::BatchRequests;
use common::query::{Page, Sort};
use dataplane::data::factory_trait::MockDataplaneRepoTrait;
use dataplane::data::repo::transfer_event::{
    MockTransferEventRepo, TransferEventRepo, TransferEventRepoErrors,
};
use dataplane::data::sea_orm::orm::transfer_event::{
    LogLevel, Model as EventModel, NewTransferEvent,
};
use dataplane::entities::filters::TransferEventFilter;
use dataplane::entities::transfer_events::NewTransferEventDto;
use dataplane::services::transfer_events::{TransferEventServiceTrait, TransferEventsService};
use urn::Urn;

fn admin_scope() -> AccessScope {
    AccessScope::from_role(RbacRole::Admin, "admin-tenant")
}

fn tenant_scope(tenant: &str) -> AccessScope {
    AccessScope::from_role(RbacRole::Owner, tenant)
}

fn reader_scope(tenant: &str) -> AccessScope {
    AccessScope::from_role(RbacRole::Reader, tenant)
}

fn test_urn(n: u32) -> Urn {
    Urn::from_str(&format!("urn:uuid:{:08x}-0000-0000-0000-000000000000", n)).expect("valid URN")
}

fn make_event_model(n: u32, tenant: &str) -> EventModel {
    EventModel {
        id: test_urn(n).to_string(),
        tenant_id: tenant.to_string(),
        transfer_id: test_urn(1).to_string(),
        level: LogLevel::Info,
        component: "Driver".to_string(),
        message: "Test event".to_string(),
        data: None,
        created_at: Utc::now().into(),
    }
}

fn make_new_event_dto() -> NewTransferEventDto {
    NewTransferEventDto {
        tenant_id: "tenant-default".to_string(),
        transfer_id: test_urn(1),
        level: LogLevel::Info,
        component: "Driver".to_string(),
        message: "New test event".to_string(),
        data: None,
    }
}

fn make_events_svc(repo: MockTransferEventRepo) -> TransferEventsService {
    let mut factory = MockDataplaneRepoTrait::new();
    let repo_arc: Arc<dyn TransferEventRepo> = Arc::new(repo);
    factory
        .expect_get_transfer_events_repo()
        .return_const(repo_arc);

    TransferEventsService::new(Arc::new(factory))
}

// ─────────────────────────────────────────────────────────────────────────────
// Isolation and Authorization Tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn get_one_foreign_tenant_returns_not_found() {
    let mut repo = MockTransferEventRepo::new();
    repo.expect_get_transfer_event_by_id()
        .withf(|tenant, id| tenant.as_deref() == Some("tenant-2") && id == &test_urn(1))
        .returning(|_, _| Ok(None));

    let svc = make_events_svc(repo);
    let result = svc.get_one(&tenant_scope("tenant-2"), &test_urn(1)).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn get_all_foreign_tenant_query_rejected_with_forbidden() {
    let repo = MockTransferEventRepo::new();
    let svc = make_events_svc(repo);

    let mut filter = TransferEventFilter::default();
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
async fn get_by_process_id_passes_acting_tenant() {
    let mut repo = MockTransferEventRepo::new();
    repo.expect_get_all_transfer_events_by_process_id()
        .withf(|tenant, pid| tenant.as_deref() == Some("tenant-2") && pid == &test_urn(10))
        .returning(|_, _| Ok(vec![make_event_model(1, "tenant-2")]));

    let svc = make_events_svc(repo);
    let events = svc
        .get_by_process_id(&tenant_scope("tenant-2"), &test_urn(10))
        .await
        .unwrap();

    assert_eq!(events.len(), 1);
}

#[tokio::test]
async fn batch_filters_out_foreign_tenant_records() {
    let mut repo = MockTransferEventRepo::new();
    repo.expect_get_batch_transfer_events()
        .withf(|tenant, ids| tenant.as_deref() == Some("tenant-2") && ids == &[test_urn(1)])
        .returning(|_, _| Ok(vec![]));

    let svc = make_events_svc(repo);
    let views = svc
        .batch(
            &tenant_scope("tenant-2"),
            &BatchRequests {
                ids: vec![test_urn(1)],
            },
        )
        .await
        .unwrap();

    assert!(views.is_empty());
}

#[tokio::test]
async fn create_forces_caller_tenant_for_non_admin() {
    let mut repo = MockTransferEventRepo::new();
    repo.expect_create_transfer_event()
        .withf(|cmd| cmd.tenant_id == "tenant-2")
        .returning(|cmd| {
            Ok(EventModel {
                id: test_urn(10).to_string(),
                tenant_id: cmd.tenant_id.clone(),
                transfer_id: cmd.transfer_id.clone(),
                level: cmd.level.clone(),
                component: cmd.component.clone(),
                message: cmd.message.clone(),
                data: cmd.data.clone(),
                created_at: Utc::now().into(),
            })
        });

    let svc = make_events_svc(repo);
    let mut cmd = make_new_event_dto();
    cmd.tenant_id = "tenant-foreign".to_string();

    let created = svc.create(&tenant_scope("tenant-2"), &cmd).await.unwrap();
    assert_eq!(created.inner.tenant_id, "tenant-2");
}

#[tokio::test]
async fn admin_can_query_all_or_specific_tenant() {
    let mut repo = MockTransferEventRepo::new();
    repo.expect_get_all_transfer_events()
        .withf(|filter, _, _| filter.tenant_id.is_none())
        .returning(|_, _, _| Ok(vec![make_event_model(1, "tenant-1")]));
    repo.expect_count_transfer_events().returning(|_| Ok(1));

    let svc = make_events_svc(repo);
    let paginated = svc
        .get_all(
            &admin_scope(),
            &TransferEventFilter::default(),
            &Page::default(),
            &Sort::default(),
        )
        .await
        .unwrap();

    assert_eq!(paginated.items.len(), 1);
}

#[tokio::test]
async fn reader_cannot_create_event() {
    let repo = MockTransferEventRepo::new();
    let svc = make_events_svc(repo);

    let result = svc
        .create(&reader_scope("tenant-1"), &make_new_event_dto())
        .await;
    assert!(result.is_err());
}
