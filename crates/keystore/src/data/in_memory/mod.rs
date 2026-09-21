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

//! In-memory repository implementations for keystore parameters and secrets.

use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use tokio::sync::RwLock;
use ymir::errors::{Outcome, RepoIntoErrors};

use crate::data::repo::parameters::{ParameterRepoErrors, ParameterRepoTrait};
use crate::data::repo::secrets::{SecretRepoErrors, SecretRepoTrait};
use crate::entities::commands::{
    EditParameterCommand, EditSecretCommand, NewParameterCommand, NewSecretCommand,
};
use crate::entities::entry::{Entry, SecretEntry};
use crate::entities::filters::PrefixFilter;
use crate::entities::key::Key;
use crate::entities::metadata::Metadata;
use crate::entities::version::Version;

// helpers ─────────────────────────────────────────────────────────────────

fn matches_filter<T>(entry: &Entry<T>, filter: &PrefixFilter) -> bool {
    if entry.metadata.deleted_at.is_some() {
        return false;
    }
    if let Some(tenant_id) = &filter.tenant_id {
        if &entry.metadata.tenant_id != tenant_id {
            return false;
        }
    }
    if let Some(prefix) = &filter.prefix {
        if !prefix.is_empty() && !entry.metadata.key.as_str().starts_with(prefix) {
            return false;
        }
    }
    true
}

// Parameters ──────────────────────────────────────────────────────────────

pub struct InMemoryParameterRepo {
    store: Arc<RwLock<HashMap<(String, String), Entry<serde_json::Value>>>>,
}

impl InMemoryParameterRepo {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryParameterRepo {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ParameterRepoTrait for InMemoryParameterRepo {
    type Value = serde_json::Value;

    async fn get_all_parameters(&self, filter: &PrefixFilter) -> Outcome<Vec<Entry<Self::Value>>> {
        let g = self.store.read().await;
        Ok(g.values()
            .filter(|e| matches_filter(e, filter))
            .cloned()
            .collect())
    }

    async fn count_parameters(&self, filter: &PrefixFilter) -> Outcome<u64> {
        let g = self.store.read().await;
        Ok(g.values().filter(|e| matches_filter(e, filter)).count() as u64)
    }

    async fn get_batch_parameters(
        &self,
        tenant_id: &str,
        keys: &[Key],
    ) -> Outcome<Vec<Entry<Self::Value>>> {
        let g = self.store.read().await;
        Ok(keys
            .iter()
            .filter_map(|k| {
                g.get(&(tenant_id.to_string(), k.as_str().to_string()))
                    .filter(|e| e.metadata.deleted_at.is_none())
                    .cloned()
            })
            .collect())
    }

    async fn get_parameter_by_key(
        &self,
        tenant_id: &str,
        key: &Key,
    ) -> Outcome<Option<Entry<Self::Value>>> {
        let g = self.store.read().await;
        Ok(g.get(&(tenant_id.to_string(), key.as_str().to_string()))
            .filter(|e| e.metadata.deleted_at.is_none())
            .cloned())
    }

    async fn create_parameter(
        &self,
        tenant_id: &str,
        cmd: &NewParameterCommand<Self::Value>,
    ) -> Outcome<Entry<Self::Value>> {
        let mut g = self.store.write().await;
        let map_key = (tenant_id.to_string(), cmd.key.as_str().to_owned());
        if g.get(&map_key)
            .map_or(false, |e| e.metadata.deleted_at.is_none())
        {
            return Err(ParameterRepoErrors::ParameterAlreadyExists.into_errors());
        }
        let now = Utc::now();
        let entry = Entry {
            metadata: Metadata {
                tenant_id: tenant_id.to_string(),
                key: cmd.key.clone(),
                version: Version::INITIAL,
                created_at: now,
                updated_at: now,
                created_by: String::new(),
                updated_by: String::new(),
                deleted_at: None,
                description: cmd.description.clone(),
            },
            value: cmd.value.clone(),
        };
        g.insert(map_key, entry.clone());
        Ok(entry)
    }

    async fn put_parameter(
        &self,
        tenant_id: &str,
        key: &Key,
        cmd: &EditParameterCommand<Self::Value>,
    ) -> Outcome<Entry<Self::Value>> {
        let mut g = self.store.write().await;
        let map_key = (tenant_id.to_string(), key.as_str().to_owned());

        let (cur_version, created_at, created_by, cur_desc) = {
            let e = g
                .get(&map_key)
                .filter(|e| e.metadata.deleted_at.is_none())
                .ok_or_else(|| ParameterRepoErrors::ParameterNotFound.into_errors())?;

            if e.metadata.version != cmd.expected_version {
                return Err(ParameterRepoErrors::VersionConflict {
                    expected: cmd.expected_version,
                    actual: e.metadata.version,
                }
                .into_errors());
            }
            (
                e.metadata.version,
                e.metadata.created_at,
                e.metadata.created_by.clone(),
                e.metadata.description.clone(),
            )
        };

        let now = Utc::now();
        let updated = Entry {
            metadata: Metadata {
                tenant_id: tenant_id.to_string(),
                key: key.clone(),
                version: cur_version.next(),
                created_at,
                updated_at: now,
                created_by,
                updated_by: String::new(),
                deleted_at: None,
                description: cmd.description.clone().or(cur_desc),
            },
            value: cmd.value.clone(),
        };
        g.insert(map_key, updated.clone());
        Ok(updated)
    }

