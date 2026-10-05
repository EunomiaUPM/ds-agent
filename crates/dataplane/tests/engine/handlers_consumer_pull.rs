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

//! DataplaneHandlerConsumerPull: each state step with a mocked transfer service; pull has no
//! subscriber, so subscribing steps are no-ops.

use dataplane::engine::dataplane_manager::dataplane_commands::{
    DataplaneCommandStateMachine, DataplaneInitCommandTypes,
};
use dataplane::engine::dataplane_manager::dataplane_context::DataplaneContext;
use std::sync::Arc;

use common::test_utils::config_fixtures::transfer_config_fixture;
use dataplane::data::sea_orm::orm::dataplane_transfers;
use dataplane::engine::dataplane_manager::dataplane_commands::DataplaneInitCommandDirection;
use dataplane::engine::dataplane_manager::dataplane_handlers_consumer_pull::DataplaneHandlerConsumerPull;
use dataplane::entities::dataplane_transfers::{
    DataplaneTransferDto, InteractionMode, TransferRole, TransferState,
};
use dataplane::services::dataplane_transfers::MockDataplaneTransferServiceTrait;
use dataplane::DataplaneAddress;
use serde_json::json;
use std::str::FromStr;
use urn::Urn;

use connector::MockConnectorInstanceFacadeTrait as MockConnectorInstance;

const DP_URN: &str = "urn:dataplane-transfer:1";
const TP_URN: &str = "urn:transfer-process:1";

fn forward_address_fixture() -> DataplaneAddress {
    DataplaneAddress {
        endpoint_type: "HTTP".to_string(),
        endpoint: "http://provider-endpoint.com/data".to_string(),
        authorization_type: Some("Bearer".to_string()),
        authorization: Some("token-abc".to_string()),
    }
}

fn dto(state: TransferState) -> DataplaneTransferDto {
    DataplaneTransferDto {
        inner: dataplane_transfers::Model {
            user_id: "tenant-1".to_string(),
            user_role: common::oauth::RolePath::root(),
            visibility: common::oauth::Visibility::Private,
            id: DP_URN.to_string(),
            transfer_process_id: TP_URN.to_string(),
            role: TransferRole::Consumer,
            interaction_mode: InteractionMode::Pull,
            state,
            connector_instance_id: None,
            ingress_config: json!({}),
            egress_config: json!({}),
            flow_control: Some(json!({})),
            created_at: chrono::Utc::now().into(),
            updated_at: None,
        },
        fields: Default::default(),
        logs: vec![],
    }
}

fn handler(
    entity: Arc<dyn dataplane::services::dataplane_transfers::DataplaneTransferServiceTrait>,
) -> DataplaneHandlerConsumerPull {
    DataplaneHandlerConsumerPull::new(
        entity,
        Arc::new(MockConnectorInstance::new()),
        transfer_config_fixture(),
        None,
    )
}

async fn init_context(
    entity: Arc<dyn dataplane::services::dataplane_transfers::DataplaneTransferServiceTrait>,
) -> DataplaneContext {
    DataplaneContext::from_init(
        entity,
        Arc::new(MockConnectorInstance::new()),
        transfer_config_fixture(),
        DataplaneInitCommandTypes::AsConsumer {
            owner: common::test_utils::scopes::TestUsers::owner("tenant-1"),
            transfer_process_id: Urn::from_str(TP_URN).unwrap(),
            direction: DataplaneInitCommandDirection::Pull {
                data_address: Some(forward_address_fixture()),
            },
        },
    )
    .await
    .unwrap()
}

fn expect_create(mock: &mut MockDataplaneTransferServiceTrait) {
    mock.expect_create()
        .times(1)
        .returning(|_, _| Ok(dto(TransferState::Init)));
}

fn expect_put(mock: &mut MockDataplaneTransferServiceTrait, expected: TransferState) {
    let check = expected.clone();
    mock.expect_edit()
        .times(1)
        .withf(move |_, _, edit| edit.state == Some(check.clone()))
        .returning(move |_, _, _| Ok(dto(expected.clone())));
}

/// set_init leaves the transfer in Init without touching the database.
#[tokio::test]
async fn set_init_is_noop_for_consumer_pull() {
    let mut mock = MockDataplaneTransferServiceTrait::new();
    expect_create(&mut mock);
    // no put expectations — set_init does not touch the DB

    let entity: Arc<dyn dataplane::services::dataplane_transfers::DataplaneTransferServiceTrait> =
        Arc::new(mock);
    let context = init_context(entity.clone()).await;
    let result = handler(entity).set_init(context).await;

    assert!(result.is_ok());
    let ctx = result.unwrap();
    assert_eq!(ctx.dataplane_process().inner.state, TransferState::Init);
    assert!(ctx.connector_instance().is_none());
}

/// set_configuring runs the whole consumer-pull flow in one go: it configures the proxy,
/// authenticates, and moves through ready to started.
#[tokio::test]
async fn set_configuring_builds_driver_and_proxy() {
    let mut mock = MockDataplaneTransferServiceTrait::new();
    expect_create(&mut mock);
    expect_put(&mut mock, TransferState::Configuring);
    expect_put(&mut mock, TransferState::Auth);
    expect_put(&mut mock, TransferState::Ready);
    expect_put(&mut mock, TransferState::Started);

    let entity: Arc<dyn dataplane::services::dataplane_transfers::DataplaneTransferServiceTrait> =
        Arc::new(mock);
    let context = init_context(entity.clone()).await;
    let result = handler(entity.clone()).set_configuring(context).await;

    assert!(result.is_ok());
    let ctx = result.unwrap();
    assert_eq!(ctx.dataplane_process().inner.state, TransferState::Started);
    assert!(
        ctx.driver().is_some(),
        "driver must be set after configuring"
    );
    assert!(
        ctx.proxy().is_some(),
        "proxy must be built after configuring"
    );
    assert!(ctx.connector_instance().is_none());
    // after set_ready/set_started the forward address is the local proxy ingress URL
    let addr = ctx
        .forward_dataplane_address()
        .expect("proxy ingress address must be set");
    assert!(
        addr.endpoint.contains("/dataplane/proxy/"),
        "expected proxy ingress path, got: {}",
        addr.endpoint
    );
}

