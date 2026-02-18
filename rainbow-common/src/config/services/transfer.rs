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

use crate::config::services::{CommonConfig, MinKnownConfig};
use crate::config::traits::{CacheConfigTrait, CommonConfigTrait, ConfigLoader};
use crate::config::types::cache::CacheConfig;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransferConfig {
    common: CommonConfig,
    cache: CacheConfig,
    contracts: MinKnownConfig,
    catalog: MinKnownConfig,
    is_catalog_datahub: bool,
    ssi_auth: MinKnownConfig,
}

impl TransferConfig {
    pub fn contracts(&self) -> &MinKnownConfig {
        &self.contracts
    }
    pub fn catalog(&self) -> &MinKnownConfig {
        &self.catalog
    }
    pub fn ssi_auth(&self) -> &MinKnownConfig {
        &self.ssi_auth
    }
    pub fn is_catalog_datahub(&self) -> bool {
        self.is_catalog_datahub
    }
    pub fn cache(&self) -> &CacheConfig {
        &self.cache
    }
}
impl ConfigLoader for TransferConfig {
    fn load(env_file: String) -> Self {
        match Self::global_load(env_file.clone()) {
            Ok(data) => data.transfer(),
            Err(_) => Self::local_load(env_file).expect("Unable to load catalog config"),
        }
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
