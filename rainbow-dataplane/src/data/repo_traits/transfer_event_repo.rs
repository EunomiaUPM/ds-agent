use crate::data::entities::transfer_event;
use crate::data::entities::transfer_event::NewTransferEvent;
use thiserror::Error;
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

#[async_trait::async_trait]
pub trait TransferEventRepo: Send + Sync + 'static {
    async fn get_all_transfer_events(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<transfer_event::Model>>;
    async fn get_batch_transfer_events(
        &self,
        ids: &Vec<Urn>,
    ) -> Outcome<Vec<transfer_event::Model>>;
    async fn get_all_transfer_events_by_process_id(
        &self,
        process_id: &Urn,
    ) -> Outcome<Vec<transfer_event::Model>>;
    async fn get_transfer_event_by_id(
        &self,
        transfer_event: &Urn,
    ) -> Outcome<Option<transfer_event::Model>>;
    async fn create_transfer_event(
        &self,
        new_transfer_event: &NewTransferEvent,
    ) -> Outcome<transfer_event::Model>;
}

#[derive(Debug, Error)]
pub enum TransferEventRepoErrors {
    #[error("Transfer event not found")]
    TransferEventNotFound,
    #[error("Error fetching transfer event. {0}")]
    ErrorFetchingTransferEvent(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error creating transfer event. {0}")]
    ErrorCreatingTransferEvent(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error deleting transfer event. {0}")]
    ErrorDeletingTransferEvent(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error updating transfer event. {0}")]
    ErrorUpdatingTransferEvent(Box<dyn std::error::Error + Send + Sync>),
}

impl RepoIntoErrors for TransferEventRepoErrors {}
