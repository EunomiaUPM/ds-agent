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

//! Keys: protocol identity per peer, supplied keys, and content conflicts.

use super::*;

/// The first message is new and claimed; the same message again is a replay.
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
/// restart does. Distinct keys give distinct records, so both execute.
#[tokio::test]
async fn distinct_supplied_keys_separate_identical_messages() {
    let store = InMemoryIdempotencyStore::new();
    let guard = IdempotencyGuard::new(&store);
    let body = message("urn:uuid:cc", "https://c/cb");

    let first = typed_from_with_key("did:example:consumer", body.clone(), Some("attempt-1")).await;
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
