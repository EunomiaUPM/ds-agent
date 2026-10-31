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

//! The inbound DSP message stage by stage (raw, parsed, RDF, typed, domain), each consuming
//! the previous one so no stage can be skipped, and what applying it produced.

use std::collections::HashMap;

use axum::extract::{FromRequestParts, Path, Request};
use bytes::Bytes;
use common::facades::catalog::CatalogOffer;
use common::facades::grants_facade::VerifiedPeer;
use common::flow::{IdempotencyKey, IdempotentFactsTrait, IdempotentOutputTrait};
use common::rdf::RdfProtocolTrait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use ymir::errors::{BadFormat, Errors, Outcome};
use ymir::oauth::Owner;

use crate::protocols::dsp::entities::ack::NegotiationAck;
use crate::protocols::dsp::entities::context_common::NegotiationProcessSlot;
use crate::protocols::dsp::entities::message_types::NegotiationMessageType;
use crate::protocols::dsp::entities::process::NegotiationProcess;
use crate::protocols::dsp::entities::protocol_fields::NegotiationProtocolFields;
use crate::protocols::dsp::entities::rdf_extractor_dsp::DspNegotiation;
use crate::protocols::dsp::entities::role::NegotiationRole;

/// The request as it arrived: who sent it, where, and the body bytes.
#[derive(Debug)]
pub struct NegotiationDspRaw {
    /// The peer the auth layer resolved.
    pub peer: VerifiedPeer,
    pub request_path: String,
    /// The pid in the route, on the routes that carry one.
    pub path_id: Option<String>,
    /// The peer's `Idempotency-Key` header: one ingredient of the key, not the key.
    pub supplied_idempotency_key: Option<String>,
    pub body: Bytes,
}

impl NegotiationDspRaw {
    /// Cap on inbound bodies: control-plane messages are tiny, this only bounds a hostile peer.
    const MAX_BODY_BYTES: usize = 1024 * 1024;

    /// The request `peer` sent, as the auth layer resolved it.
    pub async fn from_request(peer: VerifiedPeer, request: Request) -> Outcome<Self> {
        let (mut parts, body) = request.into_parts();
        let body = axum::body::to_bytes(body, Self::MAX_BODY_BYTES)
            .await
            .map_err(|_| {
                Errors::format(BadFormat::Received, "failed to read request body", None)
            })?;
        let supplied_idempotency_key = parts
            .headers
            .get("idempotency-key")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        let request_path = parts.uri.path().to_string();
        let path_id = Path::<HashMap<String, String>>::from_request_parts(&mut parts, &())
            .await
            .ok()
            .and_then(|Path(params)| params.get("id").cloned());
        Ok(Self {
            peer,
            request_path,
            path_id,
            supplied_idempotency_key,
            body,
        })
    }
}

/// The body read as JSON, with the message type of its route.
#[derive(Debug)]
pub struct NegotiationDspParsed {
    pub raw: NegotiationDspRaw,
    /// The message the route handles; the body's own type is read in the typed stage.
    pub route: NegotiationMessageType,
    pub json: Value,
}

impl NegotiationDspParsed {
    pub fn from_raw(raw: NegotiationDspRaw, route: NegotiationMessageType) -> Outcome<Self> {
        let json = serde_json::from_slice(&raw.body).map_err(|e| {
            Errors::format(BadFormat::Received, format!("body is not JSON: {e}"), None)
        })?;
        Ok(Self { raw, route, json })
    }
}

/// The message expanded to RDF, with its canonical n-quads and their hash.
#[derive(Debug)]
pub struct NegotiationDspRdf {
    pub parsed: NegotiationDspParsed,
    /// What extraction reads: invariant under the aliasing a peer is free to choose.
    pub expanded: Value,
    pub canonical_n_quads: String,
    /// A semantic identity of the message, not an attestation of its bytes.
    pub canonical_hash: [u8; 32],
}

impl NegotiationDspRdf {
    pub async fn from_parsed(parsed: NegotiationDspParsed) -> Outcome<Self> {
        let expansion = DspNegotiation::engine().expand(&parsed.json).await?;
        Ok(Self {
            canonical_hash: expansion.canonical_hash(),
            canonical_n_quads: expansion.canonical_n_quads,
            expanded: expansion.expanded,
            parsed,
        })
    }
}

/// The DSP fields read off the expansion, the type the body declares and the idempotency key.
#[derive(Debug)]
pub struct NegotiationDspTyped {
    pub rdf: NegotiationDspRdf,
    /// What the body declares; agreeing with the route is a shape rule.
    pub message: NegotiationMessageType,
    pub fields: NegotiationProtocolFields,
    pub idempotency_key: IdempotencyKey,
}

impl NegotiationDspTyped {
    /// DSP version the routes serve; a nested router strips it from the path.
    const VERSION: &'static str = "2025-1";

