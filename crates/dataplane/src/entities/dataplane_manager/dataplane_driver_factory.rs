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

use crate::entities::dataplane_drivers::authentication::api_key::ApiKeyAuthenticator;
use crate::entities::dataplane_drivers::authentication::basic_config::BasicConfigAuthenticator;
use crate::entities::dataplane_drivers::authentication::bearer_token::BearerTokenAuthenticator;
use crate::entities::dataplane_drivers::authentication::no_auth::NoAuthAuthenticator;
use crate::entities::dataplane_drivers::authentication::no_op::NoOpAuthenticator;
use crate::entities::dataplane_drivers::authentication::oauth::OauthAuthenticator;
use crate::entities::dataplane_drivers::configuration::http_consumer_pull::HttpConsumerPullConfigurator;
use crate::entities::dataplane_drivers::configuration::http_consumer_push::HttpConsumerPushConfigurator;
use crate::entities::dataplane_drivers::configuration::http_provider_pull::HttpProviderPullConfigurator;
use crate::entities::dataplane_drivers::configuration::http_provider_push::HttpProviderPushConfigurator;
use crate::entities::dataplane_drivers::pubsub::http::HttpPubSubscriber;
use crate::entities::dataplane_drivers::pubsub::no_op::NoOpPubSubscriber;
use crate::entities::dataplane_drivers::{
    DataplaneDriver, DriverAuthenticatorTrait, DriverProxyConfiguratorTrait, DriverPubSubTrait,
};
use crate::entities::dataplane_manager::dataplane_context::DataplaneContext;
use crate::entities::dataplane_transfers::{InteractionMode, TransferRole};
use crate::errors::DataplaneError;
use connector::{
    AuthenticationConfig, ConnectorInstanceDto, InteractionConfig, KeystoreLookup, ProtocolSpec,
};
use std::sync::Arc;
use ymir::errors::Outcome;

pub struct DataplaneDriverFactory {
    keystore: Option<Arc<dyn KeystoreLookup>>,
}

impl Default for DataplaneDriverFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl DataplaneDriverFactory {
    pub fn new() -> Self {
        Self { keystore: None }
    }

    /// Attaches the keystore used to resolve driver credentials.
    pub fn with_keystore(mut self, keystore: Arc<dyn KeystoreLookup>) -> Self {
        self.keystore = Some(keystore);
        self
    }
}

#[cfg_attr(test, mockall::automock)]
pub trait DataplaneDriverFactoryTrait: Send + Sync {
    fn get_or_create_driver(&self, context: &DataplaneContext) -> Outcome<DataplaneDriver>;
}

impl DataplaneDriverFactoryTrait for DataplaneDriverFactory {
    fn get_or_create_driver(&self, context: &DataplaneContext) -> Outcome<DataplaneDriver> {
        let authenticator = self.resolve_authenticator(context)?;
        let proxy_configurator = self.resolve_proxy_configurator(context)?;
        let subscriber = self.resolve_subscriber(context)?;
        Ok(DataplaneDriver {
            authenticator,
            proxy_configurator,
            subscriber,
        })
    }
}

impl DataplaneDriverFactory {
    fn resolve_authenticator(
        &self,
        context: &DataplaneContext,
    ) -> Outcome<Arc<dyn DriverAuthenticatorTrait>> {
        if let Some(connector_instance) = context.connector_instance() {
            match connector_instance.authentication_config {
                AuthenticationConfig::NoAuth => Ok(Arc::new(NoAuthAuthenticator)),
                AuthenticationConfig::BasicAuth(_) => Ok(Arc::new(BasicConfigAuthenticator)),
                AuthenticationConfig::BearerToken { .. } => Ok(Arc::new(BearerTokenAuthenticator)),
                AuthenticationConfig::ApiKey { .. } => Ok(Arc::new(ApiKeyAuthenticator)),
                AuthenticationConfig::OAuth2 { .. } => Ok(Arc::new(OauthAuthenticator)),
            }
        } else {
            match context.dataplane_process_role() {
                TransferRole::Consumer => Ok(Arc::new(NoOpAuthenticator)),
                TransferRole::Provider => Err(DataplaneError::ConnectorNotAvailable.into()),
            }
        }
    }

    fn resolve_proxy_configurator(
        &self,
        context: &DataplaneContext,
    ) -> Outcome<Arc<dyn DriverProxyConfiguratorTrait>> {
        if let Some(connector_instance) = context.connector_instance() {
            let role = context.dataplane_process_role();
            let interaction_mode = context.dataplane_process_interaction_mode();
            let protocol_spec = Self::extract_protocol(&connector_instance);

            match (protocol_spec, &role, &interaction_mode) {
                (ProtocolSpec::Http(_), TransferRole::Provider, InteractionMode::Pull) => {
                    Ok(Arc::new(HttpProviderPullConfigurator))
                }
                (ProtocolSpec::Http(_), TransferRole::Provider, InteractionMode::Push) => {
                    Ok(Arc::new(HttpProviderPushConfigurator))
                }
                (_, _, _) => Err(DataplaneError::NoDriverForCombination {
                    role: role.to_string(),
                    mode: interaction_mode.to_string(),
                }
                .into()),
            }
        } else {
            match (
                context.dataplane_process_role(),
                context.dataplane_process_interaction_mode(),
            ) {
                (TransferRole::Consumer, InteractionMode::Pull) => {
                    Ok(Arc::new(HttpConsumerPullConfigurator))
                }
                (TransferRole::Consumer, InteractionMode::Push) => {
                    Ok(Arc::new(HttpConsumerPushConfigurator))
                }
                _ => Err(DataplaneError::ConnectorNotAvailable.into()),
            }
        }
    }

    fn resolve_subscriber(
        &self,
        context: &DataplaneContext,
    ) -> Outcome<Option<Arc<dyn DriverPubSubTrait>>> {
        if let Some(connector_instance) = context.connector_instance() {
            let interaction_mode = context.dataplane_process_interaction_mode();
            match interaction_mode {
                InteractionMode::Pull => Ok(None),
                InteractionMode::Push => match Self::extract_protocol(&connector_instance) {
                    ProtocolSpec::Http(_) => Ok(Some(Arc::new(HttpPubSubscriber::new(
                        self.keystore.clone(),
                    )))),
                    ProtocolSpec::Kafka(_) => {
                        return Err(DataplaneError::FeatureNotImplemented {
                            feature: "Kafka push subscriber".to_string(),
                        }
                        .into())
                    }
                },
            }
        } else {
            match context.dataplane_process_interaction_mode() {
                InteractionMode::Pull => Ok(None),
                InteractionMode::Push => Ok(Some(Arc::new(NoOpPubSubscriber))),
            }
        }
    }

    fn extract_protocol(connector: &ConnectorInstanceDto) -> ProtocolSpec {
        match connector.interaction {
            InteractionConfig::Pull(ref p) => p.data_access.clone(),
            InteractionConfig::Push(ref p) => p.subscribe.clone(),
        }
    }
}
