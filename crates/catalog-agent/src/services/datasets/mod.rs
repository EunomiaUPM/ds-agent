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
use common::oauth::UserInfo;
use common::paginated_spec::{Page, Paginated, Sort};
use urn::Urn;
use ymir::errors::Outcome;

/// Management of datasets.
#[mockall::automock]
#[async_trait::async_trait]
pub trait DatasetServiceTrait: Send + Sync {
    /// Page of datasets visible to the caller.
    async fn get_all_datasets(
        &self,
        user: &UserInfo,
        filters: &DatasetFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<DatasetDto>>;
    /// Datasets found among `ids`.
    async fn get_batch_datasets(
        &self,
        user: &UserInfo,
        ids: &[Urn],
    ) -> Outcome<Vec<DatasetDto>>;
    /// Datasets of the catalog.
    async fn get_datasets_by_catalog_id(
        &self,
        user: &UserInfo,
        catalog_id: &Urn,
    ) -> Outcome<Vec<DatasetDto>>;
    /// 404 when the dataset is not visible to the caller.
    async fn get_dataset_by_id(&self, user: &UserInfo, dataset_id: &Urn)
        -> Outcome<DatasetDto>;

    async fn put_dataset_by_id(
        &self,
        user: &UserInfo,
        dataset_id: &Urn,
        edit_dataset_model: &EditDatasetDto,
    ) -> Outcome<DatasetDto>;
    async fn create_dataset(
        &self,
        user: &UserInfo,
        new_dataset_model: &NewDatasetDto,
    ) -> Outcome<DatasetDto>;

    async fn delete_dataset_by_id(&self, user: &UserInfo, dataset_id: &Urn) -> Outcome<()>;
}
