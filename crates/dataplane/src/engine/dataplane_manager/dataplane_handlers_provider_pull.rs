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
    set_configuring_helper, DataplaneCommandStateMachine, DataplaneInitCommandTypes,
};
use crate::engine::dataplane_manager::dataplane_context::DataplaneContext;
use crate::services::dataplane_transfers::DataplaneTransferServiceTrait;
use common::config::services::TransferConfig;
use connector::ConnectorInstanceFacadeTrait;
use keystore::SecretStore;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::{Errors, Outcome};

pub struct DataplaneHandlerProviderPull {
    dataplane_service: Arc<dyn DataplaneTransferServiceTrait>,
    connector_service: Arc<dyn ConnectorInstanceFacadeTrait>,
    config: Arc<TransferConfig>,
    secret_store: Option<Arc<dyn SecretStore>>,
}

impl DataplaneHandlerProviderPull {
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
}

#[async_trait::async_trait]
impl DataplaneCommandStateMachine for DataplaneHandlerProviderPull {
    fn handler_name(&self) -> &'static str {
        "ProviderPull"
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

    #[tracing::instrument(level = "info", skip_all, err, fields(handler = self.handler_name()))]
    async fn set_init(&self, context: DataplaneContext) -> Outcome<DataplaneContext> {
        let ctx = set_configuring_helper(
            self.dataplane_service(),
            self.driver_factory().as_ref(),
            context,
        )
        .await?;
        let ctx = self.set_auth(ctx).await?;
        let ctx = self.set_ready(ctx).await?;
        Ok(ctx)
    }
    #[tracing::instrument(level = "info", skip_all, err, fields(handler = self.handler_name()))]
    async fn set_configuring(&self, context: DataplaneContext) -> Outcome<DataplaneContext> {
        let ctx = set_configuring_helper(
            self.dataplane_service(),
            self.driver_factory().as_ref(),
            context,
        )
        .await?;
        let ctx = self.set_auth(ctx).await?;
        let ctx = self.set_ready(ctx).await?;
        let ctx = self.set_started(ctx).await?;
        Ok(ctx)
    }
}
