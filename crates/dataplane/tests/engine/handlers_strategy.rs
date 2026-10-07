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

//! Handler strategy: each role and direction of the init command picks its handler.

use dataplane::engine::dataplane_manager::dataplane_commands::{
    DataplaneCommandStateMachine, DataplaneInitCommandTypes,
};
use dataplane::engine::dataplane_manager::dataplane_context::DataplaneContext;
use dataplane::services::dataplane_transfers::DataplaneTransferServiceTrait;
use std::sync::Arc;

use common::test_utils::config_fixtures::transfer_config_fixture;
use connector::{
    AuthenticationConfig, ConnectorInstanceDto, ConnectorMetadata, HttpSpec, InteractionConfig,
    ProtocolSpec, PushLifecycle, TemplateVecString,
};
use dataplane::data::sea_orm::orm::dataplane_transfers;
use dataplane::engine::dataplane_manager::dataplane_commands::DataplaneInitCommandDirection;
use dataplane::engine::dataplane_manager::dataplane_handlers_strategy::DataplaneStrategyFactory;
use dataplane::entities::dataplane_transfers::{DataplaneTransferDto, TransferState};
use dataplane::services::dataplane_transfers::MockDataplaneTransferServiceTrait;
use dataplane::DataplaneAddress;
use std::str::FromStr;
use urn::Urn;

use connector::MockConnectorInstanceFacadeTrait as MockConnectorMock;

// Entity mock that handles the create call made by DataplaneContext::from_init,
// reflecting back the role and mode from the NewDataplaneTransferDto it receives.
fn entity_for_init() -> Arc<dyn DataplaneTransferServiceTrait> {
    let mut mock = MockDataplaneTransferServiceTrait::new();
    mock.expect_create().returning(|_, dto| {
        Ok(DataplaneTransferDto {
            inner: dataplane_transfers::Model {
                user_id: dto.owner.clone().unwrap().user_id,
                user_role: common::oauth::RolePath::root(),
                visibility: common::oauth::Visibility::Private,
                id: "urn:dataplane-transfer:test".to_string(),
                transfer_process_id: dto.transfer_process_id.clone(),
                role: dto.role.clone(),
                interaction_mode: dto.interaction_mode.clone(),
                state: TransferState::Init,
                connector_instance_id: dto.connector_instance_id.as_ref().map(|u| u.to_string()),
                ingress_config: serde_json::json!("NoOp"),
                egress_config: serde_json::json!("NoOp"),
                flow_control: None,
                created_at: chrono::Utc::now().into(),
                updated_at: None,
            },
            fields: Default::default(),
            logs: vec![],
        })
    });
    Arc::new(mock)
}

async fn dummy_context(init: DataplaneInitCommandTypes) -> DataplaneContext {
    DataplaneContext::from_init(
        entity_for_init(),
        Arc::new(MockConnectorMock::new()),
        transfer_config_fixture(),
        init,
    )
    .await
    .unwrap()
}

// Connector fixture for provider tests — interaction type doesn't affect routing
// (routing is based on role+direction stored in the DTO, not on connector config).
fn connector_fixture() -> ConnectorInstanceDto {
    ConnectorInstanceDto {
        id: Urn::from_str("urn:connector-instance:1").unwrap(),
        user_id: "user-1".to_string(),
        metadata: ConnectorMetadata {
            name: None,
            author: None,
            description: None,
            version: None,
            created_at: None,
        },
        authentication_config: AuthenticationConfig::NoAuth,
        interaction: InteractionConfig::Push(PushLifecycle {
            subscribe: ProtocolSpec::Http(HttpSpec {
                url_template: "http://example.com/events".to_string(),
                method: TemplateVecString::Value(vec!["POST".to_string()]),
                headers: None,
                body_template: None,
            }),
            unsubscribe: None,
        }),
        distribution_id: Urn::from_str("urn:distribution:1").unwrap(),
    }
}

fn get_strategy(context: DataplaneContext) -> Box<dyn DataplaneCommandStateMachine> {
    DataplaneStrategyFactory::new(
        entity_for_init(),
        Arc::new(MockConnectorMock::new()),
        transfer_config_fixture(),
        None,
    )
    .get_strategy(&context)
}

fn empty_address() -> DataplaneAddress {
    DataplaneAddress {
        endpoint_type: "HTTP".to_string(),
        endpoint: "http://example.com".to_string(),
        authorization_type: None,
        authorization: None,
    }
}

/// A consumer pull init is handled by ConsumerPull.
#[tokio::test]
async fn consumer_pull_routes_to_consumer_pull_handler() {
    let context = dummy_context(DataplaneInitCommandTypes::AsConsumer {
        owner: common::test_utils::scopes::TestUsers::owner("tenant-1"),
        transfer_process_id: Urn::from_str("urn:tp:1").unwrap(),
        direction: DataplaneInitCommandDirection::Pull {
            data_address: Some(empty_address()),
        },
    })
    .await;
    assert_eq!(get_strategy(context).handler_name(), "ConsumerPull");
}

/// A consumer push init is handled by ConsumerPush.
#[tokio::test]
async fn consumer_push_routes_to_consumer_push_handler() {
    let context = dummy_context(DataplaneInitCommandTypes::AsConsumer {
        owner: common::test_utils::scopes::TestUsers::owner("tenant-1"),
        transfer_process_id: Urn::from_str("urn:tp:1").unwrap(),
        direction: DataplaneInitCommandDirection::Push {
            data_address: Some(empty_address()),
        },
    })
    .await;
    assert_eq!(get_strategy(context).handler_name(), "ConsumerPush");
}

/// A provider pull init is handled by ProviderPull.
#[tokio::test]
async fn provider_pull_routes_to_provider_pull_handler() {
    let context = dummy_context(DataplaneInitCommandTypes::AsProvider {
        owner: common::test_utils::scopes::TestUsers::owner("tenant-1"),
        transfer_process_id: Urn::from_str("urn:tp:1").unwrap(),
        connector_instance: connector_fixture(),
        direction: DataplaneInitCommandDirection::Pull {
            data_address: Some(empty_address()),
        },
    })
    .await;
    assert_eq!(get_strategy(context).handler_name(), "ProviderPull");
}

/// A provider push init is handled by ProviderPush.
#[tokio::test]
async fn provider_push_routes_to_provider_push_handler() {
    let context = dummy_context(DataplaneInitCommandTypes::AsProvider {
        owner: common::test_utils::scopes::TestUsers::owner("tenant-1"),
        transfer_process_id: Urn::from_str("urn:tp:1").unwrap(),
        connector_instance: connector_fixture(),
        direction: DataplaneInitCommandDirection::Push {
            data_address: Some(empty_address()),
        },
    })
    .await;
    assert_eq!(get_strategy(context).handler_name(), "ProviderPush");
}
