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

//! Process versions: restarts after a suspension, in-flight retries and superseded records.

use super::*;

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
