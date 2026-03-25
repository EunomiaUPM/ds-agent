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

use crate::data::entities::distribution;
use crate::data::entities::distribution::{EditDistributionModel, NewDistributionModel};
use crate::data::repo_traits::catalog_db_errors::CatalogAgentRepoErrors;
use urn::Urn;
use ymir::errors::Outcome;

#[async_trait::async_trait]
pub trait DistributionRepositoryTrait: Send + Sync {
    async fn get_all_distributions(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<distribution::Model>>;
    async fn get_batch_distributions(&self, ids: &Vec<Urn>) -> Outcome<Vec<distribution::Model>>;

    async fn get_distributions_by_dataset_id(
        &self,
        dataset_id: &Urn,
    ) -> Outcome<Vec<distribution::Model>>;
    async fn get_distribution_by_dataset_id_and_dct_format(
        &self,
        dataset_id: &Urn,
        dct_formats: &String,
    ) -> Outcome<distribution::Model>;
    async fn get_distribution_by_id(
        &self,
        distribution_id: &Urn,
    ) -> Outcome<Option<distribution::Model>>;
    async fn put_distribution_by_id(
        &self,
        distribution_id: &Urn,
        edit_distribution_model: &EditDistributionModel,
    ) -> Outcome<distribution::Model>;
    async fn create_distribution(
        &self,
        new_distribution_model: &NewDistributionModel,
    ) -> Outcome<distribution::Model>;
    async fn delete_distribution_by_id(&self, distribution_id: &Urn) -> Outcome<()>;
}
