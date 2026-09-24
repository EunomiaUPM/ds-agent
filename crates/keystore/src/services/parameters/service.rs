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

use crate::data::repo::parameters::ParameterRepoTrait;
use crate::entities::commands::{EditParameterCommand, NewParameterCommand};
use crate::entities::entry::Entry;
use crate::entities::filters::PrefixFilter;
use crate::entities::key::Key;
use crate::entities::version::Version;
use crate::services::parameters::ParameterStore;

pub struct ParameterStoreImpl {
    repo: Arc<dyn ParameterRepoTrait<Value = serde_json::Value>>,
    event_bus: Option<events::EventBus>,
}

impl ParameterStoreImpl {
    pub fn new(repo: Arc<dyn ParameterRepoTrait<Value = serde_json::Value>>) -> Self {
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
impl ParameterStore<serde_json::Value> for ParameterStoreImpl {
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn create(
        &self,
        scope: &AccessScope,
        cmd: &NewParameterCommand<serde_json::Value>,
    ) -> Outcome<Entry<serde_json::Value>> {
        let mut cmd = cmd.clone();
        let target_tenant = scope.resolve_create_tenant(cmd.tenant_id.as_deref())?;
        cmd.tenant_id = Some(target_tenant.clone());
        let entry = self.repo.create_parameter(&target_tenant, &cmd).await?;
        events::emit_action!(
            self.event_bus,
            &target_tenant,
            crate::EVENT_PREFIX,
            "parameter",
            "create",
            &entry
        );
        Ok(entry)
    }

    #[tracing::instrument(level = "info", skip(self, scope), fields(key = %key), err)]
    async fn read(&self, scope: &AccessScope, key: &Key) -> Outcome<Entry<serde_json::Value>> {
        scope.require_read()?;
        self.repo
            .get_parameter_by_key(scope.acting_tenant(), key)
            .await?
            .or_not_found(key, "parameter")
    }

    #[tracing::instrument(level = "info", skip(self, scope, cmd, actor), fields(key = %key), err)]
    async fn update(
        &self,
        scope: &AccessScope,
        key: &Key,
        cmd: &EditParameterCommand<serde_json::Value>,
        actor: &str,
    ) -> Outcome<Version> {
        scope.require_write()?;
        let _ = actor;
        let entry = self
            .repo
            .put_parameter(scope.acting_tenant(), key, cmd)
            .await?;
        events::emit_action!(
            self.event_bus,
            scope.acting_tenant(),
            crate::EVENT_PREFIX,
            "parameter",
            "edit",
            &entry
        );
        Ok(entry.metadata.version)
    }

    #[tracing::instrument(level = "info", skip(self, scope), fields(key = %key), err)]
    async fn delete(&self, scope: &AccessScope, key: &Key) -> Outcome<()> {
        scope.require_write()?;
        self.repo
            .delete_parameter(scope.acting_tenant(), key)
            .await?;
        events::emit_action!(
            self.event_bus,
            scope.acting_tenant(),
            crate::EVENT_PREFIX,
            "parameter",
            "delete",
            &events::EntityDeletedDto::new(key.as_str())
        );
        Ok(())
    }

    #[tracing::instrument(level = "info", skip(self, scope), err)]
    async fn list(
        &self,
        scope: &AccessScope,
        filter: &PrefixFilter,
    ) -> Outcome<Vec<Entry<serde_json::Value>>> {
        scope.require_read()?;
        filter.validate()?;
        let mut filter = filter.clone();
        filter.tenant_id = scope.resolve_query_tenant(filter.tenant_id.as_deref())?;
        self.repo.get_all_parameters(&filter).await
    }

    #[tracing::instrument(level = "info", skip_all, err)]
    async fn batch(
        &self,
        scope: &AccessScope,
        keys: &[Key],
    ) -> Outcome<Vec<Entry<serde_json::Value>>> {
        scope.require_read()?;
        if keys.is_empty() {
            return Ok(vec![]);
        }
        self.repo
            .get_batch_parameters(scope.acting_tenant(), keys)
            .await
    }
}
