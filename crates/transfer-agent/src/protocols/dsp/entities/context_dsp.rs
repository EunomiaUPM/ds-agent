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
use common::dsp_common::odrl::OdrlAgreement;
use common::facades::grants_facade::VerifiedPeer;
use common::flow::{IdempotencyKey, IdempotentFactsTrait, IdempotentOutputTrait};
use common::oauth::Owner;
use common::rdf::RdfProtocolTrait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use urn::Urn;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::entities::message_envelope::MessageEnvelope;
use crate::entities::protocol::{ProtocolId, TransferDirection, TransferRole};
use crate::entities::transfer_process::TransferProcess;
use crate::protocols::dsp::entities::ack::TransferProcessAck;
use crate::protocols::dsp::entities::context_common::{
    TransferContextConnectorRole, TransferContextProcessSlot,
};
use crate::protocols::dsp::entities::message_types::TransferDSPMessageType;
use crate::protocols::dsp::entities::protocol_fields::TransferProtocolFields;
use crate::protocols::dsp::entities::rdf_extractor_dsp::DspTransfer;
use crate::protocols::dsp::entities::state::TransferDSPState;
use crate::protocols::dsp::facades::dataplane_facade::DataplaneHookContext;

/// The request as it arrived: who sent it, where, and the body bytes.
#[derive(Debug)]
pub struct TransferDspRaw {
    /// The peer the auth layer resolved.
    pub peer: VerifiedPeer,
    pub request_path: String,
    /// The pid in the route, on the routes that carry one.
    pub path_id: Option<String>,
    /// The peer's `Idempotency-Key` header: one ingredient of the key, not the key.
    pub supplied_idempotency_key: Option<String>,
    pub body: Bytes,
}

impl TransferDspRaw {
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

/// The body read as JSON, with the protocol version and the message type of its route.
#[derive(Debug)]
pub struct TransferDspParsed {
    pub raw: TransferDspRaw,
    pub version: ProtocolId,
    /// The message the route handles; the body's own type is read in the typed stage.
    pub route: TransferDSPMessageType,
    pub json: Value,
}

impl TransferDspParsed {
    pub fn from_raw(
        raw: TransferDspRaw,
        version: &ProtocolId,
        route: &TransferDSPMessageType,
    ) -> Outcome<Self> {
        let json = serde_json::from_slice(&raw.body).map_err(|e| {
            Errors::format(BadFormat::Received, format!("body is not JSON: {e}"), None)
        })?;
        Ok(Self {
            raw,
            version: version.clone(),
            route: route.clone(),
            json,
        })
    }
}

/// The message expanded to RDF, with its canonical n-quads and their hash.
#[derive(Debug)]
pub struct TransferDspRdf {
    pub parsed: TransferDspParsed,
    /// What extraction reads: invariant under the aliasing a peer is free to choose.
    pub expanded: Value,
    pub canonical_n_quads: String,
    /// A semantic identity of the message, not an attestation of its bytes.
    pub canonical_hash: [u8; 32],
}

impl TransferDspRdf {
    pub async fn from_parsed(parsed: TransferDspParsed) -> Outcome<Self> {
        let expansion = DspTransfer::engine().expand(&parsed.json).await?;
        Ok(Self {
            canonical_hash: expansion.canonical_hash(),
            canonical_n_quads: expansion.canonical_n_quads,
            expanded: expansion.expanded,
            parsed,
        })
    }
}

/// The DSP 9.2 fields read off the expansion, the type the body declares and the
/// idempotency key.
#[derive(Debug)]
pub struct TransferDspTyped {
    pub rdf: TransferDspRdf,
    /// What the body declares; agreeing with the route is a shape rule.
    pub message: TransferDSPMessageType,
    pub fields: TransferProtocolFields,
    pub idempotency_key: IdempotencyKey,
}

impl TransferDspTyped {
    pub fn from_rdf(rdf: TransferDspRdf) -> Outcome<Self> {
        let (message, fields) = DspTransfer::read_expanded(&rdf.expanded)?;
        let idempotency_key = Self::derive_key(&rdf.parsed, &message, &fields);
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

    pub fn route(&self) -> &TransferDSPMessageType {
        &self.rdf.parsed.route
    }

    pub fn path_id(&self) -> Option<&str> {
        self.rdf.parsed.raw.path_id.as_deref()
    }

    /// How the message is kept in the process history.
    pub fn envelope(&self) -> MessageEnvelope {
        MessageEnvelope::new(
            self.rdf.parsed.json.clone(),
            Some((self.rdf.canonical_n_quads.clone(), self.rdf.canonical_hash)),
        )
    }

    /// Participant, route, version, type, pids and the peer's header, length-prefixed so no
    /// pid can imitate another tuple; pids are unique per peer, so the participant is in.
    fn derive_key(
        parsed: &TransferDspParsed,
        message: &TransferDSPMessageType,
        fields: &TransferProtocolFields,
    ) -> IdempotencyKey {
        let raw = &parsed.raw;
        let mut hasher = Sha256::new();
        let mut field = |bytes: &[u8]| {
            hasher.update((bytes.len() as u64).to_be_bytes());
            hasher.update(bytes);
        };
        field(raw.peer.participant_id.as_bytes());
        field(raw.request_path.as_bytes());
        // A nested router strips the mount prefix, so the path alone cannot tell versions apart.
        field(parsed.version.to_string().as_bytes());
        field(message.to_string().as_bytes());
        field(fields.consumer_pid.as_deref().unwrap_or_default().as_bytes());
        field(fields.provider_pid.as_deref().unwrap_or_default().as_bytes());
        field(raw.supplied_idempotency_key.as_deref().unwrap_or_default().as_bytes());
        IdempotencyKey::new(format!("dsp-{}", hex::encode(hasher.finalize())))
    }
}

/// The typed message with the process, agreement, role and connector it refers to.
#[derive(Debug)]
pub struct TransferDspDomain {
    pub typed: TransferDspTyped,
    pub process: TransferContextProcessSlot,
    pub agreement: OdrlAgreement,
    pub role: TransferRole,
    pub transfer_direction: TransferDirection,
    pub connector_instance: TransferContextConnectorRole,
    pub is_restart: bool,
}

impl TransferDspDomain {
    /// Owner of the existing process, or of a new one a peer opens: seen as its grant says.
    pub fn owner(&self) -> Owner {
        match &self.process {
            TransferContextProcessSlot::Existing(p) => p.owner().clone(),
            TransferContextProcessSlot::New { .. } => self.typed.peer().owner(),
        }
    }