/// set_auth on its own: a no-op authentication followed by put(Auth).
#[tokio::test]
async fn set_auth_persists_auth_state() {
    let mut mock = MockDataplaneTransferServiceTrait::new();
    expect_create(&mut mock);
    expect_put(&mut mock, TransferState::Auth);

    let entity: Arc<dyn dataplane::services::dataplane_transfers::DataplaneTransferServiceTrait> =
        Arc::new(mock);
    let context = init_context(entity.clone()).await;

    let result = handler(entity.clone()).set_auth(context).await;
    assert!(result.is_ok());
    let ctx = result.unwrap();
    assert_eq!(ctx.dataplane_process().inner.state, TransferState::Auth);
    assert!(ctx.connector_instance().is_none());
}

/// set_ready in isolation: put(Ready) exactly once.
#[tokio::test]
async fn set_ready_persists_state() {
    let mut mock = MockDataplaneTransferServiceTrait::new();
    expect_create(&mut mock);
    expect_put(&mut mock, TransferState::Ready);

    let entity: Arc<dyn dataplane::services::dataplane_transfers::DataplaneTransferServiceTrait> =
        Arc::new(mock);
    let context = init_context(entity.clone()).await;

    let result = handler(entity.clone()).set_ready(context).await;
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap().dataplane_process().inner.state,
        TransferState::Ready
    );
}

/// set_started must call put exactly once with state=Started and return the updated context.
#[tokio::test]
async fn set_started_persists_state() {
    let mut mock = MockDataplaneTransferServiceTrait::new();
    expect_create(&mut mock);
    expect_put(&mut mock, TransferState::Started);

    let entity: Arc<dyn dataplane::services::dataplane_transfers::DataplaneTransferServiceTrait> =
        Arc::new(mock);
    let context = init_context(entity.clone()).await;

    let result = handler(entity).set_started(context).await;

    assert!(result.is_ok());
    assert_eq!(
        result.unwrap().dataplane_process().inner.state,
        TransferState::Started
    );
}

/// set_stopped must call put exactly once with state=Stopped and return the updated context.
#[tokio::test]
async fn set_stopped_persists_state() {
    let mut mock = MockDataplaneTransferServiceTrait::new();
    expect_create(&mut mock);
    expect_put(&mut mock, TransferState::Stopped);

    let entity: Arc<dyn dataplane::services::dataplane_transfers::DataplaneTransferServiceTrait> =
        Arc::new(mock);
    let context = init_context(entity.clone()).await;

    let result = handler(entity).set_stopped(context).await;

    assert!(result.is_ok());
    assert_eq!(
        result.unwrap().dataplane_process().inner.state,
        TransferState::Stopped
    );
}

/// set_terminating must call put exactly once with state=Terminated and return the updated
/// context.
#[tokio::test]
async fn set_terminating_persists_state() {
    let mut mock = MockDataplaneTransferServiceTrait::new();
    expect_create(&mut mock);
    expect_put(&mut mock, TransferState::Terminated);

    let entity: Arc<dyn dataplane::services::dataplane_transfers::DataplaneTransferServiceTrait> =
        Arc::new(mock);
    let context = init_context(entity.clone()).await;

    let result = handler(entity).set_terminating(context).await;

    assert!(result.is_ok());
    assert_eq!(
        result.unwrap().dataplane_process().inner.state,
        TransferState::Terminated
    );
}

/// Pull has no subscriber: set_subscribing changes nothing and persists nothing.
#[tokio::test]
async fn set_subscribing_is_noop_for_pull() {
    let mut mock = MockDataplaneTransferServiceTrait::new();
    expect_create(&mut mock);
    // no expect_put — any unexpected call makes the mock panic

    let entity: Arc<dyn dataplane::services::dataplane_transfers::DataplaneTransferServiceTrait> =
        Arc::new(mock);
    let context = init_context(entity.clone()).await;

    let result = handler(entity).set_subscribing(context).await;

    assert!(result.is_ok());
    // context comes back untouched: state is still Init as returned by create
    assert_eq!(
        result.unwrap().dataplane_process().inner.state,
        TransferState::Init
    );
}

/// Pull has no subscriber: set_unsubscribing changes nothing either.
#[tokio::test]
async fn set_unsubscribing_is_noop_for_pull() {
    let mut mock = MockDataplaneTransferServiceTrait::new();
    expect_create(&mut mock);
    // no expect_put — any unexpected call makes the mock panic

    let entity: Arc<dyn dataplane::services::dataplane_transfers::DataplaneTransferServiceTrait> =
        Arc::new(mock);
    let context = init_context(entity.clone()).await;

    let result = handler(entity).set_unsubscribing(context).await;

    assert!(result.is_ok());
    // context comes back untouched: state is still Init as returned by create
    assert_eq!(
        result.unwrap().dataplane_process().inner.state,
        TransferState::Init
    );
}
