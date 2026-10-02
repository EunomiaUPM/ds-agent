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

//! Idempotent handling of inbound DSP messages.
//!
//! The key is protocol identity — message type plus pids — because DSP already
//! says two requests with the same `consumerPid` are the same request. Neither
//! hash can key it: the wire hash is too strict, the canonical one too blind. The
//! canonical hash rides along as a guard, and [`TransferProcess::version`] tells a
//! legitimate restart apart from a replay when the pids alone cannot.
//!
//! [`TransferProcess::version`]: crate::entities::transfer_process::TransferProcess::version

use std::collections::HashMap;
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::entities::ids::IdempotencyKey;
use crate::entities::protocol::ProtocolId;
use crate::protocols::dsp::entities::auth::TransferDSPAuthn;
use crate::protocols::dsp::entities::context_common::TransferContextRaw;
use crate::protocols::dsp::entities::context_dsp::TransferDSPContextTyped;
use crate::protocols::dsp::entities::message_types::TransferDSPMessageType;

/// What the store knows about a key it has seen before.
#[derive(Debug, Clone)]
pub struct IdempotencyRecord {
    pub key: IdempotencyKey,
    /// The canonical hash of the message that first claimed this key.
    pub canonical_hash: [u8; 32],
    pub first_seen_at: DateTime<Utc>,
    /// `TransferProcess::version` when the key was claimed. `None` on
    /// `/request`, where the message *creates* the process.
    pub process_version_at_claim: Option<u64>,
    /// `TransferProcess::version` after the message applied — what a retry should
    /// still find. `None` while in flight.
    pub process_version_after: Option<u64>,
    /// The ack returned the first time, replayed verbatim on a retry. `None`
    /// while the first request is still in flight.
    pub response: Option<serde_json::Value>,
}

impl IdempotencyRecord {
    /// What a retry should still see: the version this message's transition left
    /// behind, or the one it started from while still in flight.
    fn expected_process_version(&self) -> Option<u64> {
        self.process_version_after.or(self.process_version_at_claim)
    }

    /// Whether `current` shows the process has moved on since this record was
    /// written, by something other than the message that wrote it.
    fn superseded_by(&self, current: Option<u64>) -> bool {
        match (self.expected_process_version(), current) {
            (Some(expected), Some(current)) => current > expected,
            // No process to compare against (`/request`, or a caller that does not
            // know the version): nothing can be said, so nothing is claimed.
            _ => false,
        }
    }
}

/// The outcome of checking a message against the store.
#[derive(Debug)]
pub enum IdempotencyVerdict {
    /// First time this key is seen — proceed and execute.
    New(IdempotencyKey),
    /// Same message again: replay the stored ack. A `None` response means the
    /// first request has not answered yet.
    Replay(Box<IdempotencyRecord>),
    /// Same key, different content: the peer reused a pid. A protocol violation,
    /// not a retry.
    Conflict(Box<IdempotencyRecord>),
    /// Same message, but the process has moved on since — a new transition that
    /// merely looks like the old one, such as a restart after a suspension.
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

impl IdempotencyKey {
    /// Participant, route, version, type, pids and the peer's header, folded
    /// together. The participant is in because pids are unique only per peer; the
    /// header is folded rather than substituted so a peer can neither pick
    /// another's key nor shed its own; components are length-prefixed so no pid can
    /// imitate a different tuple.
    pub fn derive(
        raw: &TransferContextRaw<TransferDSPAuthn>,
        dsp_version: &ProtocolId,
        message: &TransferDSPMessageType,
        consumer_pid: Option<&str>,
        provider_pid: Option<&str>,
    ) -> Self {
        let mut hasher = Sha256::new();
        let mut field = |bytes: &[u8]| {
            hasher.update((bytes.len() as u64).to_be_bytes());
            hasher.update(bytes);
        };
        field(raw.authn.associated_participant.participant_id.as_bytes());
        field(raw.request_path.as_bytes());
        // The protocol version, because `request_path` cannot stand in for it: a
        // nested router hands the handler the path with its mount prefix stripped, so
        // a connector serving 2024-1 and 2025-1 side by side (DSP 4.3) sees `/request`
        // for both, and the same `consumerPid` would collide across versions.
        field(dsp_version.to_string().as_bytes());
        field(message.to_string().as_bytes());
        field(consumer_pid.unwrap_or_default().as_bytes());
        field(provider_pid.unwrap_or_default().as_bytes());
        field(
            raw.supplied_idempotency_key
                .as_ref()
                .map(|k| k.as_str())
                .unwrap_or_default()
                .as_bytes(),
        );

        Self::new(format!("dsp-{}", hex::encode(hasher.finalize())))
    }
}

/// Derives the key and decides the verdict. Holds no state of its own — the
/// store does.
pub struct IdempotencyGuard<'a> {
    store: &'a dyn IdempotencyStoreTrait,
}

