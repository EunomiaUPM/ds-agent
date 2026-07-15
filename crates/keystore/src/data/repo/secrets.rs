/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

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
    async fn list_secrets_by_prefix(&self, prefix: &str) -> Outcome<Vec<SecretEntry>>;
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
