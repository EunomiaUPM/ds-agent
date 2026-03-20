pub(crate) mod datasets;

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

impl From<NewDatasetDto> for NewDatasetModel {
    fn from(dto: NewDatasetDto) -> Self {
        Self {
            id: dto.id,
            dct_conforms_to: dto.dct_conforms_to,
            dct_creator: dto.dct_creator,
            dct_title: dto.dct_title,
            dct_description: dto.dct_description,
            catalog_id: dto.catalog_id,
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

#[mockall::automock]
#[async_trait::async_trait]
pub trait DatasetEntityTrait: Send + Sync {
    async fn get_all_datasets(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<DatasetDto>>;
    async fn get_batch_datasets(&self, ids: &Vec<Urn>) -> Outcome<Vec<DatasetDto>>;
    async fn get_datasets_by_catalog_id(&self, catalog_id: &Urn)
        -> Outcome<Vec<DatasetDto>>;
    async fn get_dataset_by_id(&self, dataset_id: &Urn) -> Outcome<Option<DatasetDto>>;

    async fn put_dataset_by_id(
        &self,
        dataset_id: &Urn,
        edit_dataset_model: &EditDatasetDto,
    ) -> Outcome<DatasetDto>;
    async fn create_dataset(&self, new_dataset_model: &NewDatasetDto)
        -> Outcome<DatasetDto>;

    async fn delete_dataset_by_id(&self, dataset_id: &Urn) -> Outcome<()>;
}
