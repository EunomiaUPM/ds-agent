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

use crate::engine::dataplane_manager::dataplane_commands::{
    DataplaneCommand, DataplaneCommandResponse,
};
use crate::engine::dataplane_manager::dataplane_context::DataplaneContext;
use crate::engine::dataplane_manager::dataplane_driver_factory::{
    DataplaneDriverFactory, DataplaneDriverFactoryTrait,
};
use crate::engine::dataplane_manager::dataplane_handlers_strategy::DataplaneStrategyFactory;
use crate::errors::DataplaneError;
use crate::services::dataplane_transfers::DataplaneTransferServiceTrait;
use common::config::services::TransferConfig;
use connector::ConnectorInstanceFacadeTrait;
use keystore::SecretStore;
use std::sync::Arc;
use ymir::errors::Outcome;

/// Runs dataplane commands: moves the process state and drives the matching driver.
pub struct DataplaneManager {
    dataplane_service: Arc<dyn DataplaneTransferServiceTrait>,
    connector_service: Arc<dyn ConnectorInstanceFacadeTrait>,
    config: Arc<TransferConfig>,
    driver_factory: Arc<dyn DataplaneDriverFactoryTrait>,
    secret_store: Option<Arc<dyn SecretStore>>,
}

impl DataplaneManager {
    /// Required dependencies. The driver factory defaults to a keystore-less
    /// `DataplaneDriverFactory` and the secret store to `None`; override either
    /// with the `with_*` methods below.
    pub fn new(
        dataplane_service: Arc<dyn DataplaneTransferServiceTrait>,
        connector_service: Arc<dyn ConnectorInstanceFacadeTrait>,
        config: Arc<TransferConfig>,
    ) -> Self {
        Self {
            dataplane_service,
            connector_service,
            config,
            driver_factory: Arc::new(DataplaneDriverFactory::new()),
            secret_store: None,
        }
    }

    /// Overrides the default driver factory (e.g. one backed by the keystore).
    pub fn with_driver_factory(
        mut self,
        driver_factory: Arc<dyn DataplaneDriverFactoryTrait>,
    ) -> Self {
        self.driver_factory = driver_factory;
        self
    }

    /// Enables runtime-secret resolution via the given secret store.
    pub fn with_secret_store(mut self, store: Arc<dyn SecretStore>) -> Self {
        self.secret_store = Some(store);
        self
    }

    /// Applies the command to its process and returns what the control plane needs back.
    #[tracing::instrument(level = "info", skip_all, err, fields(command = %command))]
    pub async fn execute_command(
        &self,
        command: DataplaneCommand,
    ) -> Outcome<DataplaneCommandResponse> {
        let mut context = match &command {
            DataplaneCommand::SetInit(init) => {
                DataplaneContext::from_init(
                    self.dataplane_service.clone(),
                    self.connector_service.clone(),
                    self.config.clone(),
                    init.clone(),
                )
                .await?
            }
            DataplaneCommand::SetConfiguring((cont, address)) => {
                DataplaneContext::from_continuation(
                    self.dataplane_service.clone(),
                    self.connector_service.clone(),
                    self.driver_factory.clone(),
                    self.config.clone(),
                    cont.clone(),
                    Some(address.clone()),
                )
                .await?
            }
            DataplaneCommand::GetAssociated(continuation)
            | DataplaneCommand::SetStarted(continuation)
            | DataplaneCommand::SetSubscribing(continuation)
            | DataplaneCommand::SetUnsubscribing(continuation)
            | DataplaneCommand::SetStopped(continuation)
            | DataplaneCommand::SetTerminating(continuation) => {
                DataplaneContext::from_continuation(
                    self.dataplane_service.clone(),
                    self.connector_service.clone(),
                    self.driver_factory.clone(),
                    self.config.clone(),
                    continuation.clone(),
                    None,
                )
                .await?
            }
            cmd => {
                return Err(DataplaneError::UnexpectedCommand {
                    command: cmd.to_string(),
                }
                .into())
            }
        };

        // Resolve runtime secret placeholders before dispatch.
        if let (Some(runtime), Some(store)) = (context.runtime().cloned(), &self.secret_store) {
            use crate::engine::dataplane_manager::dataplane_runtime::RuntimeSecretVault;
            let user_id = context.dataplane_process().inner.user_id.clone();
            let resolved = RuntimeSecretVault::new(store.as_ref(), &user_id)
                .resolve(runtime)
                .await;
            context.set_runtime(resolved);
        }

        // Select strategy based on (role, interaction_mode) from the loaded context
        let handler_strategy = DataplaneStrategyFactory::new(
            self.dataplane_service.clone(),
            self.connector_service.clone(),
            self.config.clone(),
            self.secret_store.clone(),
        )
        .get_strategy(&context);

        // Dispatch to the appropriate handler method
        let new_context = match command {
            DataplaneCommand::GetAssociated(_) => handler_strategy.get_associated(context),
            DataplaneCommand::SetInit(_) => handler_strategy.set_init(context),
            DataplaneCommand::SetConfiguring(_) => handler_strategy.set_configuring(context),
            DataplaneCommand::SetStarted(_) => handler_strategy.set_started(context),
            DataplaneCommand::SetSubscribing(_) => handler_strategy.set_subscribing(context),
            DataplaneCommand::SetUnsubscribing(_) => handler_strategy.set_unsubscribing(context),
            DataplaneCommand::SetStopped(_) => handler_strategy.set_stopped(context),
            DataplaneCommand::SetTerminating(_) => handler_strategy.set_terminating(context),
            cmd => unreachable!("command '{}' was rejected in context-building phase", cmd),
        }
        .await?;

        if let Some(forward_address) = new_context.forward_dataplane_address() {
            Ok(DataplaneCommandResponse::OkWithAddress(
                forward_address.clone(),
            ))
        } else {
            Ok(DataplaneCommandResponse::Ok)
        }
    }
}