impl<'a> IdempotencyGuard<'a> {
    pub fn new(store: &'a dyn IdempotencyStoreTrait) -> Self {
        Self { store }
    }

    /// Classify a message.
    ///
    /// `process_version` is `TransferProcess::version` as it stands right now,
    /// or `None` on `/request`, where the message creates the process and there is
    /// nothing yet to compare against. It is what separates a retry from a
    /// restart: both carry the same pids and the same bytes, but a restart arrives
    /// with the process moved on by the suspension in between.
    pub async fn check(
        &self,
        typed: &TransferDSPContextTyped,
        process_version: Option<u64>,
    ) -> Outcome<IdempotencyVerdict> {
        let key = typed.idempotency_key.clone();
        let Some(record) = self.store.get_idempotency_record(&key).await? else {
            return Ok(IdempotencyVerdict::New(key));
        };
        if record.canonical_hash != typed.rdf.canonical_hash {
            return Ok(IdempotencyVerdict::Conflict(Box::new(record)));
        }
        if record.superseded_by(process_version) {
            return Ok(IdempotencyVerdict::Superseded {
                key,
                previous: Box::new(record),
            });
        }
        Ok(IdempotencyVerdict::Replay(Box::new(record)))
    }

    /// Claim the key for a message about to be executed, recording the canonical
    /// hash every later retry is compared against and the process version it
    /// starts from. Overwrites a superseded record.
    pub async fn claim(
        &self,
        key: IdempotencyKey,
        typed: &TransferDSPContextTyped,
        process_version: Option<u64>,
    ) -> Outcome<IdempotencyRecord> {
        let record = IdempotencyRecord {
            key,
            canonical_hash: typed.rdf.canonical_hash,
            first_seen_at: typed.rdf.parsed.raw.incoming_at,
            process_version_at_claim: process_version,
            process_version_after: None,
            response: None,
        };
        self.store.put_idempotency_record(record.clone()).await?;
        Ok(record)
    }

    /// Attach the ack once the message has been handled, so a retry can replay it,
    /// along with the process version the transition left behind — the version a
    /// retry must still find for this to count as a retry rather than a restart.
    pub async fn record_response(
        &self,
        mut record: IdempotencyRecord,
        response: serde_json::Value,
        process_version_after: Option<u64>,
    ) -> Outcome<()> {
        record.response = Some(response);
        record.process_version_after = process_version_after;
        self.store.put_idempotency_record(record).await
    }
}

impl IdempotencyRecord {
    /// The peer reused a key for a materially different message. Should be a 409;
    /// answers 400 because `ymir`'s taxonomy has no conflict variant yet.
    pub fn conflict_error(&self) -> Errors {
        Errors::format(
            BadFormat::Received,
            format!(
                "idempotency key {} was first used at {} for a message with different content",
                self.key, self.first_seen_at
            ),
            None,
        )
    }
}

/// Adequate for a single process. More than one replica needs shared storage, or
/// each will think it saw the message first.
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
