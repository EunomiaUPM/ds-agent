/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tracing::debug;
use ymir::config::traits::ConnectionConfigTrait;
use ymir::config::types::ConnectionConfig;
use ymir::errors::{Errors, Outcome};
use ymir::utils::read;

use crate::config::services::traits::CatalogConfigTrait;
use crate::config::services::{
    CatalogConfig, CommonConfig, ContractsConfig, GatewayConfig, MonolithConfig, SsiAuthConfig,
    TransferConfig,
};
use crate::config::types::traits::CommonConfigTrait;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApplicationConfig {
    monolith: Option<MonolithConfig>,
    transfer: Option<TransferConfig>,
    contracts: Option<ContractsConfig>,
    catalog: Option<CatalogConfig>,
    ssi_auth: Option<SsiAuthConfig>,
    gateway: Option<GatewayConfig>,
}

impl ApplicationConfig {
    pub fn new(common_config: CommonConfig) -> Self {
        Self {
            monolith: Some(MonolithConfig::new(common_config)),
            transfer: None,
            contracts: None,
            catalog: None,
            ssi_auth: None,
            gateway: None,
        }
    }
}

impl ApplicationConfig {
    pub fn is_mono_catalog_datahub(&self) -> bool {
        self.catalog
            .as_ref()
            .map(|catalog| catalog.is_datahub())
            .unwrap_or(false)
    }
}

impl ApplicationConfig {
    pub fn ssi_auth(&self) -> &SsiAuthConfig {
        self.ssi_auth
            .as_ref()
            .expect("Missing SSI Authentication Config")
    }
    pub fn transfer(&self) -> &TransferConfig {
        self.transfer.as_ref().expect("Missing Transfer Config")
    }
    pub fn contracts(&self) -> &ContractsConfig {
        self.contracts.as_ref().expect("Missing Contracts Config")
    }
    pub fn catalog(&self) -> &CatalogConfig {
        self.catalog.as_ref().expect("Missing Catalog Config")
    }
    pub fn gateway(&self) -> &GatewayConfig {
        self.gateway.as_ref().expect("Missing Gateway Config")
    }

    pub fn monolith(&self) -> &MonolithConfig {
        self.monolith.as_ref().expect("Missing Monolith Config")
    }

    pub fn load(env_file: &str) -> Outcome<Self> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(env_file);
        debug!("Config file path: {}", path.display());

        let data = read(path)?;
        serde_norway::from_str(&data)
            .map_err(|e| Errors::parse("Unable to parse config file", Some(Box::new(e))))
    }
}

impl ConnectionConfigTrait for ApplicationConfig {
    fn connection(&self) -> &ConnectionConfig {
        self.monolith().common().connection()
    }
}
