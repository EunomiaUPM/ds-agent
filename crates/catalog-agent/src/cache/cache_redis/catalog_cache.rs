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

use crate::cache::cache_traits::DESIRED_CACHE_TTL;
use crate::CatalogDto;
use async_trait::async_trait;
use common::cache::{RedisCacheConnectorTrait, UtilsCacheTrait};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use urn::Urn;

pub struct CatalogCacheForRedis {
    pub redis_connection: redis::aio::MultiplexedConnection,
}

impl CatalogCacheForRedis {
    pub fn new(redis_connection: redis::aio::MultiplexedConnection) -> Self {
        Self { redis_connection }
    }
}

impl UtilsCacheTrait for CatalogCacheForRedis {
    type Dto = CatalogDto;

    fn key_namespace(&self) -> &str {
        "ds_agent_catalogs"
    }
}
impl RedisCacheConnectorTrait for CatalogCacheForRedis {
    type Dto = CatalogDto;
    fn get_conn(&self) -> redis::aio::MultiplexedConnection {
        self.redis_connection.clone()
    }
    fn get_entity_name(&self) -> &str {
        "catalogs"
    }
    fn cache_ttl(&self) -> i32 {
        DESIRED_CACHE_TTL
    }
}
