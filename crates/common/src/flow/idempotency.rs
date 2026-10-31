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

//! Idempotent flows: a repeated message gets the first answer, keyed on what the protocol says
//! identifies a message, with its canonical hash as a guard and the process version for restarts.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde::Serialize;
use ymir::errors::{Errors, Outcome};

use crate::flow::{FlowCx, FlowError, FlowGuardTrait, FlowTrait, GuardVerdict};
use crate::validation::{codes, violation};

ymir::str_id!(IdempotencyKey, String);

/// What the store knows about a key it has seen before.
#[derive(Debug, Clone)]
pub struct IdempotencyRecord {
    pub key: IdempotencyKey,
    /// The canonical hash of the message that first claimed this key.
    pub canonical_hash: [u8; 32],
    pub first_seen_at: DateTime<Utc>,
    /// Process version when the key was claimed; `None` when the message creates the process.
    pub process_version_at_claim: Option<u64>,
    /// Process version the message left behind, what a retry must still find; `None` in flight.
    pub process_version_after: Option<u64>,
    /// The first answer, replayed on a retry; `None` while the first message is in flight.
    pub response: Option<serde_json::Value>,
}

impl IdempotencyRecord {
    /// Whether `current` shows the process moved on since, by something other than this message.
    fn superseded_by(&self, current: Option<u64>) -> bool {
        match (
            self.process_version_after.or(self.process_version_at_claim),
            current,
        ) {
            (Some(expected), Some(current)) => current > expected,
            _ => false,
        }
    }

    fn conflict_reason(&self) -> String {
        format!(
            "idempotency key {} was first used at {} for a message with different content",
            self.key, self.first_seen_at
        )
    }
}

/// The outcome of checking a message against the store.
#[derive(Debug)]
pub enum IdempotencyVerdict {
    /// First time this key is seen: execute.
    New(IdempotencyKey),
    /// Same message again: replay the stored answer (`None` while still in flight).
    Replay(Box<IdempotencyRecord>),
    /// Same key, different content: the peer reused an identity, not a retry.
    Conflict(Box<IdempotencyRecord>),
    /// Same message, but the process moved on since: a new transition, such as a restart.
    Superseded {
        key: IdempotencyKey,
        previous: Box<IdempotencyRecord>,
    },
}

/// Storage of idempotency records.
#[mockall::automock]
#[async_trait::async_trait]
pub trait IdempotencyStoreTrait: Send + Sync {
    async fn get_idempotency_record(
        &self,
        key: &IdempotencyKey,
    ) -> Outcome<Option<IdempotencyRecord>>;
    /// Stores the record, replacing any earlier one with the same key.
    async fn put_idempotency_record(&self, record: IdempotencyRecord) -> Outcome<()>;
}

/// Adequate for one process; several replicas need shared storage.
#[derive(Debug, Default)]
pub struct InMemoryIdempotencyStore {
    records: Mutex<HashMap<String, IdempotencyRecord>>,
}

impl InMemoryIdempotencyStore {
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> Outcome<std::sync::MutexGuard<'_, HashMap<String, IdempotencyRecord>>> {
        self.records
            .lock()
            .map_err(|_| Errors::crazy("idempotency store mutex is poisoned", None))
    }
}

#[async_trait::async_trait]
impl IdempotencyStoreTrait for InMemoryIdempotencyStore {
    async fn get_idempotency_record(
        &self,
        key: &IdempotencyKey,
    ) -> Outcome<Option<IdempotencyRecord>> {
        Ok(self.lock()?.get(key.as_str()).cloned())
    }

    async fn put_idempotency_record(&self, record: IdempotencyRecord) -> Outcome<()> {
        self.lock()?.insert(record.key.as_str().to_string(), record);
        Ok(())
    }
}

/// What a flow's facts say about the message they were loaded for.
pub trait IdempotentFactsTrait {
    fn idempotency_key(&self) -> &IdempotencyKey;
    fn canonical_hash(&self) -> &[u8; 32];
    /// The process version now, or `None` when the message creates the process.
    fn process_version(&self) -> Option<u64>;
}

/// What a flow's output says about the message it answered.
pub trait IdempotentOutputTrait {
    fn idempotency_key(&self) -> &IdempotencyKey;
    /// The process version the message left behind.
    fn process_version(&self) -> Option<u64>;
}

