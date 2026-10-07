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

//! Distribution repository.

use crate::data::entities::distribution;
use common::oauth::OwnerScope;
use crate::data::entities::distribution::{EditDistributionModel, NewDistributionModel};
use urn::Urn;
use ymir::errors::Outcome;

use crate::entities::filters::DistributionFilter;
use common::paginated_spec::{Page, Sort};

/// Persistence of distributions, within the owner scope each call gives.
#[mockall::automock]
#[async_trait::async_trait]
pub trait DistributionRepositoryTrait: Send + Sync {
    /// Page of distributions matching the filters, with the total.
    async fn get_all_distributions(
        &self,
        scope: &OwnerScope,
        filters: &DistributionFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<(Vec<distribution::Model>, Option<u64>)>;
    async fn get_batch_distributions(
        &self,
        scope: &OwnerScope,
        ids: &[Urn],
    ) -> Outcome<Vec<distribution::Model>>;

    /// Distributions of the dataset.
    async fn get_distributions_by_dataset_id(
        &self,
        scope: &OwnerScope,
        dataset_id: &Urn,
    ) -> Outcome<Vec<distribution::Model>>;
    /// Distribution of the dataset in the given `dct:format`.
    async fn get_distribution_by_dataset_id_and_dct_format(
        &self,
        scope: &OwnerScope,
        dataset_id: &Urn,
        dct_formats: &str,
    ) -> Outcome<Option<distribution::Model>>;
    async fn get_distribution_by_id(
        &self,
        scope: &OwnerScope,
        distribution_id: &Urn,
    ) -> Outcome<Option<distribution::Model>>;
    async fn put_distribution_by_id(
        &self,
        scope: &OwnerScope,
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
        scope: &OwnerScope,
        distribution_id: &Urn,
    ) -> Outcome<distribution::Model>;
}
