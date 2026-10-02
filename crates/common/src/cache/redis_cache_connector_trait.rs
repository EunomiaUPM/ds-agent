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

//! Connection and naming contract a Redis-backed entity cache must provide.

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::cache::DEFAULT_CACHE_TTL;

/// Connection and naming a Redis entity cache provides; the cache operations come for free.
#[async_trait::async_trait]
pub trait RedisCacheConnectorTrait: Send + Sync {
    /// Type stored in the cache.
    type Dto: Serialize + DeserializeOwned + Send + Sync;

    /// Connection for one operation; multiplexed connections are cheap to clone.
    fn get_conn(&self) -> redis::aio::MultiplexedConnection;
    /// Entity segment of every key, e.g. `catalogs`.
    fn get_entity_name(&self) -> &str;

    /// Seconds an entity written by this cache stays alive.
    fn cache_ttl(&self) -> i32 {
        DEFAULT_CACHE_TTL
    }
}
