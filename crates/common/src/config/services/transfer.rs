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

use serde::{Deserialize, Serialize};
use ymir::config::traits::ConnectionConfigTrait;
use ymir::config::types::ConnectionConfig;
use ymir::errors::Outcome;

use crate::config::services::traits::TransferConfigTrait;
use ymir::config::types::CommonConfig;
use crate::config::types::cache::CacheConfig;
use crate::config::types::min_known_config::MinKnownConfig;
use crate::config::types::traits::{CacheConfigTrait, GlobalConfigLoader};
use ymir::config::traits::{CommonConfigTrait, ConfigLoader};

/// `transfer` section: transfer agent config.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransferConfig {
    common: CommonConfig,
    cache: CacheConfig,
    contracts: MinKnownConfig,
    catalog: MinKnownConfig,
    is_catalog_datahub: bool,
    ssi_auth: MinKnownConfig,
    #[serde(default)]
    proxy_accept_invalid_certs: bool,
    #[serde(default)]
    proxy_allow_private_egress: bool,
    #[serde(default = "default_proxy_max_retries")]
    proxy_max_retries: u32,
}

const fn default_proxy_max_retries() -> u32 {
    3
}

impl TransferConfigTrait for TransferConfig {
    fn contracts(&self) -> &MinKnownConfig {
        &self.contracts
    }
    fn catalog(&self) -> &MinKnownConfig {
        &self.catalog
    }
    fn ssi_auth(&self) -> &MinKnownConfig {
        &self.ssi_auth
    }
    fn cache(&self) -> &CacheConfig {
        &self.cache
    }
    fn is_catalog_datahub(&self) -> bool {
        self.is_catalog_datahub
    }
    fn proxy_accept_invalid_certs(&self) -> bool {
        self.proxy_accept_invalid_certs
    }
    fn proxy_allow_private_egress(&self) -> bool {
        self.proxy_allow_private_egress
    }
    fn proxy_max_retries(&self) -> u32 {
        self.proxy_max_retries
    }
}
impl ConfigLoader for TransferConfig {
    fn load(env_file: &str) -> Outcome<Self> {
        Self::global_load(env_file)
            .and_then(|data| data.transfer().cloned())
            .or_else(|_| Self::local_load(env_file))
    }
}

impl CommonConfigTrait for TransferConfig {
    fn common(&self) -> &CommonConfig {
        &self.common
    }
}

impl CacheConfigTrait for TransferConfig {
    fn cache_config(&self) -> &CacheConfig {
        &self.cache
    }
}

impl ConnectionConfigTrait for TransferConfig {
    fn connection(&self) -> &ConnectionConfig {
        self.common.connection()
    }
}
