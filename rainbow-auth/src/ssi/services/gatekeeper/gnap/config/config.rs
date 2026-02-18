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
use ymir::config::traits::{ApiConfigTrait, ConnectionConfigTrait};
use ymir::config::types::CommonHostsConfig;

use crate::ssi::services::gatekeeper::gnap::config::GnapGateKeeperConfigTrait;

pub struct GnapGateKeeperConfig {
    hosts: CommonHostsConfig,
    is_local: bool,
    api_path: String,
}

impl From<SsiAuthConfig> for GnapGateKeeperConfig {
    fn from(value: SsiAuthConfig) -> Self {
        Self {
            hosts: value.common().hosts.clone(),
            is_local: value.common().is_local(),
            api_path: value.common().get_api_version(),
        }
    }
}

impl GnapGateKeeperConfigTrait for GnapGateKeeperConfig {
    fn hosts(&self) -> &CommonHostsConfig {
        &self.hosts
    }
    fn is_local(&self) -> bool {
        self.is_local
    }
    fn get_api_path(&self) -> String {
        self.api_path.clone()
    }
}
