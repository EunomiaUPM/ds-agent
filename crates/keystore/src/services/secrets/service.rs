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

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use events::schemas::dataspace::{KeystoreAuditPayload, ResourceAction};
use events::{EventContext, EventSinkTrait, NoopEventSink, TypedEventSink};
use ymir::errors::{NotFoundExt, Outcome};
use ymir::query::QueryFilter;
use ymir::types::oauth::UserInfo;
use ymir::types::participants::Visibility;

use crate::data::repo::secrets::SecretRepoTrait;
use crate::entities::commands::{EditSecretCommand, NewSecretCommand};
use crate::entities::entry::SecretEntry;
use crate::entities::filters::PrefixFilter;
use crate::entities::key::Key;
use crate::entities::secret_value::SecretValue;
use crate::entities::version::Version;
use crate::services::secrets::SecretStore;

/// Secret store over a repository, emitting redacted `KeystoreAuditPayload` events.
pub struct SecretStoreImpl {
    repo: Arc<dyn SecretRepoTrait>,
    events: Arc<dyn EventSinkTrait>,
    l1: RwLock<HashMap<(String, String), (Instant, SecretEntry)>>,
}

impl SecretStoreImpl {
    const L1_TTL: Duration = Duration::from_secs(5);
    const L1_MAX_ENTRIES: usize = 2048;

    pub fn new(repo: Arc<dyn SecretRepoTrait>) -> Self {
        Self {
            repo,
            events: Arc::new(NoopEventSink),
            l1: RwLock::new(HashMap::new()),
        }
    }

    fn read_l1(&self, user_id: &str, key: &Key) -> Option<SecretEntry> {
        let guard = self.l1.read().ok()?;
        let (ts, entry) = guard.get(&(user_id.to_string(), key.as_str().to_string()))?;
        (ts.elapsed() < Self::L1_TTL).then(|| entry.clone())
    }

    fn write_l1(&self, user_id: &str, entry: &SecretEntry) {
        if let Ok(mut guard) = self.l1.write() {
            if guard.len() >= Self::L1_MAX_ENTRIES {
                guard.retain(|_, (ts, _)| ts.elapsed() < Self::L1_TTL);
            }
            guard.insert(
                (user_id.to_string(), entry.metadata.key.as_str().to_string()),
                (Instant::now(), entry.clone()),
            );
        }
    }

    fn evict_l1(&self, user_id: &str, key: &Key) {
        if let Ok(mut guard) = self.l1.write() {
            guard.remove(&(user_id.to_string(), key.as_str().to_string()));
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
            entry_kind: "secret".to_string(),
            entry_id: key.as_str().to_string(),
            name: key.as_str().to_string(),
            key_type: None,
            value_redacted: true,
        };
        if let Err(err) = TypedEventSink::emit(&*self.events, &ctx, &payload).await {
            tracing::warn!(error = %err, key = %key, "Failed to emit keystore secret audit event");
        }
    }
}

#[async_trait::async_trait]
impl SecretStore for SecretStoreImpl {
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn create(&self, user: &UserInfo, cmd: &NewSecretCommand) -> Outcome<SecretEntry> {
        let entry = self.repo.create_secret(user.id(), cmd).await?;
        self.write_l1(user.id(), &entry);
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
    async fn read(&self, user: &UserInfo, key: &Key) -> Outcome<SecretEntry> {
        if let Some(cached) = self.read_l1(user.id(), key) {
            return Ok(cached);
        }
        let entry = self
            .repo
            .get_secret_by_key(user.id(), key)
            .await?
            .or_not_found(key, "secret")?;
        self.write_l1(user.id(), &entry);
        Ok(entry)
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
        self.write_l1(user.id(), &entry);
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
        self.repo.delete_secret(user.id(), key).await?;
        self.evict_l1(user.id(), key);
        self.emit_audit(user, ResourceAction::Deleted, key).await;
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
                self.update(
                    user,
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
