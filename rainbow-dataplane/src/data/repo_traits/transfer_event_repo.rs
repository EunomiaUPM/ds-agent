use crate::data::entities::transfer_event;
use crate::data::entities::transfer_event::NewTransferEvent;
use anyhow::Error;
use thiserror::Error;
use urn::Urn;
use ymir::errors::RepoIntoErrors;

#[async_trait::async_trait]
pub trait TransferEventRepo: Send + Sync + 'static {
    async fn get_all_transfer_events(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> anyhow::Result<Vec<transfer_event::Model>, TransferEventRepoErrors>;
    async fn get_batch_transfer_events(
        &self,
        ids: &Vec<Urn>,
    ) -> anyhow::Result<Vec<transfer_event::Model>, TransferEventRepoErrors>;
    async fn get_all_transfer_events_by_process_id(
        &self,
        process_id: &Urn,
    ) -> anyhow::Result<Vec<transfer_event::Model>, TransferEventRepoErrors>;
    async fn get_transfer_event_by_id(
        &self,
        transfer_event: &Urn,
    ) -> anyhow::Result<Option<transfer_event::Model>, TransferEventRepoErrors>;
    async fn create_transfer_event(
        &self,
        new_transfer_event: &NewTransferEvent,
    ) -> anyhow::Result<transfer_event::Model, TransferEventRepoErrors>;
}

#[derive(Debug, Error)]
pub enum TransferEventRepoErrors {
    #[error("Transfer event not found")]
    TransferEventNotFound,
    #[error("Error fetching transfer event. {0}")]
    ErrorFetchingTransferEvent(Error),
    #[error("Error creating transfer event. {0}")]
    ErrorCreatingTransferEvent(Error),
    #[error("Error deleting transfer event. {0}")]
    ErrorDeletingTransferEvent(Error),
    #[error("Error updating transfer event. {0}")]
    ErrorUpdatingTransferEvent(Error),
}

impl RepoIntoErrors for TransferEventRepoErrors {}
