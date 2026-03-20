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

use crate::data::repo_traits::catalog_repo::CatalogRepositoryTrait;
use crate::data::repo_traits::dataservice_repo::DataServiceRepositoryTrait;
use crate::data::repo_traits::dataset_repo::DatasetRepositoryTrait;
use crate::data::repo_traits::distribution_repo::DistributionRepositoryTrait;
use crate::data::repo_traits::odrl_offer_repo::OdrlOfferRepositoryTrait;
use crate::data::repo_traits::policy_template_repo::PolicyTemplatesRepositoryTrait;
use std::sync::Arc;

#[mockall::automock]
pub trait CatalogAgentRepoTrait: Send + Sync + 'static {
    fn get_catalog_repo(&self) -> Arc<dyn CatalogRepositoryTrait>;
    fn get_dataservice_repo(&self) -> Arc<dyn DataServiceRepositoryTrait>;
    fn get_dataset_repo(&self) -> Arc<dyn DatasetRepositoryTrait>;
    fn get_distribution_repo(&self) -> Arc<dyn DistributionRepositoryTrait>;
    fn get_odrl_offer_repo(&self) -> Arc<dyn OdrlOfferRepositoryTrait>;
    fn get_policy_template_repo(&self) -> Arc<dyn PolicyTemplatesRepositoryTrait>;
}
