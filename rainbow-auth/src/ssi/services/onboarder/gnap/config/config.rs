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
use ymir::config::traits::ApiConfigTrait;
use ymir::config::types::CommonHostsConfig;
use ymir::types::gnap::grant_request::Client4GR;

use crate::ssi::services::onboarder::gnap::config::GnapOnboarderConfigTrait;
use crate::ssi::utils::get_pretty_client_config_helper;

pub struct GnapOnboarderConfig {
    host: CommonHostsConfig,
    client: ClientConfig,
    api_path: String,
}

impl From<SsiAuthConfig> for GnapOnboarderConfig {
    fn from(value: SsiAuthConfig) -> Self {
        GnapOnboarderConfig {
            host: value.common().hosts.clone(),
            client: value.client_config(),
            api_path: value.common().get_api_version(),
        }
    }
}

impl GnapOnboarderConfigTrait for GnapOnboarderConfig {
    fn get_pretty_client_config(&self, cert: &str) -> anyhow::Result<Client4GR> {
        get_pretty_client_config_helper(&self.client, &cert)
    }
    fn hosts(&self) -> &CommonHostsConfig {
        &self.host
    }
    fn get_api_path(&self) -> String {
        self.api_path.clone()
    }
}
