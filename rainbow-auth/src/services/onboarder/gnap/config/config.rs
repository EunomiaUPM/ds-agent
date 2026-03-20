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
use rainbow_common::config::types::traits::{CommonConfigTrait, EntityClientTrait};
use rainbow_common::config::types::EntityClientConfig;
use ymir::config::traits::{ApiConfigTrait, HostsConfigTrait};
use ymir::config::types::CommonHostsConfig;

use crate::services::onboarder::gnap::config::GnapOnboarderConfigTrait;

pub struct GnapOnboarderConfig {
    hosts: CommonHostsConfig,
    client: EntityClientConfig,
    api_path: String
}

impl From<SsiAuthConfig> for GnapOnboarderConfig {
    fn from(value: SsiAuthConfig) -> Self {
        GnapOnboarderConfig {
            hosts: value.common().hosts.clone(),
            client: value.client_config().clone(),
            api_path: value.common().get_api_version()
        }
    }
}

impl HostsConfigTrait for GnapOnboarderConfig {
    fn hosts(&self) -> &CommonHostsConfig { &self.hosts }
}

impl EntityClientTrait for GnapOnboarderConfig {
    fn client_config(&self) -> &EntityClientConfig { &self.client }
}

impl GnapOnboarderConfigTrait for GnapOnboarderConfig {
    fn get_api_path(&self) -> &str { &self.api_path }
}
