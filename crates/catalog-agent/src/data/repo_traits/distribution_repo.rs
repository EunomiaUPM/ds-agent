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

use crate::entities::filters::DistributionFilter;
use common::paginated_spec::{Page, Sort};

#[mockall::automock]
#[async_trait::async_trait]
pub trait DistributionRepositoryTrait: Send + Sync {
    async fn get_all_distributions(
        &self,
        filters: &DistributionFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<(Vec<distribution::Model>, Option<u64>)>;
    async fn get_batch_distributions(
        &self,
        tenant_id: &str,
        ids: &[Urn],
    ) -> Outcome<Vec<distribution::Model>>;

    async fn get_distributions_by_dataset_id(
        &self,
        tenant_id: &str,
        dataset_id: &Urn,
    ) -> Outcome<Vec<distribution::Model>>;
    async fn get_distribution_by_dataset_id_and_dct_format(
        &self,
        tenant_id: &str,
        dataset_id: &Urn,
        dct_formats: &str,
    ) -> Outcome<Option<distribution::Model>>;
    async fn get_distribution_by_id(
        &self,
        tenant_id: &str,
        distribution_id: &Urn,
    ) -> Outcome<Option<distribution::Model>>;
    async fn put_distribution_by_id(
        &self,
        tenant_id: &str,
        distribution_id: &Urn,
        edit_distribution_model: &EditDistributionModel,
    ) -> Outcome<distribution::Model>;
    async fn create_distribution(
        &self,
        new_distribution_model: &NewDistributionModel,
    ) -> Outcome<distribution::Model>;
    /// Deletes and returns the removed row so callers can evict derived caches.
    async fn delete_distribution_by_id(
        &self,
        tenant_id: &str,
        distribution_id: &Urn,
    ) -> Outcome<distribution::Model>;
}
