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

//! TransferEventsService with a mocked repository: tenant isolation and role checks.

use std::str::FromStr;
use std::sync::Arc;

use chrono::Utc;
use common::batch_requests::BatchRequests;
use common::query::{Page, Sort};
use common::oauth::OwnerScope;
use common::test_utils::scopes::TestUsers;
use dataplane::data::factory_trait::MockDataplaneRepoTrait;
use dataplane::data::repo::transfer_event::{MockTransferEventRepo, TransferEventRepo};
use dataplane::data::sea_orm::orm::transfer_event::{LogLevel, Model as EventModel};
use dataplane::entities::filters::TransferEventFilter;
use dataplane::entities::transfer_events::NewTransferEventDto;
use dataplane::services::transfer_events::{TransferEventServiceTrait, TransferEventsService};
use urn::Urn;

fn test_urn(n: u32) -> Urn {
    Urn::from_str(&format!("urn:uuid:{n:08x}-0000-0000-0000-000000000000")).expect("valid URN")
}

fn make_event_model(n: u32, tenant: &str) -> EventModel {
    EventModel {
        id: test_urn(n).to_string(),
        user_id: tenant.to_string(),
        user_role: common::oauth::RolePath::root(),
        visibility: common::oauth::Visibility::Private,
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
        owner: Some(TestUsers::owner("tenant-default")),
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

/// Reading a record of another tenant is not found: the lookup only searches the
/// caller's tenant.
#[tokio::test]
async fn get_one_foreign_tenant_returns_not_found() {
    let mut repo = MockTransferEventRepo::new();
    repo.expect_get_transfer_event_by_id()
        .withf(|scope, id| *scope == OwnerScope::seeing(&TestUsers::alone("tenant-2")) && id == &test_urn(1))
        .returning(|_, _| Ok(None));

    let svc = make_events_svc(repo);
    let result = svc
        .get_one(&TestUsers::user("tenant-2", "/admin/tenant-2"), &test_urn(1))
        .await;
    assert!(result.is_err());
}

/// Narrowing a listing to another user stays within what the caller sees.
#[tokio::test]
async fn get_all_of_another_user_stays_within_what_the_caller_sees() {
    let mut repo = MockTransferEventRepo::new();
    repo.expect_get_all_transfer_events()
        .withf(|scope, f, _, _| {
            *scope == OwnerScope::seeing(&TestUsers::alone("tenant-1"))
                && f.user_id.as_deref() == Some("tenant-foreign")
        })
        .returning(|_, _, _, _| Ok(vec![]));
    repo.expect_count_transfer_events().returning(|_, _| Ok(0));
    let svc = make_events_svc(repo);

    let filter = TransferEventFilter {
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

/// Events of a transfer process are looked up in the caller's tenant.
#[tokio::test]
async fn get_by_process_id_passes_acting_tenant() {
    let mut repo = MockTransferEventRepo::new();
    repo.expect_get_all_transfer_events_by_process_id()
        .withf(|scope, pid| *scope == OwnerScope::seeing(&TestUsers::alone("tenant-2")) && pid == &test_urn(10))
        .returning(|_, _| Ok(vec![make_event_model(1, "tenant-2")]));

    let svc = make_events_svc(repo);
    let events = svc
        .get_by_process_id(&TestUsers::user("tenant-2", "/admin/tenant-2"), &test_urn(10))
        .await
        .unwrap();

    assert_eq!(events.len(), 1);
}

/// A batch read only returns records of the caller's tenant.
#[tokio::test]
async fn batch_filters_out_foreign_tenant_records() {
    let mut repo = MockTransferEventRepo::new();
    repo.expect_get_batch_transfer_events()
        .withf(|scope, ids| *scope == OwnerScope::seeing(&TestUsers::alone("tenant-2")) && ids == [test_urn(1)])
        .returning(|_, _| Ok(vec![]));

    let svc = make_events_svc(repo);
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
    let mut repo = MockTransferEventRepo::new();
    repo.expect_create_transfer_event()
        .withf(|cmd| cmd.owner == TestUsers::owner("tenant-2"))
        .returning(|cmd| {
            Ok(EventModel {
                id: test_urn(10).to_string(),
                user_id: cmd.owner.user_id.clone(),
                user_role: common::oauth::RolePath::root(),
                visibility: common::oauth::Visibility::Private,
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
    cmd.owner = Some(TestUsers::owner("tenant-foreign"));

    let created = svc
        .create(&TestUsers::user("tenant-2", "/admin/tenant-2"), &cmd)
        .await
        .unwrap();
    assert_eq!(created.inner.user_id, "tenant-2");
}

/// An admin without a tenant filter lists every tenant.
#[tokio::test]
async fn admin_can_query_all_or_specific_tenant() {
    let mut repo = MockTransferEventRepo::new();
    repo.expect_get_all_transfer_events()
        .withf(|scope, _, _, _| *scope == OwnerScope::All)
        .returning(|_, _, _, _| Ok(vec![make_event_model(1, "tenant-1")]));
    repo.expect_count_transfer_events().returning(|_, _| Ok(1));

    let svc = make_events_svc(repo);
    let paginated = svc
        .get_all(
            &TestUsers::user("admin-tenant", "/admin"),
            &TransferEventFilter::default(),
            &Page::default(),
            &Sort::default(),
        )
        .await
        .unwrap();

    assert_eq!(paginated.items.len(), 1);
}
