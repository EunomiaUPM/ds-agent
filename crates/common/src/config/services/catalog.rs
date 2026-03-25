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

use crate::config::services::traits::CatalogConfigTrait;
use crate::config::services::CommonConfig;
use crate::config::types::cache::CacheConfig;
use crate::config::types::min_known_config::MinKnownConfig;
use crate::config::types::traits::{
    CacheConfigTrait, CommonConfigTrait, ConfigLoader, DatahubConfigTrait,
};
use crate::config::types::DatahubConfig;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CatalogConfig {
    common: CommonConfig,
    cache: CacheConfig,
    policy_templates_folder: Option<String>,
    datahub: Option<DatahubConfig>,
    ssi_auth: MinKnownConfig,
    contracts: MinKnownConfig,
}

impl DatahubConfigTrait for CatalogConfig {
    fn datahub(&self) -> &DatahubConfig {
        self.datahub.as_ref().expect("Datahub is not active")
    }
}

impl ConfigLoader for CatalogConfig {
    fn load(env_file: &str) -> Outcome<Self> {
        Self::global_load(env_file)
            .map(|data| data.catalog().clone())
            .or_else(|_| Self::local_load(env_file))
    }
}

impl CommonConfigTrait for CatalogConfig {
    fn common(&self) -> &CommonConfig {
        &self.common
    }
}

impl CacheConfigTrait for CatalogConfig {
    fn cache_config(&self) -> &CacheConfig {
        &self.cache
    }
}

impl CatalogConfigTrait for CatalogConfig {
    fn contracts(&self) -> &MinKnownConfig {
        &self.contracts
    }
    fn ssi_auth(&self) -> &MinKnownConfig {
        &self.ssi_auth
    }
    fn cache(&self) -> &CacheConfig {
        &self.cache
    }
    fn is_datahub(&self) -> bool {
        self.datahub.is_some()
    }

    fn get_policy_templates_folder(&self) -> &str {
        self.policy_templates_folder.as_deref().unwrap_or("/")
    }
}

impl ConnectionConfigTrait for CatalogConfig {
    fn connection(&self) -> &ConnectionConfig {
        self.common.connection()
    }
}
