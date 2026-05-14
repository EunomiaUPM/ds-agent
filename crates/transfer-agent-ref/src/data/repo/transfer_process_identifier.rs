use crate::entities::transfer_process_identifier::TransferProcessIdentifier;
use thiserror::Error;
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

#[mockall::automock]
#[async_trait::async_trait]
pub trait TransferIdentifierRepoTrait: Send + Sync {
    async fn get_identifiers_by_process_id(
        &self,
        process_id: &Urn,
    ) -> Outcome<Vec<TransferProcessIdentifier>>;

    async fn get_identifier_by_key(
        &self,
        process_id: &Urn,
        key: &str,
    ) -> Outcome<Option<TransferProcessIdentifier>>;

    async fn upsert_identifier(
        &self,
        process_id: &Urn,
        identifier: &TransferProcessIdentifier,
    ) -> Outcome<TransferProcessIdentifier>;

    async fn delete_identifier(&self, process_id: &Urn, key: &str) -> Outcome<()>;
}

#[derive(Debug, Error)]
pub enum TransferIdentifierRepoErrors {
    #[error("Transfer Identifier not found")]
    TransferIdentifierNotFound,
    #[error("Error fetching transfer identifier. {0}")]
    ErrorFetchingTransferIdentifier(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error upserting transfer identifier. {0}")]
    ErrorUpsertingTransferIdentifier(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error deleting transfer identifier. {0}")]
    ErrorDeletingTransferIdentifier(Box<dyn std::error::Error + Send + Sync>),
}

impl RepoIntoErrors for TransferIdentifierRepoErrors {}