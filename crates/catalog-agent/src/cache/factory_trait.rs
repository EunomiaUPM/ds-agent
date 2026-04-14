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

use crate::cache::cache_traits::entity_cache_trait::EntityCacheTrait;
use crate::cache::cache_traits::peer_catalog_cache_trait::PeerCatalogCacheTrait;
use crate::{CatalogDto, DataServiceDto, DatasetDto, DistributionDto, OdrlPolicyDto};
use std::sync::Arc;

#[mockall::automock]
pub trait CatalogAgentCacheTrait: Send + Sync + 'static {
    fn get_catalog_cache(&self) -> Arc<dyn EntityCacheTrait<CatalogDto>>;
    fn get_dataservice_cache(&self) -> Arc<dyn EntityCacheTrait<DataServiceDto>>;
    fn get_dataset_cache(&self) -> Arc<dyn EntityCacheTrait<DatasetDto>>;
    fn get_distribution_cache(&self) -> Arc<dyn EntityCacheTrait<DistributionDto>>;
    fn get_odrl_offer_cache(&self) -> Arc<dyn EntityCacheTrait<OdrlPolicyDto>>;
    fn get_peer_catalog_cache(&self) -> Arc<dyn PeerCatalogCacheTrait>;
}
