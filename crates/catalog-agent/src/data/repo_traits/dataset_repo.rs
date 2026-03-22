/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use crate::data::entities::dataset;
use crate::data::entities::dataset::{EditDatasetModel, NewDatasetModel};
use crate::data::repo_traits::catalog_db_errors::CatalogAgentRepoErrors;
use urn::Urn;
use ymir::errors::Outcome;

#[async_trait::async_trait]
pub trait DatasetRepositoryTrait: Send + Sync {
    async fn get_all_datasets(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<dataset::Model>>;
    async fn get_batch_datasets(&self, ids: &Vec<Urn>) -> Outcome<Vec<dataset::Model>>;
    async fn get_datasets_by_catalog_id(&self, catalog_id: &Urn) -> Outcome<Vec<dataset::Model>>;
    async fn get_dataset_by_id(&self, dataset_id: &Urn) -> Outcome<Option<dataset::Model>>;

    async fn put_dataset_by_id(
        &self,
        dataset_id: &Urn,
        edit_dataset_model: &EditDatasetModel,
    ) -> Outcome<dataset::Model>;
    async fn create_dataset(&self, new_dataset_model: &NewDatasetModel) -> Outcome<dataset::Model>;

    async fn delete_dataset_by_id(&self, dataset_id: &Urn) -> Outcome<()>;
}
