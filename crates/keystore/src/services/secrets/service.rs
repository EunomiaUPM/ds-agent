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

//! Store implementation over the repository.

use std::sync::Arc;

use common::oauth::{Owner, UserInfo};
use common::errors::NotFoundExt;
use common::query::QueryFilter;
use ymir::errors::Outcome;

use crate::data::repo::secrets::SecretRepoTrait;
use crate::entities::commands::{EditSecretCommand, NewSecretCommand};
use crate::entities::entry::SecretEntry;
use crate::entities::filters::PrefixFilter;
use crate::entities::key::Key;
use crate::entities::secret_value::SecretValue;
use crate::entities::version::Version;
use crate::services::secrets::SecretStore;

/// Secret store over a repository, emitting `keystore:` events when a bus is set.
pub struct SecretStoreImpl {
    repo: Arc<dyn SecretRepoTrait>,
    event_bus: Option<events::EventBus>,
}

impl SecretStoreImpl {
    pub fn new(repo: Arc<dyn SecretRepoTrait>) -> Self {
        Self {
            repo,
            event_bus: None,
        }
    }

    /// Publishes create, update and delete events on `event_bus`.
    pub fn with_event_bus(mut self, event_bus: Option<events::EventBus>) -> Self {
        self.event_bus = event_bus;
        self
    }
}

#[async_trait::async_trait]
impl SecretStore for SecretStoreImpl {
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn create(&self, user: &UserInfo, cmd: &NewSecretCommand) -> Outcome<SecretEntry> {
        let entry = self.repo.create_secret(user.id(), cmd).await?;
        events::emit_action!(
            self.event_bus,
            &Owner::private(user),
            crate::EVENT_PREFIX,
            "secret",
            "create",
            &entry
        );
        Ok(entry)
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(user = %user.id(), key = %key)
    )]
    async fn read(&self, user: &UserInfo, key: &Key) -> Outcome<SecretEntry> {
        self.repo
            .get_secret_by_key(user.id(), key)
            .await?
            .or_not_found(key, "secret")
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(user = %user.id(), key = %key)
    )]
    async fn update(
        &self,
        user: &UserInfo,
        key: &Key,
        cmd: &EditSecretCommand,
    ) -> Outcome<Version> {
        let entry = self
            .repo
            .put_secret(user.id(), key, cmd)
            .await?;
        events::emit_action!(
            self.event_bus,
            &Owner::private(user),
            crate::EVENT_PREFIX,
            "secret",
            "edit",
            &entry
        );
        Ok(entry.metadata.version)
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(user = %user.id(), key = %key)
    )]
    async fn delete(&self, user: &UserInfo, key: &Key) -> Outcome<()> {
        self.repo.delete_secret(user.id(), key).await?;
        events::emit_action!(
            self.event_bus,
            &Owner::private(user),
            crate::EVENT_PREFIX,
            "secret",
            "delete",
            &events::EntityDeletedDto::new(key.as_str())
        );
        Ok(())
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn list(&self, user: &UserInfo, filter: &PrefixFilter) -> Outcome<Vec<SecretEntry>> {
        filter.validate()?;
        let mut filter = filter.clone();
        filter.user_id = crate::services::owner_for_list(user, filter.user_id.as_deref())?;
        self.repo.get_all_secrets(&filter).await
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn batch(&self, user: &UserInfo, keys: &[Key]) -> Outcome<Vec<SecretEntry>> {
        if keys.is_empty() {
            return Ok(vec![]);
        }
        self.repo
            .get_batch_secrets(user.id(), keys)
            .await
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(user = %user.id(), key = %key)
    )]
    async fn upsert(&self, user: &UserInfo, key: &Key, value: SecretValue) -> Outcome<()> {
        match self
            .repo
            .get_secret_by_key(user.id(), key)
            .await?
        {
            None => {
                let cmd = NewSecretCommand {
                    key: key.clone(),
                    value,
                    description: None,
                };
                self.create(user, &cmd).await?;
            }
            Some(existing) => {
                self.repo
                    .put_secret(
                        user.id(),
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
