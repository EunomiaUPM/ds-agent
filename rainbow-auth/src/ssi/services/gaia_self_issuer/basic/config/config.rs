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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use rainbow_common::config::services::SsiAuthConfig;
use rainbow_common::config::traits::CommonConfigTrait;
use rainbow_common::config::types::ClientConfig;
use ymir::config::traits::{ApiConfigTrait, ConnectionConfigTrait, VcConfigTrait};
use ymir::config::types::{CommonHostsConfig, DidConfig, HostConfig};
use ymir::types::vcs::W3cDataModelVersion;

use super::GaiaGaiaSelfIssuerConfigTrait;

pub struct GaiaSelfIssuerConfig {
    hosts: CommonHostsConfig,
    is_local: bool,
    api_path: String,
    vc_data_model: W3cDataModelVersion,
    did_config: DidConfig,
    client_config: ClientConfig,
    gaia_api: HostConfig,
}

impl From<SsiAuthConfig> for GaiaSelfIssuerConfig {
    fn from(value: SsiAuthConfig) -> Self {
        Self {
            hosts: value.common().hosts.clone(),
            is_local: value.common().is_local(),
            api_path: value.common().get_api_version(),
            vc_data_model: value.vc_config().get_w3c_data_model().unwrap().clone(),
            did_config: value.did_config().clone(),
            client_config: value.client_config().clone(),
            gaia_api: value.gaia_config().api.clone(),
        }
    }
}

impl GaiaGaiaSelfIssuerConfigTrait for GaiaSelfIssuerConfig {
    fn hosts(&self) -> &CommonHostsConfig {
        &self.hosts
    }
    fn gaia_api(&self) -> &HostConfig {
        &self.gaia_api
    }
    fn is_local(&self) -> bool {
        self.is_local
    }
    fn get_api_path(&self) -> String {
        self.api_path.clone()
    }
    fn get_data_model_version(&self) -> W3cDataModelVersion {
        self.vc_data_model.clone()
    }
    fn get_did(&self) -> String {
        self.did_config.did.clone()
    }
    fn get_client_config(&self) -> &ClientConfig {
        &self.client_config
    }
}
