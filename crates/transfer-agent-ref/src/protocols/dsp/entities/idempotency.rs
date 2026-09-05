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

use std::collections::HashMap;
use std::ops::Deref;
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
use common::dsp_common::well_known_types::DSPProtocolVersions;

/// What the store knows about a key it has seen before.
#[derive(Debug, Clone)]
pub struct IdempotencyRecord {
    pub key: IdempotencyKey,
    /// The canonical hash of the message that first claimed this key.
    pub canonical_hash: [u8; 32],
    pub first_seen_at: DateTime<Utc>,
    /// [`TransferProcess::version`] when the key was claimed. `None` on
    /// `/request`, where the message *creates* the process.
    pub process_version_at_claim: Option<u64>,
    /// [`TransferProcess::version`] after the message applied — what a retry should
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

#[allow(dead_code)]
#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait IdempotencyStoreTrait: Send + Sync {
    async fn get_idempotency_record(
        &self,
        key: &IdempotencyKey,
    ) -> Outcome<Option<IdempotencyRecord>>;
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
    /// `process_version` is [`TransferProcess::version`] as it stands right now,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocols::dsp::entities::auth::TransferDSPAuthn;
    use crate::protocols::dsp::entities::context_common::TransferContextRaw;
    use crate::protocols::dsp::entities::context_dsp::TransferDSPContextParsed;
    use crate::protocols::dsp::entities::message_types::TransferDSPMessageType;
    use axum::extract::Request;
    use common::dsp_common::well_known_types::DSPProtocolVersions;
    use serde_json::Value;
    use ymir::data::entities::shared::participant::Model as Mates;
    use ymir::types::participants::ParticipantType;

    const CTX: &str = "https://w3id.org/dspace/2025/1/context.jsonld";

    fn mate(participant_id: &str) -> Mates {
        let t = Utc::now();
        Mates {
            participant_id: participant_id.into(),
            participant_type: ParticipantType::Agent,
            participant_nick: "peer".to_string(),
            base_url: "http://127.0.0.1:1100".to_string(),
            token: None,
            saved_at: t.into(),
            last_interaction: t.into(),
            extra_fields: Value::Null,
            is_me: false,
        }
    }

    /// Build the typed context a handler would have, for a body sent by `peer`.
    async fn typed_from(peer: &str, body: String) -> TransferDSPContextTyped {
        typed_from_with_key(peer, body, None).await
    }

    async fn typed_from_with_key(
        peer: &str,
        body: String,
        supplied_key: Option<&str>,
    ) -> TransferDSPContextTyped {
        typed_at(
            peer,
            body,
            supplied_key,
            "/request",
            TransferDSPMessageType::TransferRequestMessage,
        )
        .await
    }

    async fn typed_at(
        peer: &str,
        body: String,
        supplied_key: Option<&str>,
        uri: &str,
        message_type: TransferDSPMessageType,
    ) -> TransferDSPContextTyped {
        typed_versioned(
            peer,
            body,
            supplied_key,
            uri,
            message_type,
            ProtocolId::Dsp2025_1,
        )
        .await
    }

