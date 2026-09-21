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

//! Unit and tenant isolation tests for DataplaneTransferService.

use std::str::FromStr;
use std::sync::Arc;

use chrono::Utc;
use common::auth::access::AccessScope;
use common::auth::claims::RbacRole;
use common::batch_requests::BatchRequests;
use common::query::{Page, Sort};
use dataplane::cache::NoopCache;
use dataplane::data::entities::dataplane_field::Model as FieldModel;
use dataplane::data::entities::dataplane_transfer_logs::{Model as LogModel, NewTransferLog};
use dataplane::data::entities::dataplane_transfers::{
    self as transfers_model, InteractionMode, Model as TransferModel, NewDataplaneTransfer,
    TransferRole, TransferState,
};
use dataplane::data::factory_trait::{DataplaneRepoTrait, MockDataplaneRepoTrait};
use dataplane::data::repo::dataplane_field::{
    DataplaneFieldRepoTrait, MockDataplaneFieldRepoTrait,
};
use dataplane::data::repo::dataplane_transfer::{
    DataplaneTransfersRepo, DataplaneTransfersRepoErrors, MockDataplaneTransfersRepo,
};
use dataplane::data::repo::dataplane_transfer_log::{
    DataplaneTransferLogsRepo, MockDataplaneTransferLogsRepo,
};
use dataplane::entities::dataplane_transfers::{
    DataplaneTransferDto, EditDataplaneTransferDto, NewDataplaneTransferDto,
};
use dataplane::entities::filters::DataplaneTransferFilter;
use dataplane::services::dataplane_transfers::{
    DataplaneTransferService, DataplaneTransferServiceTrait,
};
use urn::Urn;
use ymir::errors::RepoIntoErrors;

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

