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

use common::auth::AccessScope;
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

    pub fn with_event_bus(mut self, event_bus: Option<events::EventBus>) -> Self {
        self.event_bus = event_bus;
        self
    }
}

#[async_trait::async_trait]
impl SecretStore for SecretStoreImpl {
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn create(&self, scope: &AccessScope, cmd: &NewSecretCommand) -> Outcome<SecretEntry> {
        let mut cmd = cmd.clone();
        let target_tenant = scope.resolve_create_tenant(cmd.tenant_id.as_deref())?;
        cmd.tenant_id = Some(target_tenant.clone());
        let entry = self.repo.create_secret(&target_tenant, &cmd).await?;
        events::emit_action!(
            self.event_bus,
            crate::EVENT_PREFIX,
            "secret",
            "create",
            &entry
        );
        Ok(entry)
    }

    #[tracing::instrument(level = "info", skip(self, scope), fields(key = %key), err)]
    async fn read(&self, scope: &AccessScope, key: &Key) -> Outcome<SecretEntry> {
        scope.require_read()?;
        self.repo
            .get_secret_by_key(scope.acting_tenant(), key)
            .await?
            .or_not_found(key, "secret")
    }

    #[tracing::instrument(level = "info", skip(self, scope, cmd), fields(key = %key), err)]
    async fn update(
        &self,
        scope: &AccessScope,
        key: &Key,
        cmd: &EditSecretCommand,
    ) -> Outcome<Version> {
        scope.require_write()?;
        let entry = self
            .repo
            .put_secret(scope.acting_tenant(), key, cmd)
            .await?;
        events::emit_action!(
            self.event_bus,
            crate::EVENT_PREFIX,
            "secret",
            "edit",
            &entry
        );
        Ok(entry.metadata.version)
    }

    #[tracing::instrument(level = "info", skip(self, scope), fields(key = %key), err)]
    async fn delete(&self, scope: &AccessScope, key: &Key) -> Outcome<()> {
        scope.require_write()?;
        self.repo.delete_secret(scope.acting_tenant(), key).await?;
        events::emit_action!(
            self.event_bus,
            crate::EVENT_PREFIX,
            "secret",
            "delete",
            &events::EntityDeletedDto::new(key.as_str())
        );
        Ok(())
    }

    #[tracing::instrument(level = "info", skip(self, scope), err)]
    async fn list(&self, scope: &AccessScope, filter: &PrefixFilter) -> Outcome<Vec<SecretEntry>> {
        scope.require_read()?;
        filter.validate()?;
        let mut filter = filter.clone();
        filter.tenant_id = scope.resolve_query_tenant(filter.tenant_id.as_deref())?;
        self.repo.get_all_secrets(&filter).await
    }

    #[tracing::instrument(level = "info", skip_all, err)]
    async fn batch(&self, scope: &AccessScope, keys: &[Key]) -> Outcome<Vec<SecretEntry>> {
        scope.require_read()?;
        if keys.is_empty() {
            return Ok(vec![]);
        }
        self.repo
            .get_batch_secrets(scope.acting_tenant(), keys)
            .await
    }

    #[tracing::instrument(level = "info", skip(self, scope, value), fields(key = %key), err)]
    async fn upsert(&self, scope: &AccessScope, key: &Key, value: SecretValue) -> Outcome<()> {
        scope.require_write()?;
        match self
            .repo
            .get_secret_by_key(scope.acting_tenant(), key)
            .await?
        {
            None => {
                let cmd = NewSecretCommand {
                    key: key.clone(),
                    value,
                    description: None,
                    tenant_id: Some(scope.acting_tenant().to_string()),
                };
                self.create(scope, &cmd).await?;
            }
            Some(existing) => {
                self.repo
                    .put_secret(
                        scope.acting_tenant(),
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
