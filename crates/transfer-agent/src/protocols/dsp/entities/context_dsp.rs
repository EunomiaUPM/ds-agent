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

//! The inbound DSP context, stage by stage: raw, parsed, rdf, typed and domain.
//! Each stage consumes the previous one, so the order cannot be skipped.

use crate::entities::ids::IdempotencyKey;
use crate::entities::protocol::{ProtocolId, TransferDirection, TransferRole};
use crate::protocols::dsp::entities::auth::TransferDSPAuthn;
use crate::protocols::dsp::entities::context_common::{BuildAuthn, TransferContextRaw};
use crate::protocols::dsp::entities::context_common::{
    TransferContextConnectorRole, TransferContextProcessSlot,
};
use crate::protocols::dsp::entities::message_types::TransferDSPMessageType;
use crate::protocols::dsp::entities::protocol_fields::TransferProtocolFields;
use crate::protocols::dsp::entities::rdf_extractor_dsp::{DspTransfer, ExtractProtocolFields};
use common::dsp_common::data_address::DataAddress;
use common::dsp_common::odrl::OdrlAgreement;
use common::dsp_common::rdf::DspProfile;
use common::rdf::ExpandedDoc;
use http::request::Parts;
use sha2::{Digest, Sha256};
use std::str::FromStr;
use urn::Urn;
use ymir::data::entities::shared::participant::Model as Mates;
use ymir::errors::{BadFormat, Errors, Outcome};

impl BuildAuthn for TransferDSPAuthn {
    fn from_request_parts(parts: &Parts) -> Outcome<Self> {
        let associated_participant = parts.extensions.get::<Mates>().cloned().ok_or_else(|| {
            Errors::crazy(
                "auth middleware did not resolve participant (Mates missing)",
                None,
            )
        })?;
        let raw = Self::header(&parts.headers, "authorization").unwrap_or_default();
        let (token_type, token_content) = raw
            .split_once(' ')
            .map(|(t, c)| (t.to_string(), c.trim().to_string()))
            .unwrap_or_else(|| (String::new(), raw.clone()));
        Ok(TransferDSPAuthn {
            raw,
            token_type,
            token_content,
            associated_participant,
        })
    }
}

/// Inbound message read as JSON, with the protocol version and message type of its route.
#[derive(Debug)]
pub struct TransferDSPContextParsed {
    pub raw: TransferContextRaw<TransferDSPAuthn>,
    pub dsp_version: ProtocolId,
    pub dsp_message_type: TransferDSPMessageType,
    pub json_value: serde_json::Value,
}

impl TransferDSPContextParsed {
    /// Wrap the raw context with what the route already settled: protocol version
    /// and the message type the endpoint handles.
    pub fn from_raw(
        raw: TransferContextRaw<TransferDSPAuthn>,
        dsp_version: &ProtocolId,
        dsp_message_type: &TransferDSPMessageType,
        json_value: serde_json::Value,
    ) -> Outcome<Self> {
        Ok(Self {
            raw,
            dsp_version: dsp_version.clone(),
            dsp_message_type: dsp_message_type.clone(),
            json_value,
        })
    }
}

/// Inbound message expanded to RDF, with its canonical n-quads and hash.
#[derive(Debug)]
pub struct TransferDSPContextRdf {
    pub parsed: TransferDSPContextParsed,
    /// The form extraction and payload validation read: invariant under the
    /// aliasing a peer is free to choose.
    pub expanded: serde_json::Value,
    /// URDNA2015 / RDFC-1.0 canonical n-quads of the expanded message.
    pub canonical_n_quads: String,
    pub canonical_hash: [u8; 32],
}

impl TransferDSPContextRdf {
    /// Expand once, keeping both products. `canonical_hash` is a semantic identity,
    /// not an attestation of the bytes — for that see [`TransferContextRaw::wire_hash`].
    pub async fn from_parsed(parsed: TransferDSPContextParsed) -> Outcome<Self> {
        let expansion = DspProfile::shared().expand(&parsed.json_value).await?;
        let canonical_hash = Sha256::digest(expansion.canonical_n_quads.as_bytes()).into();
        Ok(Self {
            parsed,
            expanded: expansion.expanded,
            canonical_n_quads: expansion.canonical_n_quads,
            canonical_hash,
        })
    }
}

