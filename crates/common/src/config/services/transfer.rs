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

use serde::{Deserialize, Serialize};
use ymir::config::traits::ConnectionConfigTrait;
use ymir::config::types::ConnectionConfig;
use ymir::errors::Outcome;

use crate::config::services::traits::TransferConfigTrait;
use crate::config::services::CommonConfig;
use crate::config::types::cache::CacheConfig;
use crate::config::types::min_known_config::MinKnownConfig;
use crate::config::types::traits::{CacheConfigTrait, CommonConfigTrait, ConfigLoader};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransferConfig {
    common: CommonConfig,
    cache: CacheConfig,
    contracts: MinKnownConfig,
    catalog: MinKnownConfig,
    is_catalog_datahub: bool,
    ssi_auth: MinKnownConfig,
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
}
impl ConfigLoader for TransferConfig {
    fn load(env_file: &str) -> Outcome<Self> {
        Self::global_load(env_file)
            .map(|data| data.transfer().clone())
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
