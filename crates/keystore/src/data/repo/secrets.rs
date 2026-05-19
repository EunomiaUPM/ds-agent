use crate::entities::commands::{EditSecretCommand, NewSecretCommand};
use crate::entities::entry::SecretEntry;
use crate::entities::key::Key;
use crate::entities::version::Version;
use thiserror::Error;
use ymir::errors::{Outcome, RepoIntoErrors};

#[allow(dead_code)]
#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait SecretRepoTrait: Send + Sync {
    async fn get_all_secrets(&self) -> Outcome<Vec<SecretEntry>>;
    async fn count_secrets(&self) -> Outcome<u64>;
    async fn get_batch_secrets(&self, keys: &[Key]) -> Outcome<Vec<SecretEntry>>;
    async fn get_secret_by_key(&self, key: &Key) -> Outcome<Option<SecretEntry>>;
    async fn create_secret(&self, new_model: &NewSecretCommand) -> Outcome<SecretEntry>;
    async fn put_secret(&self, key: &Key, edit_model: &EditSecretCommand) -> Outcome<SecretEntry>;
    async fn delete_secret(&self, key: &Key) -> Outcome<()>;
}

#[derive(Debug, Error)]
pub enum SecretRepoErrors {
    #[error("Secret not found")]
    SecretNotFound,
    #[error("Secret already exists")]
    SecretAlreadyExists,
    #[error("Version conflict: expected {expected:?}, actual {actual:?}")]
    VersionConflict { expected: Version, actual: Version },
    #[error("Error fetching secret. {0}")]
    ErrorFetchingSecret(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error creating secret. {0}")]
    ErrorCreatingSecret(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error deleting secret. {0}")]
    ErrorDeletingSecret(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error updating secret. {0}")]
    ErrorUpdatingSecret(Box<dyn std::error::Error + Send + Sync>),
}

impl RepoIntoErrors for SecretRepoErrors {}