/// Inbound message with its DSP fields extracted and its idempotency key derived.
#[derive(Debug)]
pub struct TransferDSPContextTyped {
    pub rdf: TransferDSPContextRdf,
    /// What the **body** declares. The route's own type stays in
    /// `rdf.parsed.dsp_message_type`, for the manager to check against this.
    pub message: TransferDSPMessageType,
    /// The DSP 9.2 fields, as extracted. Passed on to the domain by itself.
    pub fields: TransferProtocolFields,
    /// Always present: derived from protocol identity, with the peer's header
    /// folded in when it sent one. See [`IdempotencyKey::derive`].
    pub idempotency_key: IdempotencyKey,
}

impl TransferDSPContextTyped {
    /// Build the typed context from the RDF stage. Extraction only: the message
    /// type comes from the body, and agreeing with the route is the manager's call.
    pub fn from_rdf(rdf: TransferDSPContextRdf) -> Outcome<Self> {
        let (message, fields) = {
            let doc = ExpandedDoc::new(&rdf.expanded).ok_or_else(|| {
                Errors::format(
                    BadFormat::Received,
                    "expanded JSON-LD is not an array of node objects",
                    None,
                )
            })?;
            let (message, node) = DspTransfer::root_message(&doc)?;
            (message, DspTransfer::extract(&node)?)
        };

        let idempotency_key = IdempotencyKey::derive(
            &rdf.parsed.raw,
            &rdf.parsed.dsp_version,
            &message,
            fields.consumer_pid.as_deref(),
            fields.provider_pid.as_deref(),
        );

        Ok(TransferDSPContextTyped {
            rdf,
            message,
            fields,
            idempotency_key,
        })
    }
}

/// Inbound message with the process, agreement, role and connector it refers to.
#[derive(Debug)]
pub struct TransferDSPContextDomain {
    pub typed: TransferDSPContextTyped,
    pub process: TransferContextProcessSlot,
    pub agreement: OdrlAgreement,
    pub role: TransferRole,
    pub transfer_direction: TransferDirection,
    pub connector_instance: TransferContextConnectorRole,
    pub is_restart: bool,
    pub is_idempotent_replay: bool,
    pub resolved_data_address: Option<DataAddress>,
}

impl TransferDSPContextDomain {
    /// Wrap the typed context with the domain facts resolved by the
    /// `domain_loader` stage: the process slot (loaded or newly minted),
    /// agreement, role, connector, and the restart / idempotent-replay flags.
    pub fn from_typed(
        typed: TransferDSPContextTyped,
        process: TransferContextProcessSlot,
        agreement: OdrlAgreement,
        role: TransferRole,
        transfer_direction: TransferDirection,
        connector_instance: TransferContextConnectorRole,
        is_restart: bool,
        is_idempotent_replay: bool,
    ) -> Outcome<Self> {
        Ok(Self {
            typed,
            process,
            agreement,
            role,
            transfer_direction,
            connector_instance,
            is_restart,
            is_idempotent_replay,
            resolved_data_address: None,
        })
    }

    /// Tenant of the existing process, or of the peer for a new one.
    pub fn tenant_id(&self) -> &str {
        match &self.process {
            TransferContextProcessSlot::Existing(p) => p.tenant_id(),
            TransferContextProcessSlot::New { .. } => {
                &self
                    .typed
                    .rdf
                    .parsed
                    .raw
                    .authn
                    .associated_participant
                    .tenant_id
            }
        }
    }

    /// Id of the existing process, or the consumer pid of a new one; `location` names the
    /// caller in errors.
    pub fn process_urn(&self, location: &str) -> Outcome<Urn> {
        match &self.process {
            TransferContextProcessSlot::Existing(p) => Ok(p.id().as_urn().clone()),
            TransferContextProcessSlot::New { consumer_pid } => Urn::from_str(consumer_pid)
                .map_err(|_| {
                    Errors::crazy(format!("invalid consumer_pid urn for {location}"), None)
                }),
        }
    }
}
