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

use crate::config::types::cache::{CacheConfig, CacheType};

pub trait CacheConfigTrait {
    fn cache_config(&self) -> &CacheConfig;
    fn get_full_cache_url(&self) -> String {
        match self.cache_config().cache_type {
            CacheType::Redis => {
                format!(
                    "{}://{}:{}@{}:{}",
                    self.cache_config().cache_type,
                    self.cache_config().user,
                    self.cache_config().password,
                    self.cache_config().url,
                    self.cache_config().port
                )
            }
            _ => todo!("not implemented yet"),
        }
    }
}
