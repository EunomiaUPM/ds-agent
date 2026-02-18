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

use crate::config::services::CommonConfig;
use crate::config::traits::{CacheConfigTrait, CommonConfigTrait};
use crate::config::types::cache::CacheConfig;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MonolithConfig {
    common: CommonConfig,
    #[serde(default)]
    pub cache: CacheConfig,
}

impl MonolithConfig {
    pub fn new(common_config: CommonConfig) -> Self {
        Self { 
            common: common_config,
            cache: CacheConfig::default(), 
        }
    }
}

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
