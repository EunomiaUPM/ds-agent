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

pub mod datasets;

use crate::data::entities::dataset;
use crate::data::entities::dataset::{EditDatasetModel, Model, NewDatasetModel};
use serde::{Deserialize, Serialize};
use urn::Urn;
use ymir::errors::Outcome;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DatasetDto {
    #[serde(flatten)]
    pub inner: dataset::Model,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NewDatasetDto {
    pub id: Option<Urn>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    pub dct_conforms_to: Option<String>,
    pub dct_creator: Option<String>,
    pub dct_title: Option<String>,
    pub dct_description: Option<String>,
    pub catalog_id: Urn,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct EditDatasetDto {
    pub dct_conforms_to: Option<String>,
    pub dct_creator: Option<String>,
    pub dct_title: Option<String>,
    pub dct_description: Option<String>,
}

use common::auth::AccessScope;

impl NewDatasetDto {
    pub fn into_model(self, tenant_id: String) -> NewDatasetModel {
        NewDatasetModel {
            id: self.id,
            tenant_id,
            dct_conforms_to: self.dct_conforms_to,
            dct_creator: self.dct_creator,
            dct_title: self.dct_title,
            dct_description: self.dct_description,
            catalog_id: self.catalog_id,
        }
    }
}

impl From<EditDatasetDto> for EditDatasetModel {
    fn from(dto: EditDatasetDto) -> Self {
        Self {
            dct_conforms_to: dto.dct_conforms_to,
            dct_creator: dto.dct_creator,
            dct_title: dto.dct_title,
            dct_description: dto.dct_description,
        }
    }
}

impl From<dataset::Model> for DatasetDto {
    fn from(value: Model) -> Self {
        Self { inner: value }
    }
}

use crate::entities::filters::DatasetFilter;
use common::paginated_spec::{Page, Paginated, Sort};

#[mockall::automock]
#[async_trait::async_trait]
pub trait DatasetEntityTrait: Send + Sync {
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