    pub fn from_rdf(rdf: NegotiationDspRdf) -> Outcome<Self> {
        let (message, fields) = DspNegotiation::read_expanded(&rdf.expanded)?;
        let idempotency_key = Self::derive_key(&rdf.parsed, message, &fields);
        Ok(Self {
            rdf,
            message,
            fields,
            idempotency_key,
        })
    }

    pub fn peer(&self) -> &VerifiedPeer {
        &self.rdf.parsed.raw.peer
    }

    pub fn route(&self) -> NegotiationMessageType {
        self.rdf.parsed.route
    }

    pub fn path_id(&self) -> Option<&str> {
        self.rdf.parsed.raw.path_id.as_deref()
    }

    /// The message as the peer wrote it, kept in the process history.
    pub fn json(&self) -> &Value {
        &self.rdf.parsed.json
    }

    /// The offer as sent, rules included; its identity is in `fields.offer`.
    pub fn offer_content(&self) -> Option<&Value> {
        self.json().get("offer")
    }

    /// The agreement as sent, rules included; its identity is in `fields.agreement`.
    pub fn agreement_content(&self) -> Option<&Value> {
        self.json().get("agreement")
    }

    /// Participant, route, version, type, pids and the peer's header, length-prefixed so no
    /// pid can imitate another tuple; pids are unique per peer, so the participant is in.
    fn derive_key(
        parsed: &NegotiationDspParsed,
        message: NegotiationMessageType,
        fields: &NegotiationProtocolFields,
    ) -> IdempotencyKey {
        let raw = &parsed.raw;
        let mut hasher = Sha256::new();
        let mut field = |bytes: &[u8]| {
            hasher.update((bytes.len() as u64).to_be_bytes());
            hasher.update(bytes);
        };
        field(raw.peer.participant_id.as_bytes());
        field(raw.request_path.as_bytes());
        field(Self::VERSION.as_bytes());
        field(message.to_string().as_bytes());
        field(
            fields
                .consumer_pid
                .as_deref()
                .unwrap_or_default()
                .as_bytes(),
        );
        field(
            fields
                .provider_pid
                .as_deref()
                .unwrap_or_default()
                .as_bytes(),
        );
        field(
            raw.supplied_idempotency_key
                .as_deref()
                .unwrap_or_default()
                .as_bytes(),
        );
        IdempotencyKey::new(format!("dsp-{}", hex::encode(hasher.finalize())))
    }
}

/// The typed message with the process, our role and the offers it refers to.
#[derive(Debug)]
pub struct NegotiationDspDomain {
    pub typed: NegotiationDspTyped,
    pub process: NegotiationProcessSlot,
    pub role: NegotiationRole,
    /// The offer published under the request's offer id, when we provide and it exists.
    pub catalog_offer: Option<CatalogOffer>,
    /// The target of the offer that opened the process; the TCK names its tests by it.
    pub opening_target: Option<String>,
}

impl NegotiationDspDomain {
    /// Owner of the existing process, or of a new one a peer opens: seen as its grant says.
    pub fn owner(&self) -> Owner {
        match &self.process {
            NegotiationProcessSlot::Existing(p) => p.owner(),
            NegotiationProcessSlot::New { .. } => self.typed.peer().owner(),
        }
    }

    pub fn existing(&self) -> Outcome<&NegotiationProcess> {
        self.process
            .existing()
            .ok_or_else(|| Errors::crazy("the message needs a stored process", None))
    }
}

impl IdempotentFactsTrait for NegotiationDspDomain {
    fn idempotency_key(&self) -> &IdempotencyKey {
        &self.typed.idempotency_key
    }

    fn canonical_hash(&self) -> &[u8; 32] {
        &self.typed.rdf.canonical_hash
    }

    fn process_version(&self) -> Option<u64> {
        self.process.existing().map(NegotiationProcess::version)
    }
}

/// What applying an inbound message produced: the ACK, the version and key idempotency
/// replays it with, and the role and offer the TCK continues from.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NegotiationDspOutcome {
    pub ack: NegotiationAck,
    /// A request or offer opened the process (201); any other message moved it (200).
    pub created: bool,
    pub process_version: u64,
    pub idempotency_key: IdempotencyKey,
    pub role: NegotiationRole,
    pub opening_target: Option<String>,
}

impl NegotiationDspOutcome {
    /// The outcome of applying `domain`'s message, leaving `process` as it is now.
    pub fn applied(
        domain: &NegotiationDspDomain,
        process: &NegotiationProcess,
        created: bool,
    ) -> Self {
        Self {
            ack: NegotiationAck::new(
                process.consumer_pid.clone().unwrap_or_default(),
                process.provider_pid.clone().unwrap_or_default(),
                process.state,
            ),
            created,
            process_version: process.version(),
            idempotency_key: domain.typed.idempotency_key.clone(),
            role: domain.role,
            opening_target: domain.opening_target.clone(),
        }
    }
}

impl IdempotentOutputTrait for NegotiationDspOutcome {
    fn idempotency_key(&self) -> &IdempotencyKey {
        &self.idempotency_key
    }

    fn process_version(&self) -> Option<u64> {
        Some(self.process_version)
    }
}