fn make_transfer_model(n: u32, tenant: &str) -> TransferModel {
    TransferModel {
        id: test_urn(n).to_string(),
        tenant_id: tenant.to_string(),
        transfer_process_id: format!("urn:uuid:{:08x}-0000-0000-0000-000000000001", n),
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
        tenant_id: "tenant-default".to_string(),
        transfer_process_id: "urn:uuid:00000010-0000-0000-0000-000000000001".to_string(),
        role: TransferRole::Provider,
        interaction_mode: InteractionMode::Pull,
        state: TransferState::Init,
        connector_instance_id: None,
        ingress_config: serde_json::json!({}),
        egress_config: serde_json::json!({}),
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
            tenant_id: cmd.tenant_id,
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

// ─────────────────────────────────────────────────────────────────────────────
// 6 Mandatory Multi-Tenancy Isolation Tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn get_one_foreign_tenant_returns_not_found() {
    let mut transfer_repo = MockDataplaneTransfersRepo::new();
    transfer_repo
        .expect_get_dataplane_transfers_by_id()
        .withf(|tenant, id| tenant == "tenant-2" && id == &test_urn(1))
        .returning(|_, _| Ok(None));

    let svc = make_test_svc(transfer_repo);
    let result = svc.get_one(&tenant_scope("tenant-2"), &test_urn(1)).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn get_all_foreign_tenant_query_rejected_with_forbidden() {
    let transfer_repo = MockDataplaneTransfersRepo::new();
    let svc = make_test_svc(transfer_repo);

    let mut filter = DataplaneTransferFilter::default();
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
    let mut transfer_repo = MockDataplaneTransfersRepo::new();
    transfer_repo
        .expect_put_dataplane_transfers()
        .withf(|tenant, id, _| tenant == "tenant-2" && id == &test_urn(1))
        .returning(|_, _, _| {
            Err(DataplaneTransfersRepoErrors::DataplaneTransferNotFound.into_errors())
        });

    let svc = make_test_svc(transfer_repo);
    let result = svc
        .edit(
            &tenant_scope("tenant-2"),
            &test_urn(1),
            &EditDataplaneTransferDto::default(),
        )
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn delete_foreign_tenant_returns_not_found() {
    let mut transfer_repo = MockDataplaneTransfersRepo::new();
    transfer_repo
        .expect_delete_dataplane_transfers()
        .withf(|tenant, id| tenant == "tenant-2" && id == &test_urn(1))
        .returning(|_, _| {
            Err(DataplaneTransfersRepoErrors::DataplaneTransferNotFound.into_errors())
        });

    let svc = make_test_svc(transfer_repo);
    let result = svc.delete(&tenant_scope("tenant-2"), &test_urn(1)).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn batch_filters_out_foreign_tenant_records() {
    let mut transfer_repo = MockDataplaneTransfersRepo::new();
    transfer_repo
        .expect_get_batch_dataplane_transfers()
        .withf(|tenant, ids| tenant == "tenant-2" && ids == &[test_urn(1)])
        .returning(|_, _| Ok(vec![]));

    let svc = make_test_svc(transfer_repo);
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
    let mut transfer_repo = MockDataplaneTransfersRepo::new();
    transfer_repo
        .expect_create_dataplane_transfers()
        .withf(|cmd| cmd.tenant_id == "tenant-2")
        .returning(|cmd| {
            Ok(TransferModel {
                id: test_urn(10).to_string(),
                tenant_id: cmd.tenant_id.clone(),
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
    cmd.tenant_id = "tenant-foreign".to_string();

    let created = svc.create(&tenant_scope("tenant-2"), &cmd).await.unwrap();
    assert_eq!(created.inner.tenant_id, "tenant-2");
}

// ─────────────────────────────────────────────────────────────────────────────
// Additional Query and Role Tests
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn get_by_process_id_foreign_tenant_returns_not_found() {
    let mut transfer_repo = MockDataplaneTransfersRepo::new();
    transfer_repo
        .expect_get_by_transfer_process_id()
        .withf(|tenant, id| tenant == "tenant-2" && id == &test_urn(1))
        .returning(|_, _| Ok(None));

    let svc = make_test_svc(transfer_repo);
    let result = svc
        .get_by_process_id(&tenant_scope("tenant-2"), &test_urn(1))
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn admin_can_query_all_tenants() {
    let mut transfer_repo = MockDataplaneTransfersRepo::new();
    transfer_repo
        .expect_get_all_dataplane_transfers()
        .withf(|filter, _, _| filter.tenant_id.is_none())
        .returning(|_, _, _| Ok(vec![make_transfer_model(1, "tenant-1")]));
    transfer_repo
        .expect_count_dataplane_transfers()
        .returning(|_| Ok(1));

    let svc = make_test_svc(transfer_repo);
    let paginated = svc
        .get_all(
            &admin_scope(),
            &DataplaneTransferFilter::default(),
            &Page::default(),
            &Sort::default(),
        )
        .await
        .unwrap();

    assert_eq!(paginated.items.len(), 1);
}

#[tokio::test]
async fn admin_can_filter_specific_tenant() {
    let mut transfer_repo = MockDataplaneTransfersRepo::new();
    transfer_repo
        .expect_get_all_dataplane_transfers()
        .withf(|filter, _, _| filter.tenant_id.as_deref() == Some("tenant-3"))
        .returning(|_, _, _| Ok(vec![make_transfer_model(3, "tenant-3")]));
    transfer_repo
        .expect_count_dataplane_transfers()
        .returning(|_| Ok(1));

    let svc = make_test_svc(transfer_repo);
    let mut filter = DataplaneTransferFilter::default();
    filter.tenant_id = Some("tenant-3".to_string());

    let paginated = svc
        .get_all(&admin_scope(), &filter, &Page::default(), &Sort::default())
        .await
        .unwrap();

    assert_eq!(paginated.items.len(), 1);
}

#[tokio::test]
async fn reader_cannot_create_or_mutate() {
    let transfer_repo = MockDataplaneTransfersRepo::new();
    let svc = make_test_svc(transfer_repo);

    let create_res = svc.create(&reader_scope("tenant-1"), &make_new_dto()).await;
    assert!(create_res.is_err());

    let edit_res = svc
        .edit(
            &reader_scope("tenant-1"),
            &test_urn(1),
            &EditDataplaneTransferDto::default(),
        )
        .await;
    assert!(edit_res.is_err());

    let delete_res = svc.delete(&reader_scope("tenant-1"), &test_urn(1)).await;
    assert!(delete_res.is_err());
}
