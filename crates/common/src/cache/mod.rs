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

//! Redis-backed entity cache traits shared across service crates.

pub mod entity_cache_trait;
pub mod lookup_cache_trait;
pub mod redis_cache_connector_trait;
pub mod utils_trait;

pub use entity_cache_trait::{EntityCacheTrait, NoopCache};
pub use lookup_cache_trait::LookupCacheTrait;
pub use redis_cache_connector_trait::RedisCacheConnectorTrait;
pub use utils_trait::UtilsCacheTrait;

/// Default entity lifetime in the cache: one day.
pub const DEFAULT_CACHE_TTL: i32 = 86400;
