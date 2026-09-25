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

use std::collections::HashMap;
use std::sync::Arc;

use futures_util::future::try_join_all;
use serde_json::Value;
use ymir::errors::Outcome;
use ymir::services::vault::VaultService;
use ymir::services::vault::VaultTrait;

use crate::data::repo::secrets::SecretRepoTrait;
use crate::entities::commands::{EditSecretCommand, NewSecretCommand};
use crate::entities::entry::{Entry, SecretEntry};
use crate::entities::filters::PrefixFilter;
use crate::entities::key::Key;
use crate::entities::secret_value::SecretValue;

pub struct VaultSecretRepo {
    vault_service: Arc<VaultService>,
    repo: Arc<dyn SecretRepoTrait>,
}

impl VaultSecretRepo {
    pub fn new(vault_service: Arc<VaultService>, repo: Arc<dyn SecretRepoTrait>) -> Self {
        assert!(
            matches!(*vault_service, VaultService::Real(_)),
            "VaultService must be Real for VaultSecretRepo"
        );
        Self {
            vault_service,
            repo,
        }
    }

    async fn vault_read(&self, tenant_id: &str, key: &Key) -> Outcome<SecretValue> {
        let vault_path = format!("{}/{}", tenant_id, key.as_str().trim_start_matches('/'));
        let map: HashMap<String, serde_json::Value> =
            self.vault_service.read(None, &vault_path).await?;
        let inner = map.get("value").cloned().unwrap_or(serde_json::Value::Null);
        Ok(SecretValue::new(inner))
    }

    async fn vault_write(&self, tenant_id: &str, key: &Key, value: &SecretValue) -> Outcome<()> {
        let vault_path = format!("{}/{}", tenant_id, key.as_str().trim_start_matches('/'));
        let mut map = HashMap::new();
        map.insert("value".to_string(), value.expose().clone());
        self.vault_service.write(None, &vault_path, &map).await
    }

    async fn hydrate(&self, entries: Vec<SecretEntry>) -> Outcome<Vec<SecretEntry>> {
        let futures = entries.into_iter().map(|entry| async move {
            let value = self
                .vault_read(&entry.metadata.tenant_id, &entry.metadata.key)
                .await?;
            Ok::<SecretEntry, ymir::errors::Errors>(Entry {
                metadata: entry.metadata,
                value,
            })
        });
        try_join_all(futures).await
    }
}

#[async_trait::async_trait]
impl SecretRepoTrait for VaultSecretRepo {
    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_all_secrets(&self, filter: &PrefixFilter) -> Outcome<Vec<SecretEntry>> {
        let entries = self.repo.get_all_secrets(filter).await?;
        self.hydrate(entries).await
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn count_secrets(&self, filter: &PrefixFilter) -> Outcome<u64> {
        self.repo.count_secrets(filter).await
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_batch_secrets(&self, tenant_id: &str, keys: &[Key]) -> Outcome<Vec<SecretEntry>> {
        let entries = self.repo.get_batch_secrets(tenant_id, keys).await?;
        self.hydrate(entries).await
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_secret_by_key(&self, tenant_id: &str, key: &Key) -> Outcome<Option<SecretEntry>> {
        let Some(entry) = self.repo.get_secret_by_key(tenant_id, key).await? else {
            return Ok(None);
        };
        let value = self.vault_read(tenant_id, &entry.metadata.key).await?;
        Ok(Some(Entry {
            metadata: entry.metadata,
            value,
        }))
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn create_secret(
        &self,
        tenant_id: &str,
        new_model: &NewSecretCommand,
    ) -> Outcome<SecretEntry> {
        self.vault_write(tenant_id, &new_model.key, &new_model.value)
            .await?;
        let mut entry_model = new_model.clone();
        entry_model.value = SecretValue::new(Value::String("vault".to_string()));
        self.repo.create_secret(tenant_id, &entry_model).await
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn put_secret(
        &self,
        tenant_id: &str,
        key: &Key,
        edit_model: &EditSecretCommand,
    ) -> Outcome<SecretEntry> {
        // DB first: validates version conflict before touching vault
        let mut entry_model = edit_model.clone();
        entry_model.value = SecretValue::new(Value::String("vault".to_string()));
        let entry = self.repo.put_secret(tenant_id, key, &entry_model).await?;
        self.vault_write(tenant_id, key, &edit_model.value).await?;
        Ok(Entry {
            metadata: entry.metadata,
            value: edit_model.value.clone(),
        })
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn delete_secret(&self, tenant_id: &str, key: &Key) -> Outcome<()> {
        self.repo.delete_secret(tenant_id, key).await
    }
}
