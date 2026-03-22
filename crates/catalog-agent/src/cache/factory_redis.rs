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

use crate::cache::cache_redis::catalog_cache::CatalogCacheForRedis;
use crate::cache::cache_redis::dataservice_cache::DataServiceCacheForRedis;
use crate::cache::cache_redis::dataset_cache::DatasetCacheForRedis;
use crate::cache::cache_redis::distribution_cache::DistributionCacheForRedis;
use crate::cache::cache_redis::odrl_offer_cache::OdrlOfferCacheForRedis;
use crate::cache::cache_redis::peer_catalog_cache::DcatCatalogCacheForRedis;
use crate::cache::cache_traits::entity_cache_trait::EntityCacheTrait;
use crate::cache::cache_traits::peer_catalog_cache_trait::PeerCatalogCacheTrait;
use crate::cache::factory_trait::CatalogAgentCacheTrait;
use crate::data::factory_trait::CatalogAgentRepoTrait;
use crate::data::repo_traits::catalog_repo::CatalogRepositoryTrait;
use crate::data::repo_traits::dataservice_repo::DataServiceRepositoryTrait;
use crate::data::repo_traits::dataset_repo::DatasetRepositoryTrait;
use crate::data::repo_traits::distribution_repo::DistributionRepositoryTrait;
use crate::data::repo_traits::odrl_offer_repo::OdrlOfferRepositoryTrait;
use crate::data::repo_traits::policy_template_repo::PolicyTemplatesRepositoryTrait;
use crate::data::repos_sql::catalog_repo::CatalogRepositoryForSql;
use crate::data::repos_sql::dataservice_repo::DataServiceRepositoryForSql;
use crate::data::repos_sql::dataset_repo::DatasetRepositoryForSql;
use crate::data::repos_sql::distribution_repo::DistributionRepositoryForSql;
use crate::data::repos_sql::odrl_offer_repo::OdrlOfferRepositoryForSql;
use crate::data::repos_sql::policy_template_repo::PolicyTemplatesRepositoryForSql;
use crate::{CatalogDto, DataServiceDto, DatasetDto, DistributionDto, OdrlPolicyDto};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use urn::Urn;

pub struct CatalogAgentCacheForRedis {
    catalog_repo: Arc<dyn EntityCacheTrait<CatalogDto>>,
    dataservice_repo: Arc<dyn EntityCacheTrait<DataServiceDto>>,
    dataset_repo: Arc<dyn EntityCacheTrait<DatasetDto>>,
    distribution_repo: Arc<dyn EntityCacheTrait<DistributionDto>>,
    odrl_offer_repo: Arc<dyn EntityCacheTrait<OdrlPolicyDto>>,
    peer_catalog_repo: Arc<dyn PeerCatalogCacheTrait>,
}

impl CatalogAgentCacheForRedis {
    pub fn create_repo(db_connection: redis::aio::MultiplexedConnection) -> Self {
        Self {
            catalog_repo: Arc::new(CatalogCacheForRedis::new(db_connection.clone())),
            dataservice_repo: Arc::new(DataServiceCacheForRedis::new(db_connection.clone())),
            dataset_repo: Arc::new(DatasetCacheForRedis::new(db_connection.clone())),
            distribution_repo: Arc::new(DistributionCacheForRedis::new(db_connection.clone())),
            odrl_offer_repo: Arc::new(OdrlOfferCacheForRedis::new(db_connection.clone())),
            peer_catalog_repo: Arc::new(DcatCatalogCacheForRedis::new(db_connection.clone())),
        }
    }
}

impl CatalogAgentCacheTrait for CatalogAgentCacheForRedis {
    fn get_catalog_cache(&self) -> Arc<dyn EntityCacheTrait<CatalogDto>> {
        self.catalog_repo.clone()
    }

    fn get_dataservice_cache(&self) -> Arc<dyn EntityCacheTrait<DataServiceDto>> {
        self.dataservice_repo.clone()
    }

    fn get_dataset_cache(&self) -> Arc<dyn EntityCacheTrait<DatasetDto>> {
        self.dataset_repo.clone()
    }

    fn get_distribution_cache(&self) -> Arc<dyn EntityCacheTrait<DistributionDto>> {
        self.distribution_repo.clone()
    }

    fn get_odrl_offer_cache(&self) -> Arc<dyn EntityCacheTrait<OdrlPolicyDto>> {
        self.odrl_offer_repo.clone()
    }

    fn get_peer_catalog_cache(&self) -> Arc<dyn PeerCatalogCacheTrait> {
        self.peer_catalog_repo.clone()
    }
}
