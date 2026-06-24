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

use chrono::Utc;
use tokio::sync::RwLock;
use ymir::errors::{Outcome, RepoIntoErrors};

use crate::data::repo::parameters::{ParameterRepoErrors, ParameterRepoTrait};
use crate::data::repo::secrets::{SecretRepoErrors, SecretRepoTrait};
use crate::entities::commands::{
    EditParameterCommand, EditSecretCommand, NewParameterCommand, NewSecretCommand,
};
use crate::entities::entry::{Entry, SecretEntry};
use crate::entities::key::Key;
use crate::entities::metadata::Metadata;
use crate::entities::version::Version;

// helpers ─────────────────────────────────────────────────────────────────

fn active<T>(store: &HashMap<String, Entry<T>>) -> impl Iterator<Item = &Entry<T>> {
    store.values().filter(|e| e.metadata.deleted_at.is_none())
}

// Parameters ──────────────────────────────────────────────────────────────

pub struct InMemoryParameterRepo {
    store: Arc<RwLock<HashMap<String, Entry<serde_json::Value>>>>,
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

    async fn get_all_parameters(&self) -> Outcome<Vec<Entry<Self::Value>>> {
        let g = self.store.read().await;
        Ok(active(&*g).cloned().collect())
    }

    async fn count_parameters(&self) -> Outcome<u64> {
        let g = self.store.read().await;
        Ok(active(&*g).count() as u64)
    }

    async fn get_batch_parameters(&self, keys: &[Key]) -> Outcome<Vec<Entry<Self::Value>>> {
        let g = self.store.read().await;
        Ok(keys
            .iter()
            .filter_map(|k| {
                g.get(k.as_str())
                    .filter(|e| e.metadata.deleted_at.is_none())
                    .cloned()
            })
            .collect())
    }

    async fn get_parameter_by_key(&self, key: &Key) -> Outcome<Option<Entry<Self::Value>>> {
        let g = self.store.read().await;
        Ok(g.get(key.as_str())
            .filter(|e| e.metadata.deleted_at.is_none())
            .cloned())
    }

    async fn create_parameter(
        &self,
        cmd: &NewParameterCommand<Self::Value>,
    ) -> Outcome<Entry<Self::Value>> {
        let mut g = self.store.write().await;
        if g.get(cmd.key.as_str())
            .map_or(false, |e| e.metadata.deleted_at.is_none())
        {
            return Err(ParameterRepoErrors::ParameterAlreadyExists.into_errors());
        }
        let now = Utc::now();
        let entry = Entry {
            metadata: Metadata {
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
        g.insert(cmd.key.as_str().to_owned(), entry.clone());
        Ok(entry)
    }

    async fn put_parameter(
        &self,
        key: &Key,
        cmd: &EditParameterCommand<Self::Value>,
    ) -> Outcome<Entry<Self::Value>> {
        let mut g = self.store.write().await;

        // Collect the data we need before dropping the shared borrow.
        let (cur_version, created_at, created_by, cur_desc) = {
            let e = g
                .get(key.as_str())
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
        g.insert(key.as_str().to_owned(), updated.clone());
        Ok(updated)
    }

    async fn delete_parameter(&self, key: &Key) -> Outcome<()> {
        let mut g = self.store.write().await;
        if g.remove(key.as_str()).is_none() {
            return Err(ParameterRepoErrors::ParameterNotFound.into_errors());
        }
        Ok(())
    }
}

// Secrets ─────────────────────────────────────────────────────────────────

pub struct InMemorySecretRepo {
    store: Arc<RwLock<HashMap<String, SecretEntry>>>,
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
    async fn get_all_secrets(&self) -> Outcome<Vec<SecretEntry>> {
        let g = self.store.read().await;
        Ok(active(&*g).cloned().collect())
    }

    async fn count_secrets(&self) -> Outcome<u64> {
        let g = self.store.read().await;
        Ok(active(&*g).count() as u64)
    }

    async fn get_batch_secrets(&self, keys: &[Key]) -> Outcome<Vec<SecretEntry>> {
        let g = self.store.read().await;
        Ok(keys
            .iter()
            .filter_map(|k| {
                g.get(k.as_str())
                    .filter(|e| e.metadata.deleted_at.is_none())
                    .cloned()
            })
            .collect())
    }

    async fn get_secret_by_key(&self, key: &Key) -> Outcome<Option<SecretEntry>> {
        let g = self.store.read().await;
        Ok(g.get(key.as_str())
            .filter(|e| e.metadata.deleted_at.is_none())
            .cloned())
    }

    async fn list_secrets_by_prefix(&self, prefix: &str) -> Outcome<Vec<SecretEntry>> {
        let g = self.store.read().await;
        Ok(active(&*g)
            .filter(|e| prefix.is_empty() || e.metadata.key.as_str().starts_with(prefix))
            .cloned()
            .collect())
    }

    async fn create_secret(&self, cmd: &NewSecretCommand) -> Outcome<SecretEntry> {
        let mut g = self.store.write().await;
        if g.get(cmd.key.as_str())
            .map_or(false, |e| e.metadata.deleted_at.is_none())
        {
            return Err(SecretRepoErrors::SecretAlreadyExists.into_errors());
        }
        let now = Utc::now();
        let entry = Entry {
            metadata: Metadata {
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
        g.insert(cmd.key.as_str().to_owned(), entry.clone());
        Ok(entry)
    }

    async fn put_secret(&self, key: &Key, cmd: &EditSecretCommand) -> Outcome<SecretEntry> {
        let mut g = self.store.write().await;

        let (cur_version, created_at, created_by, cur_desc) = {
            let e = g
                .get(key.as_str())
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
        g.insert(key.as_str().to_owned(), updated.clone());
        Ok(updated)
    }

    async fn delete_secret(&self, key: &Key) -> Outcome<()> {
        let mut g = self.store.write().await;
        if g.remove(key.as_str()).is_none() {
            return Err(SecretRepoErrors::SecretNotFound.into_errors());
        }
        Ok(())
    }
}
