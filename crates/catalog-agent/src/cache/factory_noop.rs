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

//! Cache factory for `cache_type: Noop`: every read misses and every write is dropped.

use std::sync::Arc;

use common::cache::{EntityCacheTrait, NoopCache};

use crate::cache::cache_noop::NoopPeerCatalogCache;
use crate::cache::cache_traits::peer_catalog_cache_trait::PeerCatalogCacheTrait;
use crate::cache::factory_trait::CatalogAgentCacheTrait;
use crate::{CatalogDto, DataServiceDto, DatasetDto, DistributionDto, OdrlPolicyDto};

/// Caches that hold nothing, so the agent runs straight on its database.
pub struct CatalogAgentCacheNoop;

impl CatalogAgentCacheTrait for CatalogAgentCacheNoop {
    fn get_catalog_cache(&self) -> Arc<dyn EntityCacheTrait<CatalogDto>> {
        Arc::new(NoopCache::new())
    }
    fn get_dataservice_cache(&self) -> Arc<dyn EntityCacheTrait<DataServiceDto>> {
        Arc::new(NoopCache::new())
    }
    fn get_dataset_cache(&self) -> Arc<dyn EntityCacheTrait<DatasetDto>> {
        Arc::new(NoopCache::new())
    }
    fn get_distribution_cache(&self) -> Arc<dyn EntityCacheTrait<DistributionDto>> {
        Arc::new(NoopCache::new())
    }
    fn get_odrl_offer_cache(&self) -> Arc<dyn EntityCacheTrait<OdrlPolicyDto>> {
        Arc::new(NoopCache::new())
    }
    fn get_peer_catalog_cache(&self) -> Arc<dyn PeerCatalogCacheTrait> {
        Arc::new(NoopPeerCatalogCache)
    }
}