    async fn typed_versioned(
        peer: &str,
        body: String,
        supplied_key: Option<&str>,
        uri: &str,
        message_type: TransferDSPMessageType,
        dsp_version: ProtocolId,
    ) -> TransferDSPContextTyped {
        let mut builder = Request::builder()
            .method("POST")
            .uri(uri.to_string())
            .header("authorization", "Bearer tok");
        if let Some(key) = supplied_key {
            builder = builder.header("idempotency-key", key);
        }
        let mut req = builder.body(axum::body::Body::from(body)).unwrap();
        req.extensions_mut().insert(mate(peer));
        let raw = TransferContextRaw::<TransferDSPAuthn>::from_request(req)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&raw.body_bytes).unwrap();
        let parsed =
            TransferDSPContextParsed::from_raw(raw, &dsp_version, &message_type, json).unwrap();
        let rdf = crate::protocols::dsp::entities::context_dsp::TransferDSPContextRdf::from_parsed(
            parsed,
        )
        .await
        .unwrap();
        TransferDSPContextTyped::from_rdf(rdf).unwrap()
    }

    fn message(consumer_pid: &str, callback: &str) -> String {
        format!(
            r#"{{"@context":"{CTX}","@type":"TransferRequestMessage","consumerPid":"{consumer_pid}","callbackAddress":"{callback}"}}"#
        )
    }

    #[tokio::test]
    async fn first_message_is_new_then_a_retry_replays() {
        let store = InMemoryIdempotencyStore::new();
        let guard = IdempotencyGuard::new(&store);
        let typed = typed_from(
            "did:example:consumer",
            message("urn:uuid:cc", "https://c/cb"),
        )
        .await;

        let IdempotencyVerdict::New(key) = guard.check(&typed, None).await.unwrap() else {
            panic!("first sighting must be New");
        };
        guard.claim(key, &typed, None).await.unwrap();

        // The peer retries the very same message.
        let retry = typed_from(
            "did:example:consumer",
            message("urn:uuid:cc", "https://c/cb"),
        )
        .await;
        assert!(matches!(
            guard.check(&retry, None).await.unwrap(),
            IdempotencyVerdict::Replay(_)
        ));
    }

    /// The case the whole split exists for: a retry that took a different route
    /// through a proxy arrives with different bytes but the same statement. Keying
    /// on the wire hash would execute it twice; keying on protocol identity does
    /// not, and the canonical guard agrees it is the same message.
    #[tokio::test]
    async fn reserialized_retry_is_a_replay_not_a_second_execution() {
        let store = InMemoryIdempotencyStore::new();
        let guard = IdempotencyGuard::new(&store);

        let a = typed_from(
            "did:example:consumer",
            format!(
                r#"{{"@context":"{CTX}","@type":"TransferRequestMessage","consumerPid":"urn:uuid:cc","callbackAddress":"https://c/cb"}}"#
            ),
        )
        .await;
        // Same message: keys reordered and the pid written in its `@id` form.
        let b = typed_from(
            "did:example:consumer",
            format!(
                r#"{{"@context":"{CTX}","callbackAddress":"https://c/cb","consumerPid":{{"@id":"urn:uuid:cc"}},"@type":"TransferRequestMessage"}}"#
            ),
        )
        .await;

        assert_ne!(
            a.rdf.parsed.raw.wire_hash(),
            b.rdf.parsed.raw.wire_hash(),
            "the bytes really do differ — a wire-hash key would miss the retry"
        );
        assert_eq!(
            a.rdf.canonical_hash, b.rdf.canonical_hash,
            "but it is the same statement"
        );

        let IdempotencyVerdict::New(key) = guard.check(&a, None).await.unwrap() else {
            panic!("first sighting must be New");
        };
        guard.claim(key, &a, None).await.unwrap();
        assert!(matches!(
            guard.check(&b, None).await.unwrap(),
            IdempotencyVerdict::Replay(_)
        ));
    }

    /// Same pid, materially different message: the peer reused the key. That is a
    /// protocol violation, not a retry, so it must not get the cached ack.
    #[tokio::test]
    async fn same_pid_with_different_content_conflicts() {
        let store = InMemoryIdempotencyStore::new();
        let guard = IdempotencyGuard::new(&store);

        let first = typed_from(
            "did:example:consumer",
            message("urn:uuid:cc", "https://c/cb"),
        )
        .await;
        let IdempotencyVerdict::New(key) = guard.check(&first, None).await.unwrap() else {
            panic!("first sighting must be New");
        };
        guard.claim(key, &first, None).await.unwrap();

        let second = typed_from(
            "did:example:consumer",
            message("urn:uuid:cc", "https://attacker.example/cb"),
        )
        .await;
        assert!(matches!(
            guard.check(&second, None).await.unwrap(),
            IdempotencyVerdict::Conflict(_)
        ));
    }

    /// Pids are unique per peer, not globally: two connectors minting the same
    /// `consumerPid` must not collide into one record.
    #[tokio::test]
    async fn different_peers_reusing_a_pid_do_not_collide() {
        let store = InMemoryIdempotencyStore::new();
        let guard = IdempotencyGuard::new(&store);

        let a = typed_from("did:example:one", message("urn:uuid:cc", "https://one/cb")).await;
        let b = typed_from("did:example:two", message("urn:uuid:cc", "https://two/cb")).await;
        assert_ne!(a.idempotency_key.as_str(), b.idempotency_key.as_str());

        let IdempotencyVerdict::New(key) = guard.check(&a, None).await.unwrap() else {
            panic!("first sighting must be New");
        };
        guard.claim(key, &a, None).await.unwrap();
        assert!(
            matches!(
                guard.check(&b, None).await.unwrap(),
                IdempotencyVerdict::New(_)
            ),
            "the other peer's message must not be mistaken for a replay"
        );
    }
    /// A peer-supplied key must not be usable to reach another peer's record.
    /// Before the header was folded in rather than substituted, both peers keyed
    /// on the literal string and the second got the first's verdict.
    #[tokio::test]
    async fn supplied_key_is_still_scoped_to_the_peer() {
        let store = InMemoryIdempotencyStore::new();
        let guard = IdempotencyGuard::new(&store);

        let a = typed_from_with_key(
            "did:example:one",
            message("urn:uuid:cc", "https://one/cb"),
            Some("shared-key"),
        )
        .await;
        let b = typed_from_with_key(
            "did:example:two",
            message("urn:uuid:cc", "https://two/cb"),
            Some("shared-key"),
        )
        .await;
        assert_ne!(a.idempotency_key.as_str(), b.idempotency_key.as_str());

        let IdempotencyVerdict::New(key) = guard.check(&a, None).await.unwrap() else {
            panic!("first sighting must be New");
        };
        guard.claim(key, &a, None).await.unwrap();
        assert!(
            matches!(
                guard.check(&b, None).await.unwrap(),
                IdempotencyVerdict::New(_)
            ),
            "another peer's supplied key must not reach this record"
        );
    }

    /// The header's one legitimate job: telling two identical messages apart, as a
    /// restart does. Distinct keys → distinct records, so both execute.
    #[tokio::test]
    async fn distinct_supplied_keys_separate_identical_messages() {
        let store = InMemoryIdempotencyStore::new();
        let guard = IdempotencyGuard::new(&store);
        let body = message("urn:uuid:cc", "https://c/cb");

        let first =
            typed_from_with_key("did:example:consumer", body.clone(), Some("attempt-1")).await;
        let IdempotencyVerdict::New(key) = guard.check(&first, None).await.unwrap() else {
            panic!("first sighting must be New");
        };
        guard.claim(key, &first, None).await.unwrap();

        // Same bytes, same pids — but the peer says this is a new attempt.
        let restart =
            typed_from_with_key("did:example:consumer", body.clone(), Some("attempt-2")).await;
        assert!(matches!(
            guard.check(&restart, None).await.unwrap(),
            IdempotencyVerdict::New(_)
        ));

        // ...while a genuine retry reuses the key and replays.
        let retry = typed_from_with_key("did:example:consumer", body, Some("attempt-1")).await;
        assert!(matches!(
            guard.check(&retry, None).await.unwrap(),
            IdempotencyVerdict::Replay(_)
        ));
    }

    /// A `TransferStartMessage` on the state-transition route: same pids every
    /// time, so protocol identity alone cannot tell a retry from a restart.
    async fn start_message() -> TransferDSPContextTyped {
        typed_at(
            "did:example:consumer",
            format!(
                r#"{{"@context":"{CTX}","@type":"TransferStartMessage","consumerPid":"urn:uuid:cc","providerPid":"urn:uuid:pp"}}"#
            ),
            None,
            "/urn:uuid:pp/start",
            TransferDSPMessageType::TransferStartMessage,
        )
        .await
    }

    /// The over-collapse this exists to prevent: a transfer is started, suspended,
    /// then legitimately restarted with a byte-identical message. Keyed on protocol
    /// identity alone the restart looks like a replay and never runs; the process
    /// version tells them apart.
    #[tokio::test]
    async fn a_restart_after_a_suspension_is_not_a_replay() {
        let store = InMemoryIdempotencyStore::new();
        let guard = IdempotencyGuard::new(&store);

        // Start #1 arrives with the process at version 5 and leaves it at 6.
        let start = start_message().await;
        let IdempotencyVerdict::New(key) = guard.check(&start, Some(5)).await.unwrap() else {
            panic!("first start must be New");
        };
        let record = guard.claim(key, &start, Some(5)).await.unwrap();
        guard
            .record_response(record, serde_json::Value::Null, Some(6))
            .await
            .unwrap();

        // A genuine retry: nothing else has touched the process.
        let retry = start_message().await;
        assert!(
            matches!(
                guard.check(&retry, Some(6)).await.unwrap(),
                IdempotencyVerdict::Replay(_)
            ),
            "a retry with the process untouched must still replay"
        );

        // The transfer is suspended (version 7), then restarted with the *same*
        // message. Same pids, same bytes, same canonical hash — but a new transition.
        let restart = start_message().await;
        let IdempotencyVerdict::Superseded { key, previous } =
            guard.check(&restart, Some(7)).await.unwrap()
        else {
            panic!("a restart after an intervening transition must not be a replay");
        };
        assert_eq!(previous.process_version_after, Some(6));

        // And the restart's own retry replays again, against the new version.
        let record = guard.claim(key, &restart, Some(7)).await.unwrap();
        guard
            .record_response(record, serde_json::Value::Null, Some(8))
            .await
            .unwrap();
        let restart_retry = start_message().await;
        assert!(matches!(
            guard.check(&restart_retry, Some(8)).await.unwrap(),
            IdempotencyVerdict::Replay(_)
        ));
    }

    /// A retry that arrives while the first request is still running has no
    /// `process_version_after` yet, and must replay rather than execute twice.
    #[tokio::test]
    async fn a_retry_while_still_in_flight_replays() {
        let store = InMemoryIdempotencyStore::new();
        let guard = IdempotencyGuard::new(&store);

        let start = start_message().await;
        let IdempotencyVerdict::New(key) = guard.check(&start, Some(5)).await.unwrap() else {
            panic!("first start must be New");
        };
        guard.claim(key, &start, Some(5)).await.unwrap();

        let retry = start_message().await;
        assert!(matches!(
            guard.check(&retry, Some(5)).await.unwrap(),
            IdempotencyVerdict::Replay(_)
        ));
    }

    /// Content still outranks the version: a genuinely different message on a
    /// moved-on process is a conflict, not a permitted new transition.
    ///
    /// The difference has to be a term the DSP context defines *for this message
    /// type* — its scoped contexts give `TransferStartMessage` only `consumerPid`,
    /// `dataAddress` and `providerPid`, and anything else is dropped by expansion
    /// and so is invisible to both the guard and the field extractor.
    #[tokio::test]
    async fn different_content_conflicts_even_when_superseded() {
        let store = InMemoryIdempotencyStore::new();
        let guard = IdempotencyGuard::new(&store);

        let start = start_message().await;
        let IdempotencyVerdict::New(key) = guard.check(&start, Some(5)).await.unwrap() else {
            panic!("first start must be New");
        };
        let record = guard.claim(key, &start, Some(5)).await.unwrap();
        guard
            .record_response(record, serde_json::Value::Null, Some(6))
            .await
            .unwrap();

        let tampered = typed_at(
            "did:example:consumer",
            format!(
                r#"{{"@context":"{CTX}","@type":"TransferStartMessage","consumerPid":"urn:uuid:cc","providerPid":"urn:uuid:pp","dataAddress":{{"@type":"DataAddress","endpointType":"https://w3id.org/idsa/v4.1/HTTP","endpoint":"https://attacker.example/exfil"}}}}"#
            ),
            None,
            "/urn:uuid:pp/start",
            TransferDSPMessageType::TransferStartMessage,
        )
        .await;
        assert_ne!(
            start_message().await.rdf.canonical_hash,
            tampered.rdf.canonical_hash,
            "a differing dataAddress really is different content"
        );
        assert!(matches!(
            guard.check(&tampered, Some(7)).await.unwrap(),
            IdempotencyVerdict::Conflict(_)
        ));
    }

    /// A nested router hands the handler a path with the mount prefix stripped, so
    /// `/dsp/2024-1/transfers/request` and `/dsp/current/transfers/request` both
    /// arrive as `/request`. Without the version in the key, one peer's
    /// `consumerPid` would collide across the two versions a connector may serve
    /// side by side (DSP 4.3).
    #[tokio::test]
    async fn the_protocol_version_separates_otherwise_identical_messages() {
        let store = InMemoryIdempotencyStore::new();
        let guard = IdempotencyGuard::new(&store);
        let body = message("urn:uuid:cc", "https://c/cb");

        let v2025 = typed_versioned(
            "did:example:consumer",
            body.clone(),
            None,
            "/request",
            TransferDSPMessageType::TransferRequestMessage,
            ProtocolId::Dsp2025_1,
        )
        .await;
        let v2024 = typed_versioned(
            "did:example:consumer",
            body,
            None,
            "/request",
            TransferDSPMessageType::TransferRequestMessage,
            ProtocolId::Dsp2024,
        )
        .await;

        assert_eq!(
            v2025.rdf.parsed.raw.request_path, v2024.rdf.parsed.raw.request_path,
            "the stripped path really is identical — the version is all that differs"
        );
        assert_ne!(
            v2025.idempotency_key.as_str(),
            v2024.idempotency_key.as_str()
        );

        let IdempotencyVerdict::New(key) = guard.check(&v2025, None).await.unwrap() else {
            panic!("first sighting must be New");
        };
        guard.claim(key, &v2025, None).await.unwrap();
        assert!(
            matches!(
                guard.check(&v2024, None).await.unwrap(),
                IdempotencyVerdict::New(_)
            ),
            "the other version's message must not be mistaken for a replay"
        );
    }
}
