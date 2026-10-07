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

//! DataplaneManager executing commands with a mocked transfer service and connector facade,
//! split by role and direction.

mod consumer;
mod init;
mod provider_pull;
mod provider_push;

use dataplane::engine::dataplane_manager::dataplane_commands::{
    DataplaneCommand, DataplaneCommandResponse,
};
use std::sync::Arc;

use common::test_utils::config_fixtures::transfer_config_fixture;
use connector::{
    AuthenticationConfig, ConnectorInstanceDto, ConnectorMetadata, HttpSpec, InteractionConfig,
    ProtocolSpec, PullLifecycle, PushLifecycle, TemplateVecString,
};
use dataplane::data::sea_orm::orm::dataplane_transfers;
use dataplane::engine::dataplane_drivers::authentication::no_op::NoOpAuthenticator;
use dataplane::engine::dataplane_drivers::configuration::no_op::NoOpProxyConfigurator;
use dataplane::engine::dataplane_drivers::pubsub::no_op::NoOpPubSubscriber;
use dataplane::engine::dataplane_drivers::DataplaneDriver;
use dataplane::engine::dataplane_manager::dataplane_commands::{
    DataplaneContinuation, DataplaneInitCommandDirection, DataplaneInitCommandTypes,
};
use dataplane::engine::dataplane_manager::dataplane_driver_factory::MockDataplaneDriverFactoryTrait;
use dataplane::engine::dataplane_manager::dataplane_manager::DataplaneManager;
use dataplane::entities::dataplane_transfers::{
    DataplaneTransferDto, InteractionMode, NewDataplaneTransferDto, TransferRole, TransferState,
};
use dataplane::services::dataplane_transfers::MockDataplaneTransferServiceTrait;
use dataplane::DataplaneAddress;
use std::str::FromStr;
use urn::Urn;

use connector::MockConnectorInstanceFacadeTrait as MockConnectorMock;

fn dummy_driver() -> DataplaneDriver {
    DataplaneDriver {
        authenticator: Arc::new(NoOpAuthenticator),
        proxy_configurator: Arc::new(NoOpProxyConfigurator),
        subscriber: Some(Arc::new(NoOpPubSubscriber)),
    }
}

fn dummy_driver_pull() -> DataplaneDriver {
    DataplaneDriver {
        authenticator: Arc::new(NoOpAuthenticator),
        proxy_configurator: Arc::new(NoOpProxyConfigurator),
        subscriber: None,
    }
}

fn dummy_dataplane_transfer_dto(
    id: &str,
    tp_id: &str,
    role: TransferRole,
    mode: InteractionMode,
    state: TransferState,
    connector_instance_id: Option<Urn>,
) -> DataplaneTransferDto {
    DataplaneTransferDto {
        inner: dataplane_transfers::Model {
            user_id: "tenant-1".to_string(),
            user_role: common::oauth::RolePath::root(),
            visibility: common::oauth::Visibility::Private,
            id: id.to_string(),
            transfer_process_id: tp_id.to_string(),
            role,
            interaction_mode: mode,
            state,
            connector_instance_id: connector_instance_id.map(|u| u.to_string()),
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

fn dummy_pull_connector(urn: &Urn) -> ConnectorInstanceDto {
    ConnectorInstanceDto {
        id: urn.clone(),
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
                url_template: "http://example.com/data".to_string(),
                method: TemplateVecString::Value(vec!["GET".to_string()]),
                headers: None,
                body_template: None,
            }),
        }),
        distribution_id: Urn::from_str("urn:distribution:1").unwrap(),
    }
}

fn dummy_push_connector(urn: &Urn) -> ConnectorInstanceDto {
    ConnectorInstanceDto {
        id: urn.clone(),
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
                url_template: "http://example.com/data".to_string(),
                method: TemplateVecString::Value(vec!["GET".to_string()]),
                headers: None,
                body_template: None,
            }),
            unsubscribe: Some(ProtocolSpec::Http(HttpSpec {
                url_template: "http://example.com/data".to_string(),
                method: TemplateVecString::Value(vec!["GET".to_string()]),
                headers: None,
                body_template: None,
            })),
        }),
        distribution_id: Urn::from_str("urn:distribution:1").unwrap(),
    }
}

fn dummy_dataplane_forward_address() -> DataplaneAddress {
    DataplaneAddress {
        endpoint_type: "HTTP".to_string(),
        endpoint: "http://dummy-data-address.com".to_string(),
        authorization_type: Some("Dummy".to_string()),
        authorization: Some("Dummy".to_string()),
    }
}
