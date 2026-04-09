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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use common::config::services::SsiAuthConfig;
use common::config::types::traits::{CommonConfigTrait, EntityClientTrait, GaiaConfigTrait};
use common::config::types::{EntityClientConfig, GaiaConfig};
use ymir::config::traits::{
    ConnectionConfigTrait, DidConfigTrait, HostsConfigTrait, VcConfigTrait,
};
use ymir::config::types::CommonHostsConfig;
use ymir::types::vcs::W3cDataModelVersion;

use super::GaiaGaiaSelfIssuerConfigTrait;

pub struct GaiaSelfIssuerConfig {
    hosts: CommonHostsConfig,
    is_local: bool,
    vc_data_model: W3cDataModelVersion,
    did: String,
    client_config: EntityClientConfig,
    gaia_config: GaiaConfig,
}

impl From<SsiAuthConfig> for GaiaSelfIssuerConfig {
    fn from(value: SsiAuthConfig) -> Self {
        Self {
            hosts: value.common().hosts().clone(),
            is_local: value.common().is_local(),
            vc_data_model: value
                .vc_config()
                .get_w3c_data_model()
                .expect("Gaia Config is based on w3c data model")
                .clone(),
            did: value.get_did().to_string(),
            client_config: value.client_config().clone(),
            gaia_config: value.gaia_config().clone(),
        }
    }
}

impl HostsConfigTrait for GaiaSelfIssuerConfig {
    fn hosts(&self) -> &CommonHostsConfig {
        &self.hosts
    }
}

impl GaiaConfigTrait for GaiaSelfIssuerConfig {
    fn gaia_config(&self) -> &GaiaConfig {
        &self.gaia_config
    }
}

impl EntityClientTrait for GaiaSelfIssuerConfig {
    fn client_config(&self) -> &EntityClientConfig {
        &self.client_config
    }
}

impl GaiaGaiaSelfIssuerConfigTrait for GaiaSelfIssuerConfig {
    fn is_local(&self) -> bool {
        self.is_local
    }
    fn get_data_model_version(&self) -> &W3cDataModelVersion {
        &self.vc_data_model
    }
    fn get_did(&self) -> &str {
        &self.did
    }
}
