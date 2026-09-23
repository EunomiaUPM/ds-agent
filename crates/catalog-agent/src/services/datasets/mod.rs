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

//! Dataset management use cases.

pub mod service;

use crate::entities::datasets::{DatasetDto, EditDatasetDto, NewDatasetDto};
use crate::entities::filters::DatasetFilter;
use common::auth::AccessScope;
use common::paginated_spec::{Page, Paginated, Sort};
use urn::Urn;
use ymir::errors::Outcome;

#[mockall::automock]
#[async_trait::async_trait]
pub trait DatasetServiceTrait: Send + Sync {
    async fn get_all_datasets(
        &self,
        scope: &AccessScope,
        filters: &DatasetFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<DatasetDto>>;
    async fn get_batch_datasets(
        &self,
        scope: &AccessScope,
        ids: &[Urn],
    ) -> Outcome<Vec<DatasetDto>>;
    async fn get_datasets_by_catalog_id(
        &self,
        scope: &AccessScope,
        catalog_id: &Urn,
    ) -> Outcome<Vec<DatasetDto>>;
    async fn get_dataset_by_id(&self, scope: &AccessScope, dataset_id: &Urn)
        -> Outcome<DatasetDto>;

    async fn put_dataset_by_id(
        &self,
        scope: &AccessScope,
        dataset_id: &Urn,
        edit_dataset_model: &EditDatasetDto,
    ) -> Outcome<DatasetDto>;
    async fn create_dataset(
        &self,
        scope: &AccessScope,
        new_dataset_model: &NewDatasetDto,
    ) -> Outcome<DatasetDto>;

    async fn delete_dataset_by_id(&self, scope: &AccessScope, dataset_id: &Urn) -> Outcome<()>;
}
