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
    async fn get_batch_datasets(
        &self,
        ids: &Vec<Urn>,
    ) -> Outcome<Vec<dataset::Model>>;
    async fn get_datasets_by_catalog_id(
        &self,
        catalog_id: &Urn,
    ) -> Outcome<Vec<dataset::Model>>;
    async fn get_dataset_by_id(
        &self,
        dataset_id: &Urn,
    ) -> Outcome<Option<dataset::Model>>;

    async fn put_dataset_by_id(
        &self,
        dataset_id: &Urn,
        edit_dataset_model: &EditDatasetModel,
    ) -> Outcome<dataset::Model>;
    async fn create_dataset(
        &self,
        new_dataset_model: &NewDatasetModel,
    ) -> Outcome<dataset::Model>;

    async fn delete_dataset_by_id(
        &self,
        dataset_id: &Urn,
    ) -> Outcome<()>;
}
