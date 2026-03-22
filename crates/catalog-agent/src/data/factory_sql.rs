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
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub struct CatalogAgentRepoForSql {
    catalog_repo: Arc<dyn CatalogRepositoryTrait>,
    dataservice_repo: Arc<dyn DataServiceRepositoryTrait>,
    dataset_repo: Arc<dyn DatasetRepositoryTrait>,
    distribution_repo: Arc<dyn DistributionRepositoryTrait>,
    odrl_offer_repo: Arc<dyn OdrlOfferRepositoryTrait>,
    policy_template_repo: Arc<dyn PolicyTemplatesRepositoryTrait>,
}

impl CatalogAgentRepoForSql {
    pub fn create_repo(db_connection: DatabaseConnection) -> Self {
        Self {
            catalog_repo: Arc::new(CatalogRepositoryForSql::new(db_connection.clone())),
            dataservice_repo: Arc::new(DataServiceRepositoryForSql::new(db_connection.clone())),
            dataset_repo: Arc::new(DatasetRepositoryForSql::new(db_connection.clone())),
            distribution_repo: Arc::new(DistributionRepositoryForSql::new(db_connection.clone())),
            odrl_offer_repo: Arc::new(OdrlOfferRepositoryForSql::new(db_connection.clone())),
            policy_template_repo: Arc::new(PolicyTemplatesRepositoryForSql::new(
                db_connection.clone(),
            )),
        }
    }
}

impl CatalogAgentRepoTrait for CatalogAgentRepoForSql {
    fn get_catalog_repo(&self) -> Arc<dyn CatalogRepositoryTrait> {
        self.catalog_repo.clone()
    }

    fn get_dataservice_repo(&self) -> Arc<dyn DataServiceRepositoryTrait> {
        self.dataservice_repo.clone()
    }

    fn get_dataset_repo(&self) -> Arc<dyn DatasetRepositoryTrait> {
        self.dataset_repo.clone()
    }

    fn get_distribution_repo(&self) -> Arc<dyn DistributionRepositoryTrait> {
        self.distribution_repo.clone()
    }

    fn get_odrl_offer_repo(&self) -> Arc<dyn OdrlOfferRepositoryTrait> {
        self.odrl_offer_repo.clone()
    }

    fn get_policy_template_repo(&self) -> Arc<dyn PolicyTemplatesRepositoryTrait> {
        self.policy_template_repo.clone()
    }
}
