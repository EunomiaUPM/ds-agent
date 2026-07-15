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

use crate::data::repo::config::KeystoreConfigRepo;
use common::config::ApplicationConfig;
use common::config::services::{
    CatalogConfig, ContractsConfig, GatewayConfig, MonolithConfig, TransferConfig,
};
use std::ops::Deref;
use std::sync::Arc;
use ymir::errors::Outcome;

pub struct ConfigPassthroughRepo {
    config: Arc<ApplicationConfig>,
}

impl ConfigPassthroughRepo {
    pub fn new(config: Arc<ApplicationConfig>) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl KeystoreConfigRepo for ConfigPassthroughRepo {
    async fn get_transfer_config(&self) -> Outcome<TransferConfig> {
        Ok(self.config.transfer().clone())
    }

    async fn get_contracts_config(&self) -> Outcome<ContractsConfig> {
        Ok(self.config.contracts().clone())
    }

    async fn get_catalog_config(&self) -> Outcome<CatalogConfig> {
        Ok(self.config.catalog().clone())
    }

    async fn get_mono_config(&self) -> Outcome<MonolithConfig> {
        Ok(self.config.monolith().clone())
    }

    async fn get_gateway_config(&self) -> Outcome<GatewayConfig> {
        Ok(self.config.gateway().clone())
    }
    async fn get_config(&self) -> Outcome<ApplicationConfig> {
        Ok(self.config.deref().clone())
    }
}
