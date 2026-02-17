use crate::data::entities::dataplane_transfers;
use crate::data::entities::dataplane_transfers::{
    EditDataplaneTransferModel, NewDataplaneTransferModel,
};
use anyhow::Error;
use thiserror::Error;
use urn::Urn;

#[async_trait::async_trait]
pub trait DataplaneTransfersRepo: Send + Sync + 'static {
    async fn get_all_dataplane_transfers(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> anyhow::Result<Vec<dataplane_transfers::Model>, DataplaneTransfersRepoErrors>;
    async fn get_batch_dataplane_transfers(
        &self,
        ids: &Vec<Urn>,
    ) -> anyhow::Result<Vec<dataplane_transfers::Model>, DataplaneTransfersRepoErrors>;
    async fn get_dataplane_transfers_by_id(
        &self,
        process_id: &Urn,
    ) -> anyhow::Result<Option<dataplane_transfers::Model>, DataplaneTransfersRepoErrors>;
    async fn get_by_transfer_process_id(
        &self,
        transfer_process_id: &str,
    ) -> anyhow::Result<Option<dataplane_transfers::Model>, DataplaneTransfersRepoErrors>;
    async fn create_dataplane_transfers(
        &self,
        new_dataplane_transfer: &NewDataplaneTransferModel,
    ) -> anyhow::Result<dataplane_transfers::Model, DataplaneTransfersRepoErrors>;
    async fn put_dataplane_transfers(
        &self,
        process_id: &Urn,
        new_dataplane_transfer: &EditDataplaneTransferModel,
    ) -> anyhow::Result<dataplane_transfers::Model, DataplaneTransfersRepoErrors>;
    async fn delete_dataplane_transfers(
        &self,
        process_id: &Urn,
    ) -> anyhow::Result<(), DataplaneTransfersRepoErrors>;
}

#[derive(Debug, Error)]
pub enum DataplaneTransfersRepoErrors {
    #[error("Dataplane transfer not found")]
    DataplaneTransferNotFound,
    #[error("Error fetching dataplane transfer. {0}")]
    ErrorFetchingDataplaneTransfer(Error),
    #[error("Error creating dataplane transfer. {0}")]
    ErrorCreatingDataplaneTransfer(Error),
    #[error("Error deleting dataplane transfer. {0}")]
    ErrorDeletingDataplaneTransfer(Error),
    #[error("Error updating dataplane transfer. {0}")]
    ErrorUpdatingDataplaneTransfer(Error),
}
