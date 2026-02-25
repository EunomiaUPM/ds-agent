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

use crate::config::services::traits::MonolithConfigTrait;
use crate::config::services::CommonConfig;
use crate::config::types::cache::CacheConfig;
use crate::config::types::traits::{CacheConfigTrait, CommonConfigTrait, ConfigLoader};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MonolithConfig {
    common: CommonConfig,
    #[serde(default)]
    pub cache: CacheConfig,
}

impl MonolithConfig {
    pub fn new(common_config: CommonConfig) -> Self {
        Self { common: common_config, cache: CacheConfig::default() }
    }
}

impl ConfigLoader for MonolithConfig {
    fn load(env_file: &str) -> Self {
        Self::global_load(env_file)
            .map(|data| data.monolith().clone())
            .unwrap_or(Self::local_load(env_file).expect("Unable to load Monolith config"))
    }
}

impl MonolithConfigTrait for MonolithConfig {}

impl CommonConfigTrait for MonolithConfig {
    fn common(&self) -> &CommonConfig {
        &self.common
    }
}
impl CacheConfigTrait for MonolithConfig {
    fn cache_config(&self) -> &CacheConfig {
        &self.cache
    }
}
