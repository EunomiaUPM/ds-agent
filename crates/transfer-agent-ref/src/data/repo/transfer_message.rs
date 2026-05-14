use crate::entities::commands::NewTransferMessageCommand;
use crate::entities::query::{Page, Sort, TransferMessageFilter};
use crate::entities::transfer_message::TransferMessage;
use thiserror::Error;
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

#[mockall::automock]
#[async_trait::async_trait]
pub trait TransferMessageRepoTrait: Send + Sync {
    async fn get_all_transfer_messages(
        &self,
        filters: &TransferMessageFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Vec<TransferMessage>>;

    async fn get_messages_by_process_id(
        &self,
        process_id: &Urn,
        filters: &TransferMessageFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Vec<TransferMessage>>;

    async fn get_transfer_message_by_id(&self, id: &Urn) -> Outcome<Option<TransferMessage>>;

    async fn create_transfer_message(
        &self,
        cmd: &NewTransferMessageCommand,
    ) -> Outcome<TransferMessage>;

    async fn delete_transfer_message(&self, id: &Urn) -> Outcome<()>;
}

#[derive(Debug, Error)]
pub enum TransferMessageRepoErrors {
    #[error("Transfer Message not found")]
    TransferMessageNotFound,
    #[error("Error fetching transfer message. {0}")]
    ErrorFetchingTransferMessage(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error creating transfer message. {0}")]
    ErrorCreatingTransferMessage(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error deleting transfer message. {0}")]
    ErrorDeletingTransferMessage(Box<dyn std::error::Error + Send + Sync>),
}

impl RepoIntoErrors for TransferMessageRepoErrors {}