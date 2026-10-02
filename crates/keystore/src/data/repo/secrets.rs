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

//! Secret repository port.

use crate::entities::commands::{EditSecretCommand, NewSecretCommand};
use crate::entities::entry::SecretEntry;
use crate::entities::filters::PrefixFilter;
use crate::entities::key::Key;
use crate::entities::version::Version;
use thiserror::Error;
use ymir::errors::{Outcome, RepoIntoErrors};

/// Persistence of secrets, keyed by tenant and path.
#[mockall::automock]
#[async_trait::async_trait]
pub trait SecretRepoTrait: Send + Sync {
    /// Secrets whose key starts with the filter's prefix.
    async fn get_all_secrets(&self, filter: &PrefixFilter) -> Outcome<Vec<SecretEntry>>;
    async fn count_secrets(&self, filter: &PrefixFilter) -> Outcome<u64>;
    /// Secrets of the tenant found among `keys`; missing ones are left out.
    async fn get_batch_secrets(&self, tenant_id: &str, keys: &[Key]) -> Outcome<Vec<SecretEntry>>;
    async fn get_secret_by_key(&self, tenant_id: &str, key: &Key) -> Outcome<Option<SecretEntry>>;
    /// Fails when the key already exists for the tenant.
    async fn create_secret(
        &self,
        tenant_id: &str,
        new_model: &NewSecretCommand,
    ) -> Outcome<SecretEntry>;
    /// Replaces the value if `expected_version` matches, and bumps the version.
    async fn put_secret(
        &self,
        tenant_id: &str,
        key: &Key,
        edit_model: &EditSecretCommand,
    ) -> Outcome<SecretEntry>;
    /// Fails when the key does not exist.
    async fn delete_secret(&self, tenant_id: &str, key: &Key) -> Outcome<()>;
}

/// Failures of the secret repository, mapped onto `Errors`.
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
