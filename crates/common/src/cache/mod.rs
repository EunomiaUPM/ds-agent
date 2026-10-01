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
//!
//! A crate gets a full entity cache by implementing two small traits on a struct that holds a
//! Redis connection: [`RedisCacheConnectorTrait`] (connection, entity name, TTL) and
//! [`UtilsCacheTrait`] (key namespace). Blanket impls then provide [`EntityCacheTrait`] (single
//! entities, a per-tenant "main" entity, a sorted collection, batches) and [`LookupCacheTrait`]
//! (parent-to-child indexes). Entities are stored as RedisJSON documents, so Redis needs the
//! JSON module. [`NoopCache`] does nothing and stands in when the cache is disabled.
//!
//! The database stays the source of truth: services write through to the cache after a
//! successful write and ignore cache failures.
//!
//! ## 1. Declaring a cache
//!
//! ```rust,ignore
//! use common::cache::{RedisCacheConnectorTrait, UtilsCacheTrait};
//!
//! pub struct CatalogCacheForRedis {
//!     pub redis_connection: redis::aio::MultiplexedConnection,
//! }
//!
//! impl UtilsCacheTrait for CatalogCacheForRedis {
//!     type Dto = CatalogDto;
//!
//!     fn key_namespace(&self) -> &str {
//!         "ds_agent_catalogs"
//!     }
//! }
//!
//! impl RedisCacheConnectorTrait for CatalogCacheForRedis {
//!     type Dto = CatalogDto;
//!
//!     fn get_conn(&self) -> redis::aio::MultiplexedConnection {
//!         self.redis_connection.clone()
//!     }
//!
//!     fn get_entity_name(&self) -> &str {
//!         "catalogs"
//!     }
//! }
//! ```
//!
//! That is all: `CatalogCacheForRedis` is now an `EntityCacheTrait<CatalogDto>`. Override
//! `cache_ttl` to change the default lifetime of one day ([`DEFAULT_CACHE_TTL`]).
//!
//! ## 2. Using it from a service
//!
//! Services hold the cache as `Arc<dyn EntityCacheTrait<Dto>>`, so a [`NoopCache`] can replace
//! it. The collection is a sorted set; the score decides the order (usually a timestamp).
//!
//! ```rust,ignore
//! use common::cache::EntityCacheTrait;
//!
//! let dto: DatasetDto = dataset.into();
//! let cache = self.cache.get_dataset_cache();
//! let _ = cache.set_single(&id, &dto).await;
//! let _ = cache.add_to_collection(&id, dto.inner.dct_issued.timestamp() as f64).await;
//!
//! let cached = cache.get_single(&id).await?;              // Option<DatasetDto>
//! let main = cache.get_main(scope.acting_tenant()).await?; // the tenant's main catalog
//! let page = cache.get_collection(Some(25), Some(1)).await?;
//! ```
//!
//! ## 3. Relations
//!
//! [`LookupCacheTrait`] keeps one sorted set per parent, so children can be read without a
//! scan, for example the datasets of a catalog.
//!
//! ```rust,ignore
//! use common::cache::LookupCacheTrait;
//!
//! cache.add_to_relation("catalogs", &catalog_id, &dataset_id, score).await?;
//! let datasets = cache.get_by_relation("catalogs", &catalog_id, Some(25), Some(1)).await?;
//! ```
//!
//! ## 4. Keys
//!
//! | What | Key |
//! |---|---|
//! | One entity | `<namespace>:<entity>:<urn>` |
//! | Main entity of a tenant | `<namespace>:<entity>:main:<tenant>` (points to the entity key) |
//! | Collection | `<namespace>:<entity>:all` |
//! | Children of a parent | `<namespace>:<child>:<parent>:<parent urn>` |
//!
//! Entity keys are not scoped by tenant, so whatever is read from the cache still goes through
//! the scope checks of the service. The boot can flush Redis on start with
//! `crate::boot::seeders::RedisCacheFlush`.

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
