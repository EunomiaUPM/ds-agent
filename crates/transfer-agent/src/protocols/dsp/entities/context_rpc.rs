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

//! An outbound transfer message stage by stage: the eunomia RPC body as sent (raw), expanded
//! (RDF) and read (typed), then everything resolved to send it (domain), and what it produced.

use std::str::FromStr;
use axum::extract::Request;

use common::dsp_common::data_address::DataAddress;
use common::dsp_common::odrl::OdrlAgreement;
use common::oauth::Owner;
use common::rdf::{RdfMessage, RdfProtocolTrait};
use serde::Serialize;
use serde_json::{json, Value};
use urn::Urn;
use ymir::data::entities::shared::participant::Model as Mates;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::entities::protocol::{TransferDirection, TransferRole};
use crate::entities::transfer_process::TransferProcess;
use crate::protocols::dsp::entities::context_common::TransferContextConnectorRole;
use crate::protocols::dsp::entities::data_address::DataAddressDto;
use crate::protocols::dsp::entities::message_types::TransferDSPMessageType;
use crate::protocols::dsp::entities::rdf_extractor_dsp::DspTransfer;
use crate::protocols::dsp::entities::rdf_extractor_rpc::EunomiaTransfer;
use crate::protocols::dsp::entities::state::TransferDSPState;
use crate::protocols::dsp::facades::dataplane_facade::DataplaneHookContext;

/// RPC body for any outbound transfer message; which fields matter depends on the message.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferRpcBody {
    /// The process a continuation moves, by its id here; peers' pids never reach the caller.
    #[serde(rename = "@id")]
    pub id: Option<String>,
    /// The provider's participant id, for a request.
    pub associated_agent_peer: Option<String>,
    pub agreement_id: Option<String>,
    pub format: Option<String>,
    /// The sink a push transfer delivers to, for a request.
    pub data_address: Option<DataAddressDto>,
    /// The provider's DSP base, for a request.
    pub provider_address: Option<String>,
    /// Our DSP base as the provider will call it back; defaults to this agent's own.
    pub callback_address: Option<String>,
    pub code: Option<String>,
    pub reason: Option<Vec<String>>,
    /// Asks the provider to start the transfer as soon as it accepts the request.
    pub auto_start: Option<bool>,
}

impl TransferRpcBody {
    /// The body as a eunomia JSON-LD message of the RPC that sends `message`.
    pub fn to_json_ld(&self, message: &TransferDSPMessageType) -> Value {
        let mut json = json!({
            "@context": EunomiaTransfer::context(),
            "@type": EunomiaTransfer::compact_type(message),
        });
        if let (Value::Object(object), Ok(Value::Object(members))) =
            (&mut json, serde_json::to_value(self))
        {
            object.extend(members.into_iter().filter(|(_, v)| !v.is_null()));
        }
        json
    }
}

/// The RPC body as it arrived, with the message its route sends.
#[derive(Debug, Clone)]
pub struct TransferRpcRaw {
    pub route: TransferDSPMessageType,
    pub json: Value,
}

impl TransferRpcRaw {
    /// Cap on RPC bodies, as on the DSP side.
    pub const MAX_BODY_BYTES: usize = 1024 * 1024;

    pub fn new(route: &TransferDSPMessageType, json: Value) -> Self {
        Self {
            route: route.clone(),
            json,
        }
    }

    /// The request's body, which must be JSON.
    pub async fn from_request(route: &TransferDSPMessageType, request: Request) -> Outcome<Self> {
        let body = axum::body::to_bytes(request.into_body(), Self::MAX_BODY_BYTES)
            .await
            .map_err(|_| {
                Errors::format(BadFormat::Received, "failed to read request body", None)
            })?;
        let json = serde_json::from_slice(&body).map_err(|e| {
            Errors::format(BadFormat::Received, format!("body is not JSON: {e}"), None)
        })?;
        Ok(Self::new(route, json))
    }
}

