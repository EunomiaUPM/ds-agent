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

//! DataplaneHandlerProviderPull: each state step with a mocked transfer service.

use dataplane::engine::dataplane_manager::dataplane_commands::{
    DataplaneCommandStateMachine, DataplaneInitCommandTypes,
};
use dataplane::engine::dataplane_manager::dataplane_context::DataplaneContext;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;

use common::test_utils::config_fixtures::transfer_config_fixture;
use connector::{
    AuthenticationConfig, ConnectorInstanceDto, ConnectorMetadata, HttpSpec, InteractionConfig,
    ProtocolSpec, PullLifecycle, TemplateVecString,
};
use dataplane::data::sea_orm::orm::dataplane_transfers;
use dataplane::engine::dataplane_manager::dataplane_commands::DataplaneInitCommandDirection;
use dataplane::engine::dataplane_manager::dataplane_handlers_provider_pull::DataplaneHandlerProviderPull;
use dataplane::entities::dataplane_transfers::{
    DataplaneTransferDto, InteractionMode, TransferRole, TransferState,
};
use dataplane::services::dataplane_transfers::MockDataplaneTransferServiceTrait;
use dataplane::DataplaneAddress;

use connector::MockConnectorInstanceFacadeTrait as MockConnectorMock;

const DP_URN: &str = "urn:dataplane-transfer:1";
const TP_URN: &str = "urn:transfer-process:1";
const CONNECTOR_URN: &str = "urn:connector-instance:1";

// A provider-pull connector: NoAuth + HTTP data access.
fn connector_fixture() -> ConnectorInstanceDto {
    ConnectorInstanceDto {
        id: Urn::from_str(CONNECTOR_URN).unwrap(),
        user_id: "user-1".to_string(),
        metadata: ConnectorMetadata {
            name: None,
            author: None,
            description: None,
            version: None,
            created_at: None,
        },
        authentication_config: AuthenticationConfig::NoAuth,
        interaction: InteractionConfig::Pull(PullLifecycle {
            data_access: ProtocolSpec::Http(HttpSpec {
                url_template: "http://data-source.internal/api".to_string(),
                method: TemplateVecString::Value(vec!["GET".to_string()]),
                headers: None,
                body_template: None,
            }),
        }),
        distribution_id: Urn::from_str("urn:distribution:1").unwrap(),
    }
}

// The proxy address the consumer will use to pull data from this provider.
fn proxy_address_fixture() -> DataplaneAddress {
    DataplaneAddress {
        endpoint_type: "HTTP".to_string(),
        endpoint: "http://dataplane.example.com/proxy/transfer".to_string(),
        authorization_type: Some("Bearer".to_string()),
        authorization: Some("proxy-token-xyz".to_string()),
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
            role: TransferRole::Provider,
            interaction_mode: InteractionMode::Pull,
            state,
            connector_instance_id: Some(CONNECTOR_URN.to_string()),
            ingress_config: serde_json::json!("NoOp"),
            egress_config: serde_json::json!("NoOp"),
            flow_control: None,
            created_at: chrono::Utc::now().into(),
            updated_at: None,
        },
        fields: Default::default(),
        logs: vec![],
    }
}

fn handler(
    entity: Arc<dyn dataplane::services::dataplane_transfers::DataplaneTransferServiceTrait>,
) -> DataplaneHandlerProviderPull {
    DataplaneHandlerProviderPull::new(
        entity,
        Arc::new(MockConnectorMock::new()),
        transfer_config_fixture(),
        None,
    )
}

async fn init_context(
    entity: Arc<dyn dataplane::services::dataplane_transfers::DataplaneTransferServiceTrait>,
) -> DataplaneContext {
    DataplaneContext::from_init(
        entity,
        Arc::new(MockConnectorMock::new()),
        transfer_config_fixture(),
        DataplaneInitCommandTypes::AsProvider {
            owner: common::test_utils::scopes::TestUsers::owner("tenant-1"),
            transfer_process_id: Urn::from_str(TP_URN).unwrap(),
            connector_instance: connector_fixture(),
            direction: DataplaneInitCommandDirection::Pull {
                data_address: Some(proxy_address_fixture()),
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

/// set_configuring runs the whole provider-pull flow in one go: it configures the proxy,
/// authenticates, and moves through ready to started.
#[tokio::test]
async fn set_configuring_persists_configuring_state_and_preserves_connector() {
    let mut mock = MockDataplaneTransferServiceTrait::new();
    expect_create(&mut mock);
    expect_put(&mut mock, TransferState::Configuring);
    expect_put(&mut mock, TransferState::Auth);
    expect_put(&mut mock, TransferState::Ready);
    expect_put(&mut mock, TransferState::Started);

    let entity: Arc<dyn dataplane::services::dataplane_transfers::DataplaneTransferServiceTrait> =
        Arc::new(mock);
    let context = init_context(entity.clone()).await;

    let result = handler(entity).set_configuring(context).await;

    assert!(result.is_ok());
    let ctx = result.unwrap();
    assert_eq!(ctx.dataplane_process().inner.state, TransferState::Started);
    assert!(ctx.driver().is_some());
    // connector must survive the full flow — it describes the provider's data source
    let conn = ctx
        .connector_instance()
        .expect("connector instance must be preserved");
    assert_eq!(conn.id, Urn::from_str(CONNECTOR_URN).unwrap());
}

/// set_auth is atomic: NoAuth - NoOp authentication - put(Auth). Does NOT proceed to ready.
#[tokio::test]
async fn set_auth_persists_auth_state() {
    let mut mock = MockDataplaneTransferServiceTrait::new();
    expect_create(&mut mock);
    expect_put(&mut mock, TransferState::Auth);

    let entity: Arc<dyn dataplane::services::dataplane_transfers::DataplaneTransferServiceTrait> =
        Arc::new(mock);
    let context = init_context(entity.clone()).await;

    let result = handler(entity).set_auth(context).await;

    assert!(result.is_ok());
    let ctx = result.unwrap();
    assert_eq!(ctx.dataplane_process().inner.state, TransferState::Auth);
    assert!(ctx.connector_instance().is_some());
}

/// set_ready must call put exactly once with state=Ready and return the updated context.
#[tokio::test]
async fn set_ready_persists_state() {
    let mut mock = MockDataplaneTransferServiceTrait::new();
    expect_create(&mut mock);
    expect_put(&mut mock, TransferState::Ready);

    let entity: Arc<dyn dataplane::services::dataplane_transfers::DataplaneTransferServiceTrait> =
        Arc::new(mock);
    let context = init_context(entity.clone()).await;

    let result = handler(entity).set_ready(context).await;

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

/// Pull mode has no subscriber: set_subscribing must be a no-op — no put is called
/// and the context is returned unchanged. (driver is None when built from from_init)
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
    assert_eq!(
        result.unwrap().dataplane_process().inner.state,
        TransferState::Init
    );
}

/// Pull mode has no subscriber: set_unsubscribing must also be a no-op.
#[tokio::test]
async fn set_unsubscribing_is_noop_for_pull() {
    let mut mock = MockDataplaneTransferServiceTrait::new();
    expect_create(&mut mock);

    let entity: Arc<dyn dataplane::services::dataplane_transfers::DataplaneTransferServiceTrait> =
        Arc::new(mock);
    let context = init_context(entity.clone()).await;

    let result = handler(entity).set_unsubscribing(context).await;

    assert!(result.is_ok());
    assert_eq!(
        result.unwrap().dataplane_process().inner.state,
        TransferState::Init
    );
}