    async fn delete_parameter(&self, tenant_id: &str, key: &Key) -> Outcome<()> {
        let mut g = self.store.write().await;
        let map_key = (tenant_id.to_string(), key.as_str().to_owned());
        if g.remove(&map_key).is_none() {
            return Err(ParameterRepoErrors::ParameterNotFound.into_errors());
        }
        Ok(())
    }
}

// Secrets ─────────────────────────────────────────────────────────────────

pub struct InMemorySecretRepo {
    store: Arc<RwLock<HashMap<(String, String), SecretEntry>>>,
}

impl InMemorySecretRepo {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemorySecretRepo {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl SecretRepoTrait for InMemorySecretRepo {
    async fn get_all_secrets(&self, filter: &PrefixFilter) -> Outcome<Vec<SecretEntry>> {
        let g = self.store.read().await;
        Ok(g.values()
            .filter(|e| matches_filter(e, filter))
            .cloned()
            .collect())
    }

    async fn count_secrets(&self, filter: &PrefixFilter) -> Outcome<u64> {
        let g = self.store.read().await;
        Ok(g.values().filter(|e| matches_filter(e, filter)).count() as u64)
    }

    async fn get_batch_secrets(&self, tenant_id: &str, keys: &[Key]) -> Outcome<Vec<SecretEntry>> {
        let g = self.store.read().await;
        Ok(keys
            .iter()
            .filter_map(|k| {
                g.get(&(tenant_id.to_string(), k.as_str().to_string()))
                    .filter(|e| e.metadata.deleted_at.is_none())
                    .cloned()
            })
            .collect())
    }

    async fn get_secret_by_key(&self, tenant_id: &str, key: &Key) -> Outcome<Option<SecretEntry>> {
        let g = self.store.read().await;
        Ok(g.get(&(tenant_id.to_string(), key.as_str().to_string()))
            .filter(|e| e.metadata.deleted_at.is_none())
            .cloned())
    }

    async fn create_secret(&self, tenant_id: &str, cmd: &NewSecretCommand) -> Outcome<SecretEntry> {
        let mut g = self.store.write().await;
        let map_key = (tenant_id.to_string(), cmd.key.as_str().to_owned());
        if g.get(&map_key)
            .map_or(false, |e| e.metadata.deleted_at.is_none())
        {
            return Err(SecretRepoErrors::SecretAlreadyExists.into_errors());
        }
        let now = Utc::now();
        let entry = Entry {
            metadata: Metadata {
                tenant_id: tenant_id.to_string(),
                key: cmd.key.clone(),
                version: Version::INITIAL,
                created_at: now,
                updated_at: now,
                created_by: String::new(),
                updated_by: String::new(),
                deleted_at: None,
                description: cmd.description.clone(),
            },
            value: cmd.value.clone(),
        };
        g.insert(map_key, entry.clone());
        Ok(entry)
    }

    async fn put_secret(
        &self,
        tenant_id: &str,
        key: &Key,
        cmd: &EditSecretCommand,
    ) -> Outcome<SecretEntry> {
        let mut g = self.store.write().await;
        let map_key = (tenant_id.to_string(), key.as_str().to_owned());

        let (cur_version, created_at, created_by, cur_desc) = {
            let e = g
                .get(&map_key)
                .filter(|e| e.metadata.deleted_at.is_none())
                .ok_or_else(|| SecretRepoErrors::SecretNotFound.into_errors())?;

            if e.metadata.version != cmd.expected_version {
                return Err(SecretRepoErrors::VersionConflict {
                    expected: cmd.expected_version,
                    actual: e.metadata.version,
                }
                .into_errors());
            }
            (
                e.metadata.version,
                e.metadata.created_at,
                e.metadata.created_by.clone(),
                e.metadata.description.clone(),
            )
        };

        let now = Utc::now();
        let updated = Entry {
            metadata: Metadata {
                tenant_id: tenant_id.to_string(),
                key: key.clone(),
                version: cur_version.next(),
                created_at,
                updated_at: now,
                created_by,
                updated_by: String::new(),
                deleted_at: None,
                description: cmd.description.clone().or(cur_desc),
            },
            value: cmd.value.clone(),
        };
        g.insert(map_key, updated.clone());
        Ok(updated)
    }

    async fn delete_secret(&self, tenant_id: &str, key: &Key) -> Outcome<()> {
        let mut g = self.store.write().await;
        let map_key = (tenant_id.to_string(), key.as_str().to_owned());
        if g.remove(&map_key).is_none() {
            return Err(SecretRepoErrors::SecretNotFound.into_errors());
        }
        Ok(())
    }
}