    /// What the dataplane hooks need from this message and the process it refers to.
    pub fn dataplane_hooks(&self) -> Outcome<DataplaneHookContext> {
        Ok(DataplaneHookContext {
            process_id: self.process_urn()?,
            owner: self.owner(),
            role: self.role,
            transfer_direction: self.transfer_direction,
            connector_instance: self.connector_instance.clone(),
            data_address: self.typed.fields.data_address.clone(),
            resolved_data_address: None,
            is_restart: self.is_restart,
        })
    }

    /// Id of the process; a new one has none until the request handler stores it.
    pub fn process_urn(&self) -> Outcome<Urn> {
        match &self.process {
            TransferContextProcessSlot::Existing(p) => Ok(p.id().as_urn().clone()),
            TransferContextProcessSlot::New { .. } => Err(Errors::crazy(
                "the process is not stored yet",
                None,
            )),
        }
    }
}

impl IdempotentFactsTrait for TransferDspDomain {
    fn idempotency_key(&self) -> &IdempotencyKey {
        &self.typed.idempotency_key
    }

    fn canonical_hash(&self) -> &[u8; 32] {
        &self.typed.rdf.canonical_hash
    }

    fn process_version(&self) -> Option<u64> {
        match &self.process {
            TransferContextProcessSlot::Existing(p) => Some(p.version()),
            TransferContextProcessSlot::New { .. } => None,
        }
    }
}

/// What applying an inbound message produced: the ACK for the peer, the process version and
/// key idempotency replays it with, and the agreement and role the TCK continues from.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferDspOutcome {
    pub ack: TransferProcessAck,
    /// Our id of the process the message left.
    #[serde(default)]
    pub process_id: String,
    /// A request opened the process (201); any other message moved it (200).
    pub created: bool,
    pub process_version: u64,
    pub idempotency_key: IdempotencyKey,
    pub agreement_id: String,
    pub role: TransferRole,
    /// The request asked the provider to start the transfer unasked.
    #[serde(default)]
    pub auto_start: bool,
}

impl TransferDspOutcome {
    /// The outcome of applying `domain`'s message, leaving `process` in `state`.
    pub fn applied(
        domain: &TransferDspDomain,
        process: &TransferProcess,
        state: TransferDSPState,
        created: bool,
    ) -> Self {
        let correlation = process.correlation();
        Self {
            ack: TransferProcessAck::new(
                correlation.consumer_pid.clone().unwrap_or_default(),
                correlation.provider_pid.clone().unwrap_or_default(),
                state,
            ),
            process_id: process.id().as_urn().to_string(),
            created,
            process_version: process.version(),
            idempotency_key: domain.typed.idempotency_key.clone(),
            agreement_id: domain.agreement.id.to_string(),
            role: domain.role,
            auto_start: domain.typed.fields.auto_start,
        }
    }
}

impl IdempotentOutputTrait for TransferDspOutcome {
    fn idempotency_key(&self) -> &IdempotencyKey {
        &self.idempotency_key
    }

    fn process_version(&self) -> Option<u64> {
        Some(self.process_version)
    }
}