/// The body expanded through the eunomia context.
#[derive(Debug, Clone)]
pub struct TransferRpcRdf {
    pub raw: TransferRpcRaw,
    pub expanded: Value,
    pub canonical_n_quads: String,
    pub canonical_hash: [u8; 32],
}

impl TransferRpcRdf {
    pub async fn from_raw(raw: TransferRpcRaw) -> Outcome<Self> {
        let expansion = EunomiaTransfer::engine().expand(&raw.json).await?;
        Ok(Self {
            canonical_hash: expansion.canonical_hash(),
            canonical_n_quads: expansion.canonical_n_quads,
            expanded: expansion.expanded,
            raw,
        })
    }
}

/// The body's fields and the RPC type it declares; agreeing with the route is a shape rule.
#[derive(Debug, Clone)]
pub struct TransferRpcTyped {
    pub rdf: TransferRpcRdf,
    pub message: TransferDSPMessageType,
    pub body: TransferRpcBody,
}

impl TransferRpcTyped {
    pub fn from_rdf(rdf: TransferRpcRdf) -> Outcome<Self> {
        let (message, body) = EunomiaTransfer::read_expanded(&rdf.expanded)?;
        Ok(Self { rdf, message, body })
    }

    /// Runs the stages on a body built in process, such as the TCK's.
    pub async fn from_body(route: &TransferDSPMessageType, body: &TransferRpcBody) -> Outcome<Self> {
        let raw = TransferRpcRaw::new(route, body.to_json_ld(route));
        Self::from_rdf(TransferRpcRdf::from_raw(raw).await?)
    }

    pub fn route(&self) -> &TransferDSPMessageType {
        &self.rdf.raw.route
    }

    /// The id of the existing process the body moves.
    pub fn process_id(&self) -> Option<&str> {
        self.body.id.as_deref()
    }
}

/// The typed body plus everything resolved to send it: process, role, peer, agreement and the
/// message.
pub struct TransferRpcDomain {
    pub typed: TransferRpcTyped,
    pub process: Option<TransferProcess>,
    pub process_id: Urn,
    pub role: TransferRole,
    pub transfer_direction: TransferDirection,
    pub owner: Owner,
    pub agreement: OdrlAgreement,
    pub peer_id: String,
    /// The peer's participant record, when this connector knows it.
    pub peer_mate: Option<Mates>,
    pub target_url: String,
    pub outgoing: RdfMessage<DspTransfer>,
    pub is_restart: bool,
}

impl TransferRpcDomain {
    pub fn current_state(&self) -> Option<TransferDSPState> {
        self.process
            .as_ref()
            .and_then(|p| TransferDSPState::from_str(p.state().0.as_str()).ok())
    }

    /// What the dataplane hooks need; `data_address` is the one this side hands the peer.
    pub fn hooks(&self, data_address: Option<DataAddress>) -> DataplaneHookContext {
        DataplaneHookContext {
            process_id: self.process_id.clone(),
            owner: self.owner.clone(),
            role: self.role,
            transfer_direction: self.transfer_direction,
            connector_instance: TransferContextConnectorRole::ConsumerNotHavingConnector,
            resolved_data_address: data_address.clone(),
            data_address,
            is_restart: self.is_restart,
        }
    }
}

/// What sending produced: what was asked, the peer's answer, the process as it stands, and the
/// agreement and role the TCK continues from.
#[derive(Debug)]
pub struct TransferRpcOutcome {
    pub request: Value,
    pub response: Value,
    pub process: TransferProcess,
    pub agreement_id: String,
    pub role: TransferRole,
}

impl TransferRpcOutcome {
    /// What sending `domain`'s message produced, given the peer's `reply` and the process it left.
    pub fn sent(domain: TransferRpcDomain, reply: &Value, process: TransferProcess) -> Self {
        Self {
            request: domain.typed.rdf.raw.json,
            response: reply["raw"].clone(),
            agreement_id: process
                .correlation()
                .agreement_id
                .as_ref()
                .map(Urn::to_string)
                .unwrap_or_default(),
            role: domain.role,
            process,
        }
    }
}
