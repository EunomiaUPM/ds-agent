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

//! Distribution management use cases.

pub mod service;

use crate::entities::distributions::{DistributionDto, EditDistributionDto, NewDistributionDto};
use crate::entities::filters::DistributionFilter;
use common::auth::AccessScope;
use common::paginated_spec::{Page, Paginated, Sort};
use urn::Urn;
use ymir::errors::Outcome;

/// Management of distributions.
#[mockall::automock]
#[async_trait::async_trait]
pub trait DistributionServiceTrait: Send + Sync {
    /// Page of distributions visible to the caller.
    async fn get_all_distributions(
        &self,
        scope: &AccessScope,
        filters: &DistributionFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<DistributionDto>>;
    /// Distributions found among `ids`.
    async fn get_batch_distributions(
        &self,
        scope: &AccessScope,
        ids: &[Urn],
    ) -> Outcome<Vec<DistributionDto>>;

    /// Distributions of the dataset.
    async fn get_distributions_by_dataset_id(
        &self,
        scope: &AccessScope,
        dataset_id: &Urn,
    ) -> Outcome<Vec<DistributionDto>>;
    /// Distribution of the dataset in the given `dct:format`.
    async fn get_distribution_by_dataset_id_and_dct_format(
        &self,
        scope: &AccessScope,
        dataset_id: &Urn,
        dct_formats: &str,
    ) -> Outcome<DistributionDto>;
    /// 404 when the distribution is not visible to the caller.
    async fn get_distribution_by_id(
        &self,
        scope: &AccessScope,
        distribution_id: &Urn,
    ) -> Outcome<DistributionDto>;
    async fn put_distribution_by_id(
        &self,
        scope: &AccessScope,
        distribution_id: &Urn,
        edit_distribution_model: &EditDistributionDto,
    ) -> Outcome<DistributionDto>;
    async fn create_distribution(
        &self,
        scope: &AccessScope,
        new_distribution_model: &NewDistributionDto,
    ) -> Outcome<DistributionDto>;
    async fn delete_distribution_by_id(
        &self,
        scope: &AccessScope,
        distribution_id: &Urn,
    ) -> Outcome<()>;
}
