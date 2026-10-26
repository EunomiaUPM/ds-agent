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

//! The catalog's cache: profile projections, stamped with a generation every write bumps, and
//! the catalogs fetched from peers. The database stays the source of truth; a failing cache is
//! a miss, never an error of the request.

pub mod noop;
pub mod redis_cache;

use serde_json::Value;
use ymir::errors::Outcome;

pub use noop::NoopCatalogCache;
pub use redis_cache::RedisCatalogCache;

/// Cached projections and peer catalogs.
#[mockall::automock]
#[async_trait::async_trait]
pub trait CatalogCacheTrait: Send + Sync {
    /// The current generation; projections cached under an older one are never read.
    async fn generation(&self) -> u64;
    /// Moves to a new generation: every cached projection is stale from now on.
    async fn invalidate(&self);
    async fn get_projection(&self, key: &str) -> Option<Value>;
    async fn put_projection(&self, key: &str, projection: &Value);
    async fn get_peer_catalog(&self, user_id: &str, peer_id: &str) -> Outcome<Option<Value>>;
    async fn put_peer_catalog(&self, user_id: &str, peer_id: &str, catalog: &Value) -> Outcome<()>;
}

/// Keys of cached projections.
pub struct ProjectionKey;

impl ProjectionKey {
    /// One profile's projection of one root, for one kind of reader, at one generation.
    pub fn of(profile: &str, root: &str, reader: &str, generation: u64) -> String {
        format!("{profile}:{root}:{reader}:{generation}")
    }
}

/// Building the cache the configuration asks for.
pub struct CatalogCacheFactory;

impl CatalogCacheFactory {
    /// Redis, or no cache at all, as `cache_type` says.
    pub async fn from_config(
        config: &common::config::services::CatalogConfig,
    ) -> Outcome<std::sync::Arc<dyn CatalogCacheTrait>> {
        use common::config::types::cache::CacheType;
        use common::config::types::traits::CacheConfigTrait;
        use ymir::errors::Errors;

        match config.cache_config().cache_type {
            CacheType::Redis => {
                let connection = ::redis::Client::open(config.get_full_cache_url()?)
                    .map_err(|e| Errors::crazy("Error creating Redis client", Some(Box::new(e))))?
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|e| Errors::crazy("Redis connection failed", Some(Box::new(e))))?;
                Ok(std::sync::Arc::new(RedisCatalogCache::new(connection)))
            }
            CacheType::Noop => Ok(std::sync::Arc::new(NoopCatalogCache)),
            ref other => Err(Errors::crazy(
                format!("cache type {other:?} is not supported by the catalog agent"),
                None,
            )),
        }
    }
}