/// Decides each message's verdict over a store; as a flow guard it claims before applying
/// and records the answer after commit.
#[derive(Clone)]
pub struct IdempotencyGuard {
    store: Arc<dyn IdempotencyStoreTrait>,
}

impl IdempotencyGuard {
    pub fn new(store: Arc<dyn IdempotencyStoreTrait>) -> Self {
        Self { store }
    }

    /// Classifies a message; the version tells a retry from a restart carrying the same pids.
    pub async fn check(
        &self,
        key: &IdempotencyKey,
        canonical_hash: &[u8; 32],
        process_version: Option<u64>,
    ) -> Outcome<IdempotencyVerdict> {
        let Some(record) = self.store.get_idempotency_record(key).await? else {
            return Ok(IdempotencyVerdict::New(key.clone()));
        };
        // Once the process moved on, the same key is a new transition whatever it carries (a
        // restart may send another data address); before that, other content is a conflict.
        if record.superseded_by(process_version) {
            return Ok(IdempotencyVerdict::Superseded {
                key: key.clone(),
                previous: Box::new(record),
            });
        }
        if &record.canonical_hash != canonical_hash {
            return Ok(IdempotencyVerdict::Conflict(Box::new(record)));
        }
        Ok(IdempotencyVerdict::Replay(Box::new(record)))
    }

    /// Claims the key for a message about to run; overwrites a superseded record.
    pub async fn claim(
        &self,
        key: IdempotencyKey,
        canonical_hash: [u8; 32],
        process_version: Option<u64>,
    ) -> Outcome<IdempotencyRecord> {
        let record = IdempotencyRecord {
            key,
            canonical_hash,
            first_seen_at: Utc::now(),
            process_version_at_claim: process_version,
            process_version_after: None,
            response: None,
        };
        self.store.put_idempotency_record(record.clone()).await?;
        Ok(record)
    }

    /// Attaches the answer and the version it left, so a retry can be replayed.
    pub async fn record_response(
        &self,
        key: &IdempotencyKey,
        response: serde_json::Value,
        process_version_after: Option<u64>,
    ) -> Outcome<()> {
        let Some(mut record) = self.store.get_idempotency_record(key).await? else {
            return Ok(());
        };
        record.response = Some(response);
        record.process_version_after = process_version_after;
        self.store.put_idempotency_record(record).await
    }

    fn conflict(reason: impl Into<String>) -> FlowError {
        FlowError::Conflict(violation("@type", codes::STALE, reason.into()))
    }
}

#[async_trait::async_trait]
impl<F> FlowGuardTrait<F> for IdempotencyGuard
where
    F: FlowTrait,
    F::Facts: IdempotentFactsTrait,
    F::Output: IdempotentOutputTrait + Serialize + DeserializeOwned,
{
    async fn after_load(
        &self,
        _cx: &FlowCx<'_, F::Caller>,
        _key: &F::Key,
        facts: &F::Facts,
    ) -> Result<GuardVerdict<F::Output>, FlowError> {
        let version = facts.process_version();
        match self
            .check(facts.idempotency_key(), facts.canonical_hash(), version)
            .await?
        {
            IdempotencyVerdict::New(key) | IdempotencyVerdict::Superseded { key, .. } => {
                self.claim(key, *facts.canonical_hash(), version).await?;
                Ok(GuardVerdict::Continue)
            }
            IdempotencyVerdict::Replay(record) => match record.response {
                Some(response) => serde_json::from_value(response)
                    .map(GuardVerdict::Replay)
                    .map_err(|e| Self::conflict(format!("stored reply is unreadable: {e}"))),
                None => Err(Self::conflict("the same message is still being processed")),
            },
            IdempotencyVerdict::Conflict(record) => Err(Self::conflict(record.conflict_reason())),
        }
    }

    async fn after_commit(
        &self,
        _caller: &F::Caller,
        _key: &F::Key,
        output: &F::Output,
    ) -> Outcome<()> {
        let response = serde_json::to_value(output).unwrap_or_default();
        self.record_response(output.idempotency_key(), response, output.process_version())
            .await
    }
}
