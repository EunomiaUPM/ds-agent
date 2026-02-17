use crate::data::entities::dataplane_field;
use crate::data::entities::dataplane_field::{EditDataPlaneFieldModel, NewDataPlaneFieldModel};
use anyhow::Error;
use thiserror::Error;
use urn::Urn;

#[async_trait::async_trait]
pub trait DataplaneFieldRepoTrait: Send + Sync + 'static {
    async fn get_all_dataplane_fields(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> anyhow::Result<Vec<dataplane_field::Model>, DataplaneFieldRepoErrors>;
    async fn get_batch_dataplane_fields(
        &self,
        ids: &Vec<Urn>,
    ) -> anyhow::Result<Vec<dataplane_field::Model>, DataplaneFieldRepoErrors>;
    async fn get_all_dataplane_fields_by_process_id(
        &self,
        process_id: &Urn,
    ) -> anyhow::Result<Vec<dataplane_field::Model>, DataplaneFieldRepoErrors>;
    async fn get_dataplane_field_by_id(
        &self,
        field_id: &Urn,
    ) -> anyhow::Result<Option<dataplane_field::Model>, DataplaneFieldRepoErrors>;
    async fn create_dataplane_field(
        &self,
        process_id: &Urn,
        new_dataplane_field: &NewDataPlaneFieldModel,
    ) -> anyhow::Result<dataplane_field::Model, DataplaneFieldRepoErrors>;
    async fn put_dataplane_field(
        &self,
        field_id: &Urn,
        edit_field: &EditDataPlaneFieldModel,
    ) -> anyhow::Result<dataplane_field::Model, DataplaneFieldRepoErrors>;
    async fn delete_dataplane_field(
        &self,
        field_id: &Urn,
    ) -> anyhow::Result<(), DataplaneFieldRepoErrors>;
    async fn delete_all_dataplane_fields_by_process_id(
        &self,
        process_id: &Urn,
    ) -> anyhow::Result<(), DataplaneFieldRepoErrors>;
}

#[derive(Debug, Error)]
pub enum DataplaneFieldRepoErrors {
    #[error("Dataplane field not found")]
    DataplaneFieldNotFound,
    #[error("Error fetching dataplane field. {0}")]
    ErrorFetchingDataplaneField(Error),
    #[error("Error creating dataplane field. {0}")]
    ErrorCreatingDataplaneField(Error),
    #[error("Error deleting dataplane field. {0}")]
    ErrorDeletingDataplaneField(Error),
    #[error("Error updating dataplane field. {0}")]
    ErrorUpdatingDataplaneField(Error),
}
