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

use events::schemas::dataspace::{KeystoreAuditPayload, ResourceAction};
use events::{EventContext, EventSinkTrait, NoopEventSink, TypedEventSink};
use ymir::errors::{NotFoundExt, Outcome};
use ymir::query::QueryFilter;
use ymir::types::oauth::UserInfo;
use ymir::types::participants::Visibility;

use crate::data::repo::parameters::ParameterRepoTrait;
use crate::entities::commands::{EditParameterCommand, NewParameterCommand};
use crate::entities::entry::Entry;
use crate::entities::filters::PrefixFilter;
use crate::entities::key::Key;
use crate::entities::version::Version;
use crate::services::parameters::ParameterStore;

/// Parameter store over a repository, emitting redacted `KeystoreAuditPayload` events.
pub struct ParameterStoreImpl {
    repo: Arc<dyn ParameterRepoTrait<Value = serde_json::Value>>,
    events: Arc<dyn EventSinkTrait>,
}

impl ParameterStoreImpl {
    pub fn new(repo: Arc<dyn ParameterRepoTrait<Value = serde_json::Value>>) -> Self {
        Self {
            repo,
            events: Arc::new(NoopEventSink),
        }
    }

    /// Publishes create, update and delete audit events on `events`.
    pub fn with_event_sink(mut self, events: Arc<dyn EventSinkTrait>) -> Self {
        self.events = events;
        self
    }

    async fn emit_audit(&self, user: &UserInfo, action: ResourceAction, key: &Key) {
        let ctx = EventContext::for_user_with_visibility(user, Visibility::Private);
        let payload = KeystoreAuditPayload {
            action,
            entry_kind: "parameter".to_string(),
            entry_id: key.as_str().to_string(),
            name: key.as_str().to_string(),
            key_type: None,
            value_redacted: true,
        };
        if let Err(err) = TypedEventSink::emit(&*self.events, &ctx, &payload).await {
            tracing::warn!(error = %err, key = %key, "Failed to emit keystore parameter audit event");
        }
    }
}

#[async_trait::async_trait]
impl ParameterStore<serde_json::Value> for ParameterStoreImpl {
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn create(
        &self,
        user: &UserInfo,
        cmd: &NewParameterCommand<serde_json::Value>,
    ) -> Outcome<Entry<serde_json::Value>> {
        let entry = self.repo.create_parameter(user.id(), cmd).await?;
        self.emit_audit(user, ResourceAction::Created, &entry.metadata.key)
            .await;
        Ok(entry)
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(user = %user.id(), key = %key)
    )]
    async fn read(&self, user: &UserInfo, key: &Key) -> Outcome<Entry<serde_json::Value>> {
        self.repo
            .get_parameter_by_key(user.id(), key)
            .await?
            .or_not_found(key, "parameter")
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
        cmd: &EditParameterCommand<serde_json::Value>,
        actor: &str,
    ) -> Outcome<Version> {
        let _ = actor;
        let entry = self
            .repo
            .put_parameter(user.id(), key, cmd)
            .await?;
        self.emit_audit(user, ResourceAction::Updated, &entry.metadata.key)
            .await;
        Ok(entry.metadata.version)
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(user = %user.id(), key = %key)
    )]
    async fn delete(&self, user: &UserInfo, key: &Key) -> Outcome<()> {
        self.repo
            .delete_parameter(user.id(), key)
            .await?;
        self.emit_audit(user, ResourceAction::Deleted, key).await;
        Ok(())
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn list(
        &self,
        user: &UserInfo,
        filter: &PrefixFilter,
    ) -> Outcome<Vec<Entry<serde_json::Value>>> {
        filter.validate()?;
        let mut filter = filter.clone();
        filter.user_id = crate::services::owner_for_list(user, filter.user_id.as_deref())?;
        self.repo.get_all_parameters(&filter).await
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn batch(
        &self,
        user: &UserInfo,
        keys: &[Key],
    ) -> Outcome<Vec<Entry<serde_json::Value>>> {
        if keys.is_empty() {
            return Ok(vec![]);
        }
        self.repo
            .get_batch_parameters(user.id(), keys)
            .await
    }
}
