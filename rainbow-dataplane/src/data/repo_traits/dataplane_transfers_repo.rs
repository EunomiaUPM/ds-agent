use crate::data::entities::dataplane_transfers;
use crate::data::entities::dataplane_transfers::{
    EditDataplaneTransferModel, NewDataplaneTransferModel,
};
use thiserror::Error;
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

#[async_trait::async_trait]
pub trait DataplaneTransfersRepo: Send + Sync + 'static {
    async fn get_all_dataplane_transfers(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<dataplane_transfers::Model>>;
    async fn get_batch_dataplane_transfers(
        &self,
        ids: &Vec<Urn>,
    ) -> Outcome<Vec<dataplane_transfers::Model>>;
    async fn get_dataplane_transfers_by_id(
        &self,
        process_id: &Urn,
    ) -> Outcome<Option<dataplane_transfers::Model>>;
    async fn get_by_transfer_process_id(
        &self,
        transfer_process_id: &Urn,
    ) -> Outcome<Option<dataplane_transfers::Model>>;
    async fn create_dataplane_transfers(
        &self,
        new_dataplane_transfer: &NewDataplaneTransferModel,
    ) -> Outcome<dataplane_transfers::Model>;
    async fn put_dataplane_transfers(
        &self,
        process_id: &Urn,
        new_dataplane_transfer: &EditDataplaneTransferModel,
    ) -> Outcome<dataplane_transfers::Model>;
    async fn delete_dataplane_transfers(
        &self,
        process_id: &Urn,
    ) -> Outcome<()>;
}

#[derive(Debug, Error)]
pub enum DataplaneTransfersRepoErrors {
    #[error("Dataplane transfer not found")]
    DataplaneTransferNotFound,
    #[error("Error fetching dataplane transfer. {0}")]
    ErrorFetchingDataplaneTransfer(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error creating dataplane transfer. {0}")]
    ErrorCreatingDataplaneTransfer(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error deleting dataplane transfer. {0}")]
    ErrorDeletingDataplaneTransfer(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error updating dataplane transfer. {0}")]
    ErrorUpdatingDataplaneTransfer(Box<dyn std::error::Error + Send + Sync>),
}

impl RepoIntoErrors for DataplaneTransfersRepoErrors {}
