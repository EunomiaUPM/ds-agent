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

//! IdempotencyGuard over an in-memory store, split into how messages are keyed and how a
//! process version tells restarts from retries.

mod keys;
mod versions;

use transfer_agent::entities::protocol::ProtocolId;
use transfer_agent::protocols::dsp::entities::auth::TransferDSPAuthn;
use transfer_agent::protocols::dsp::entities::context_common::TransferContextRaw;
use transfer_agent::protocols::dsp::entities::context_dsp::TransferDSPContextTyped;
use transfer_agent::protocols::dsp::entities::message_types::TransferDSPMessageType;

use axum::extract::Request;
use transfer_agent::protocols::dsp::entities::context_dsp::TransferDSPContextParsed;
use transfer_agent::protocols::dsp::entities::idempotency::*;
use common::facades::grants_facade::VerifiedPeer;
use common::oauth::RolePath;

const CTX: &str = "https://w3id.org/dspace/2025/1/context.jsonld";

fn mate(participant_id: &str) -> VerifiedPeer {
    VerifiedPeer {
        participant_id: participant_id.into(),
        role: RolePath::root(),
        visibility: common::oauth::Visibility::Public,
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
    let rdf =
        transfer_agent::protocols::dsp::entities::context_dsp::TransferDSPContextRdf::from_parsed(
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
