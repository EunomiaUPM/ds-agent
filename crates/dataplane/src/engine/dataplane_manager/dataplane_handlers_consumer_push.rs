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
    DataplaneCommand, DataplaneCommandStateMachine,
};
use crate::engine::dataplane_manager::dataplane_context::DataplaneContext;
use crate::engine::dataplane_manager::dataplane_driver_factory::DataplaneDriverFactoryTrait;
use crate::services::dataplane_transfers::DataplaneTransferServiceTrait;
use crate::signals::DataplaneSignalSender;
use common::config::services::TransferConfig;
use connector::ConnectorInstanceFacadeTrait;
use keystore::SecretStore;
use std::sync::Arc;
use ymir::errors::Outcome;

pub struct DataplaneHandlerConsumerPush {
    dataplane_service: Arc<dyn DataplaneTransferServiceTrait>,
    connector_service: Arc<dyn ConnectorInstanceFacadeTrait>,
    config: Arc<TransferConfig>,
    secret_store: Option<Arc<dyn SecretStore>>,
    driver_factory: Arc<dyn DataplaneDriverFactoryTrait>,
    signals: DataplaneSignalSender,
}

impl DataplaneHandlerConsumerPush {
    pub fn new(
        dataplane_service: Arc<dyn DataplaneTransferServiceTrait>,
        connector_service: Arc<dyn ConnectorInstanceFacadeTrait>,
        config: Arc<TransferConfig>,
        secret_store: Option<Arc<dyn SecretStore>>,
        driver_factory: Arc<dyn DataplaneDriverFactoryTrait>,
    ) -> Self {
        Self {
            dataplane_service,
            connector_service,
            config,
            secret_store,
            driver_factory,
            signals: DataplaneSignalSender::noop(),
        }
    }

    pub fn with_signals(mut self, signals: DataplaneSignalSender) -> Self {
        self.signals = signals;
        self
    }
}

#[async_trait::async_trait]
impl DataplaneCommandStateMachine for DataplaneHandlerConsumerPush {
    fn handler_name(&self) -> &'static str {
        "ConsumerPush"
    }
    fn dataplane_service(&self) -> Arc<dyn DataplaneTransferServiceTrait> {
        self.dataplane_service.clone()
    }
    fn connector_service(&self) -> Arc<dyn ConnectorInstanceFacadeTrait> {
        self.connector_service.clone()
    }
    fn transfer_config(&self) -> Arc<TransferConfig> {
        self.config.clone()
    }
    fn secret_store(&self) -> Option<Arc<dyn SecretStore>> {
        self.secret_store.clone()
    }
    fn driver_factory(&self) -> Arc<dyn DataplaneDriverFactoryTrait> {
        self.driver_factory.clone()
    }
    fn signal_sender(&self) -> DataplaneSignalSender {
        self.signals.clone()
    }

    fn accepts_command(&self, command: &DataplaneCommand, _context: &DataplaneContext) -> bool {
        matches!(
            command,
            DataplaneCommand::SetInit(_)
                | DataplaneCommand::GetAssociated(_)
                | DataplaneCommand::SetSubscribing(_)
                | DataplaneCommand::SetUnsubscribing(_)
                | DataplaneCommand::SetTerminating(_)
        )
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(handler = self.handler_name()))]
    async fn set_terminating(&self, context: DataplaneContext) -> Outcome<DataplaneContext> {
        match self.try_unsubscribe(context.clone()).await {
            Ok(unsubscribed) => self.terminate_process(unsubscribed).await,
            Err(e) => {
                tracing::warn!(
                    handler = self.handler_name(),
                    error = %e,
                    "unsubscribe failed during termination; proceeding to Terminated"
                );
                let err_reason = match context.transition_reason() {
                    Some(caller) => {
                        format!("Unsubscribe failed during termination ({e}); {caller}")
                    }
                    None => format!("Unsubscribe failed during termination: {e}"),
                };
                self.terminate_process_with_reason(context, err_reason).await
            }
        }
    }
}
