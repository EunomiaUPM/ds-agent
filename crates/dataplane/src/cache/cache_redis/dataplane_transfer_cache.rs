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

use crate::cache::cache_traits::redis_cache_connector_trait::RedisCacheConnectorTrait;
use crate::cache::cache_traits::utils_trait::UtilsCacheTrait;
use crate::entities::dataplane_transfers::DataplaneTransferDto;

pub struct DataplaneTransferCacheForRedis {
    pub redis_connection: redis::aio::MultiplexedConnection,
}

impl DataplaneTransferCacheForRedis {
    pub fn new(redis_connection: redis::aio::MultiplexedConnection) -> Self {
        Self { redis_connection }
    }
}

impl UtilsCacheTrait for DataplaneTransferCacheForRedis {
    type Dto = DataplaneTransferDto;
}

impl RedisCacheConnectorTrait for DataplaneTransferCacheForRedis {
    type Dto = DataplaneTransferDto;
    fn get_conn(&self) -> redis::aio::MultiplexedConnection {
        self.redis_connection.clone()
    }
    fn get_entity_name(&self) -> &str {
        "transfers"
    }
}
