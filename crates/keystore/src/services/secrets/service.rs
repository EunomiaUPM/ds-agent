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

use std::sync::Arc;

use ymir::errors::{Outcome, RepoIntoErrors};

use crate::data::repo::secrets::{SecretRepoErrors, SecretRepoTrait};
use crate::entities::commands::{EditSecretCommand, NewSecretCommand};
use crate::entities::entry::SecretEntry;
use crate::entities::key::{Key, KeyPrefix};
use crate::entities::secret_value::SecretValue;
use crate::entities::version::Version;
use crate::services::secrets::SecretStore;

pub struct SecretStoreImpl {
    repo: Arc<dyn SecretRepoTrait>,
}

impl SecretStoreImpl {
    pub fn new(repo: Arc<dyn SecretRepoTrait>) -> Self {
        Self { repo }
    }
}

#[async_trait::async_trait]
impl SecretStore for SecretStoreImpl {
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn create(&self, cmd: &NewSecretCommand) -> Outcome<SecretEntry> {
        self.repo.create_secret(cmd).await
    }

    #[tracing::instrument(level = "info", skip(self), fields(key = %key), err)]
    async fn read(&self, key: &Key) -> Outcome<SecretEntry> {
        self.repo
            .get_secret_by_key(key)
            .await?
            .ok_or_else(|| SecretRepoErrors::SecretNotFound.into_errors())
    }

    #[tracing::instrument(level = "info", skip(self, cmd), fields(key = %key), err)]
    async fn update(&self, key: &Key, cmd: &EditSecretCommand) -> Outcome<Version> {
        let entry = self.repo.put_secret(key, cmd).await?;
        Ok(entry.metadata.version)
    }

    #[tracing::instrument(level = "info", skip(self), fields(key = %key), err)]
    async fn delete(&self, key: &Key) -> Outcome<()> {
        self.repo.delete_secret(key).await
    }

    #[tracing::instrument(level = "info", skip(self), err)]
    async fn list(&self, prefix: &KeyPrefix) -> Outcome<Vec<SecretEntry>> {
        let all = self.repo.get_all_secrets().await?;
        let prefix_str = prefix.as_str();
        Ok(all
            .into_iter()
            .filter(|e| prefix_str.is_empty() || e.metadata.key.as_str().starts_with(prefix_str))
            .collect())
    }

    #[tracing::instrument(level = "info", skip(self, value), fields(key = %key), err)]
    async fn upsert(&self, key: &Key, value: SecretValue) -> Outcome<()> {
        match self.repo.get_secret_by_key(key).await? {
            None => {
                self.repo
                    .create_secret(&NewSecretCommand {
                        key: key.clone(),
                        value,
                        description: None,
                    })
                    .await?;
            }
            Some(existing) => {
                self.repo
                    .put_secret(
                        key,
                        &EditSecretCommand {
                            value,
                            expected_version: existing.metadata.version,
                            description: None,
                        },
                    )
                    .await?;
            }
        }
        Ok(())
    }
}
