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

use crate::engine::dataplane_manager::dataplane_commands::DataplaneCommandStateMachine;
use crate::engine::dataplane_manager::dataplane_context::DataplaneContext;
use crate::engine::dataplane_manager::dataplane_handlers_consumer_pull::DataplaneHandlerConsumerPull;
use crate::engine::dataplane_manager::dataplane_handlers_consumer_push::DataplaneHandlerConsumerPush;
use crate::engine::dataplane_manager::dataplane_handlers_provider_pull::DataplaneHandlerProviderPull;
use crate::engine::dataplane_manager::dataplane_handlers_provider_push::DataplaneHandlerProviderPush;
use crate::entities::dataplane_transfers::{InteractionMode, TransferRole};
use crate::services::dataplane_transfers::DataplaneTransferServiceTrait;
use common::config::services::TransferConfig;
use connector::ConnectorInstanceFacadeTrait;
use keystore::SecretStore;
use std::sync::Arc;

pub struct DataplaneStrategyFactory {
    dataplane_service: Arc<dyn DataplaneTransferServiceTrait>,
    connector_service: Arc<dyn ConnectorInstanceFacadeTrait>,
    config: Arc<TransferConfig>,
    secret_store: Option<Arc<dyn SecretStore>>,
}
impl DataplaneStrategyFactory {
    pub fn new(
        dataplane_service: Arc<dyn DataplaneTransferServiceTrait>,
        connector_service: Arc<dyn ConnectorInstanceFacadeTrait>,
        config: Arc<TransferConfig>,
        secret_store: Option<Arc<dyn SecretStore>>,
    ) -> Self {
        Self {
            dataplane_service,
            connector_service,
            config,
            secret_store,
        }
    }
    pub fn get_strategy(
        &self,
        context: &DataplaneContext,
    ) -> Box<dyn DataplaneCommandStateMachine> {
        let role = context.dataplane_process_role();
        let interaction_mode = context.dataplane_process_interaction_mode();
        match (role, interaction_mode) {
            (TransferRole::Provider, InteractionMode::Pull) => {
                Box::new(DataplaneHandlerProviderPull::new(
                    self.dataplane_service.clone(),
                    self.connector_service.clone(),
                    self.config.clone(),
                    self.secret_store.clone(),
                ))
            }
            (TransferRole::Provider, InteractionMode::Push) => {
                Box::new(DataplaneHandlerProviderPush::new(
                    self.dataplane_service.clone(),
                    self.connector_service.clone(),
                    self.config.clone(),
                    self.secret_store.clone(),
                ))
            }
            (TransferRole::Consumer, InteractionMode::Pull) => {
                Box::new(DataplaneHandlerConsumerPull::new(
                    self.dataplane_service.clone(),
                    self.connector_service.clone(),
                    self.config.clone(),
                    self.secret_store.clone(),
                ))
            }
            (TransferRole::Consumer, InteractionMode::Push) => {
                Box::new(DataplaneHandlerConsumerPush::new(
                    self.dataplane_service.clone(),
                    self.connector_service.clone(),
                    self.config.clone(),
                    self.secret_store.clone(),
                ))
            }
        }
    }
}
