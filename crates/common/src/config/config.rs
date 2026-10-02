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
use crate::config::types::traits::{CommonConfigTrait, ConfigLoader};

/// The whole config file of the monolith: its own section plus one optional section per agent.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApplicationConfig {
    monolith: MonolithConfig,
    transfer: Option<TransferConfig>,
    contracts: Option<ContractsConfig>,
    catalog: Option<CatalogConfig>,
    ssi_auth: Option<SsiAuthConfig>,
    gateway: Option<GatewayConfig>,
}

impl ApplicationConfig {
    /// Config with only the monolith section, for tests and tools.
    pub fn new(common_config: CommonConfig) -> Self {
        Self {
            monolith: MonolithConfig::new(common_config),
            transfer: None,
            contracts: None,
            catalog: None,
            ssi_auth: None,
            gateway: None,
        }
    }
}

impl ApplicationConfig {
    /// Whether the catalog is backed by a datahub.
    pub fn is_mono_catalog_datahub(&self) -> bool {
        self.catalog
            .as_ref()
            .map(|catalog| catalog.is_datahub())
            .unwrap_or(false)
    }
}

impl ApplicationConfig {
    pub fn ssi_auth(&self) -> Outcome<&SsiAuthConfig> {
        Self::section(self.ssi_auth.as_ref(), "ssi_auth")
    }
    pub fn transfer(&self) -> Outcome<&TransferConfig> {
        Self::section(self.transfer.as_ref(), "transfer")
    }
    pub fn contracts(&self) -> Outcome<&ContractsConfig> {
        Self::section(self.contracts.as_ref(), "contracts")
    }
    pub fn catalog(&self) -> Outcome<&CatalogConfig> {
        Self::section(self.catalog.as_ref(), "catalog")
    }
    pub fn gateway(&self) -> Outcome<&GatewayConfig> {
        Self::section(self.gateway.as_ref(), "gateway")
    }
    pub fn monolith(&self) -> &MonolithConfig {
        &self.monolith
    }

    /// Fails unless every agent section the monolith composes is present.
    pub fn validate_monolith(&self) -> Outcome<()> {
        let missing: Vec<&str> = [
            ("ssi_auth", self.ssi_auth.is_none()),
            ("transfer", self.transfer.is_none()),
            ("contracts", self.contracts.is_none()),
            ("catalog", self.catalog.is_none()),
            ("gateway", self.gateway.is_none()),
        ]
        .into_iter()
        .filter_map(|(name, absent)| absent.then_some(name))
        .collect();
        if missing.is_empty() {
            Ok(())
        } else {
            Err(Errors::parse(
                format!("config file lacks sections: {}", missing.join(", ")),
                None,
            ))
        }
    }

    fn section<'a, T>(section: Option<&'a T>, name: &str) -> Outcome<&'a T> {
        section
            .ok_or_else(|| Errors::parse(format!("config file lacks the `{name}` section"), None))
    }

    /// Reads and parses the YAML file; a relative path is resolved from the `common` crate.
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

/// The monolith composes every agent, so its config must carry every section.
impl ConfigLoader for ApplicationConfig {
    fn load(env_file: &str) -> Outcome<Self> {
        let config = ApplicationConfig::load(env_file)?;
        config.validate_monolith()?;
        Ok(config)
    }
}

/// The monolith's own common section drives process-wide concerns (ports, vault, DB).
impl CommonConfigTrait for ApplicationConfig {
    fn common(&self) -> &CommonConfig {
        self.monolith().common()
    }
}
