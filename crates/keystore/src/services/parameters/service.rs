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

use crate::data::repo::parameters::{ParameterRepoErrors, ParameterRepoTrait};
use crate::entities::commands::{EditParameterCommand, NewParameterCommand};
use crate::entities::entry::Entry;
use crate::entities::key::{Key, KeyPrefix};
use crate::entities::version::Version;
use crate::services::parameters::ParameterStore;

pub struct ParameterStoreImpl {
    repo: Arc<dyn ParameterRepoTrait<Value = serde_json::Value>>,
}

impl ParameterStoreImpl {
    pub fn new(repo: Arc<dyn ParameterRepoTrait<Value = serde_json::Value>>) -> Self {
        Self { repo }
    }
}

#[async_trait::async_trait]
impl ParameterStore<serde_json::Value> for ParameterStoreImpl {
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn create(
        &self,
        cmd: &NewParameterCommand<serde_json::Value>,
    ) -> Outcome<Entry<serde_json::Value>> {
        self.repo.create_parameter(cmd).await
    }

    #[tracing::instrument(level = "info", skip(self), fields(key = %key), err)]
    async fn read(&self, key: &Key) -> Outcome<Entry<serde_json::Value>> {
        self.repo
            .get_parameter_by_key(key)
            .await?
            .ok_or_else(|| ParameterRepoErrors::ParameterNotFound.into_errors())
    }

    #[tracing::instrument(level = "info", skip(self, cmd, actor), fields(key = %key), err)]
    async fn update(
        &self,
        key: &Key,
        cmd: &EditParameterCommand<serde_json::Value>,
        actor: &str,
    ) -> Outcome<Version> {
        let _ = actor;
        let entry = self.repo.put_parameter(key, cmd).await?;
        Ok(entry.metadata.version)
    }

    #[tracing::instrument(level = "info", skip(self), fields(key = %key), err)]
    async fn delete(&self, key: &Key) -> Outcome<()> {
        self.repo.delete_parameter(key).await
    }

    #[tracing::instrument(level = "info", skip(self), err)]
    async fn list(&self, prefix: &KeyPrefix) -> Outcome<Vec<Entry<serde_json::Value>>> {
        let all = self.repo.get_all_parameters().await?;
        let prefix_str = prefix.as_str();
        Ok(all
            .into_iter()
            .filter(|e| prefix_str.is_empty() || e.metadata.key.as_str().starts_with(prefix_str))
            .collect())
    }
}
