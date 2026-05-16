use crate::entities::commands::{EditTransferProcessCommand, NewTransferProcessCommand};
use crate::entities::query::{Page, Sort, TransferProcessFilter};
use crate::entities::transfer_process::TransferProcess;
use thiserror::Error;
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

#[mockall::automock]
#[async_trait::async_trait]
pub trait TransferProcessRepoTrait: Send + Sync {
    async fn get_all_transfer_processes(
        &self,
        filters: &TransferProcessFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Vec<TransferProcess>>;
    async fn get_batch_transfer_processes(&self, ids: &Vec<Urn>) -> Outcome<Vec<TransferProcess>>;
    async fn get_transfer_process_by_id(&self, id: &Urn) -> Outcome<Option<TransferProcess>>;
    async fn get_transfer_process_by_key_id(
        &self,
        key_id: &str,
        id: &Urn,
    ) -> Outcome<Option<TransferProcess>>;
    async fn get_transfer_process_by_key_value(&self, id: &Urn)
    -> Outcome<Option<TransferProcess>>;
    async fn create_transfer_process(
        &self,
        new_model: &NewTransferProcessCommand,
    ) -> Outcome<TransferProcess>;
    async fn put_transfer_process(
        &self,
        id: &Urn,
        edit_model: &EditTransferProcessCommand,
    ) -> Outcome<TransferProcess>;
    async fn delete_transfer_process(&self, id: &Urn) -> Outcome<()>;
}

#[derive(Debug, Error)]
pub enum TransferProcessRepoErrors {
    #[error("Transfer Process not found")]
    TransferProcessNotFound,
    #[error("Error fetching transfer process. {0}")]
    ErrorFetchingTransferProcess(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error creating transfer process. {0}")]
    ErrorCreatingTransferProcess(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error deleting transfer process. {0}")]
    ErrorDeletingTransferProcess(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error updating transfer process. {0}")]
    ErrorUpdatingTransferProcess(Box<dyn std::error::Error + Send + Sync>),
}

impl RepoIntoErrors for TransferProcessRepoErrors {}
