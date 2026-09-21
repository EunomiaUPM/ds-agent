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

//! Unit and tenant isolation tests for DataplaneTransferLogsService.

use std::str::FromStr;
use std::sync::Arc;

use chrono::Utc;
use common::auth::access::AccessScope;
use common::auth::claims::RbacRole;
use dataplane::data::entities::dataplane_transfer_logs::Model as LogModel;
use dataplane::data::entities::dataplane_transfers::TransferState;
use dataplane::data::factory_trait::MockDataplaneRepoTrait;
use dataplane::data::repo::dataplane_field::MockDataplaneFieldRepoTrait;
use dataplane::data::repo::dataplane_transfer::MockDataplaneTransfersRepo;
use dataplane::data::repo::dataplane_transfer_log::{
    DataplaneTransferLogsRepo, MockDataplaneTransferLogsRepo,
};
use dataplane::data::repo::transfer_event::MockTransferEventRepo;
use dataplane::services::dataplane_transfer_logs::{
    DataplaneTransferLogServiceTrait, DataplaneTransferLogsService,
};
use urn::Urn;

fn tenant_scope(tenant: &str) -> AccessScope {
    AccessScope::from_role(RbacRole::Owner, tenant)
}

fn reader_scope(tenant: &str) -> AccessScope {
    AccessScope::from_role(RbacRole::Reader, tenant)
}

fn test_urn(n: u32) -> Urn {
    Urn::from_str(&format!("urn:uuid:{:08x}-0000-0000-0000-000000000000", n)).expect("valid URN")
}

fn make_log_model(n: u32, tenant: &str) -> LogModel {
    LogModel {
        id: test_urn(n).to_string(),
        tenant_id: tenant.to_string(),
        dataplane_process_id: test_urn(1).to_string(),
        previous_state: None,
        new_state: TransferState::Init,
        trigger: "Test".to_string(),
        reason: None,
        created_at: Utc::now().into(),
    }
}

fn make_logs_svc(logs_repo: MockDataplaneTransferLogsRepo) -> DataplaneTransferLogsService {
    let mut factory = MockDataplaneRepoTrait::new();
    let logs_arc: Arc<dyn DataplaneTransferLogsRepo> = Arc::new(logs_repo);
    factory
        .expect_get_dataplane_transfer_logs_repo()
        .return_const(logs_arc);

    DataplaneTransferLogsService::new(Arc::new(factory))
}

#[tokio::test]
async fn get_logs_passes_acting_tenant_to_repo() {
    let mut logs_repo = MockDataplaneTransferLogsRepo::new();
    logs_repo
        .expect_get_transfer_logs_by_dataplane_process_id()
        .withf(|tenant, id| tenant == "tenant-2" && id == &test_urn(1))
        .returning(|_, _| Ok(vec![make_log_model(10, "tenant-2")]));

    let svc = make_logs_svc(logs_repo);
    let logs = svc
        .get_transfer_logs_by_dataplane_process_id(&tenant_scope("tenant-2"), &test_urn(1))
        .await
        .unwrap();

    assert_eq!(logs.len(), 1);
}

#[tokio::test]
async fn get_logs_reader_role_is_permitted() {
    let mut logs_repo = MockDataplaneTransferLogsRepo::new();
    logs_repo
        .expect_get_transfer_logs_by_dataplane_process_id()
        .withf(|tenant, id| tenant == "tenant-1" && id == &test_urn(1))
        .returning(|_, _| Ok(vec![]));

    let svc = make_logs_svc(logs_repo);
    let result = svc
        .get_transfer_logs_by_dataplane_process_id(&reader_scope("tenant-1"), &test_urn(1))
        .await;

    assert!(result.is_ok());
}
