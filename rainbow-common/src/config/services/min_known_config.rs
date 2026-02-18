/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::config::services::{
    CatalogConfig, ContractsConfig, GatewayConfig, SsiAuthConfig, TransferConfig,
};
use crate::config::traits::CommonConfigTrait;
use crate::config::ApplicationConfig;
use serde::{Deserialize, Serialize};
use ymir::config::traits::{ApiConfigTrait, HostsConfigTrait};
use ymir::config::types::{CommonHostsConfig, HostType};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MinKnownConfig {
    pub hosts: CommonHostsConfig,
    pub api_version: String,
}

impl MinKnownConfig {
    pub fn get_host(&self, host_type: HostType) -> String {
        self.hosts.get_host(host_type)
    }
    pub fn get_api_version(&self) -> String {
        format!("/api/{}", self.api_version)
    }
}

impl From<SsiAuthConfig> for MinKnownConfig {
    fn from(value: SsiAuthConfig) -> Self {
        Self { hosts: value.common().hosts.clone(), api_version: value.common().get_api_version() }
    }
}

impl From<CatalogConfig> for MinKnownConfig {
    fn from(value: CatalogConfig) -> Self {
        Self { hosts: value.common().hosts.clone(), api_version: value.common().get_api_version() }
    }
}

impl From<&CatalogConfig> for MinKnownConfig {
    fn from(value: &CatalogConfig) -> Self {
        value.clone().into()
    }
}

impl From<ContractsConfig> for MinKnownConfig {
    fn from(value: ContractsConfig) -> Self {
        Self { hosts: value.common().hosts.clone(), api_version: value.common().get_api_version() }
    }
}

impl From<&ContractsConfig> for MinKnownConfig {
    fn from(value: &ContractsConfig) -> Self {
        value.clone().into()
    }
}

impl From<TransferConfig> for MinKnownConfig {
    fn from(value: TransferConfig) -> Self {
        Self { hosts: value.common().hosts.clone(), api_version: value.common().get_api_version() }
    }
}

impl From<&TransferConfig> for MinKnownConfig {
    fn from(value: &TransferConfig) -> Self {
        value.clone().into()
    }
}

impl From<GatewayConfig> for MinKnownConfig {
    fn from(value: GatewayConfig) -> Self {
        Self { hosts: value.common().hosts.clone(), api_version: value.common().get_api_version() }
    }
}

impl From<&GatewayConfig> for MinKnownConfig {
    fn from(value: &GatewayConfig) -> Self {
        value.clone().into()
    }
}

impl From<ApplicationConfig> for MinKnownConfig {
    fn from(value: ApplicationConfig) -> Self {
        Self {
            hosts: value.monolith().common().hosts.clone(),
            api_version: value.monolith().common().get_api_version(),
        }
    }
}

impl From<&ApplicationConfig> for MinKnownConfig {
    fn from(value: &ApplicationConfig) -> Self {
        value.clone().into()
    }
}
