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

//! An outbound negotiation message stage by stage: the eunomia RPC body as sent (raw), expanded
//! (RDF) and read (typed), then everything resolved to send it (domain), and what it produced.

use axum::extract::Request;
use common::oauth::Owner;
use common::rdf::{RdfMessage, RdfProtocolTrait};
use serde::Serialize;
use serde_json::{json, Value};
use ymir::data::entities::shared::participant::Model as Mates;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::protocols::dsp::entities::message_types::NegotiationRpcMessage;
use crate::protocols::dsp::entities::process::NegotiationProcess;
use crate::protocols::dsp::entities::rdf_extractor_dsp::DspNegotiation;
use crate::protocols::dsp::entities::rdf_extractor_rpc::EunomiaNegotiation;
use crate::protocols::dsp::entities::role::NegotiationRole;
use crate::protocols::dsp::entities::state::NegotiationState;

/// RPC body for any outbound negotiation message; which fields matter depends on the message.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NegotiationRpcBody {
    /// The process a continuation moves, by its id here; peers' pids never reach the caller.
    #[serde(rename = "@id")]
    pub id: Option<String>,
    /// The peer's participant id, for an opening message.
    pub associated_agent_peer: Option<String>,
    /// The provider's DSP base, for an opening request.
    pub provider_address: Option<String>,
    /// The consumer's DSP base, for an opening offer.
    pub consumer_address: Option<String>,
    /// Our DSP base as the peer will call it back; defaults to this agent's own.
    pub callback_address: Option<String>,
    /// The ODRL offer, as DSP sends it, for a request or offer.
    pub offer: Option<Value>,
    pub code: Option<String>,
    pub reason: Option<Vec<String>>,
}

impl NegotiationRpcBody {
    /// The body as a eunomia JSON-LD message of the RPC `message`.
    pub fn to_json_ld(&self, message: NegotiationRpcMessage) -> Value {
        let mut json = json!({
            "@context": EunomiaNegotiation::context(),
            "@type": EunomiaNegotiation::compact_type(&message),
        });
        if let (Value::Object(object), Ok(Value::Object(members))) =
            (&mut json, serde_json::to_value(self))
        {
            object.extend(members.into_iter().filter(|(_, v)| !v.is_null()));
        }
        json
    }

}

/// The RPC body as it arrived, with the RPC its route serves.
#[derive(Debug, Clone)]
pub struct NegotiationRpcRaw {
    pub route: NegotiationRpcMessage,
    pub json: Value,
}

impl NegotiationRpcRaw {
    /// Cap on RPC bodies, as on the DSP side.
    const MAX_BODY_BYTES: usize = 1024 * 1024;

    pub fn new(route: NegotiationRpcMessage, json: Value) -> Self {
        Self { route, json }
    }

    /// The request's body, which must be JSON.
    pub async fn from_request(route: NegotiationRpcMessage, request: Request) -> Outcome<Self> {
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
pub struct NegotiationRpcRdf {
    pub raw: NegotiationRpcRaw,
    pub expanded: Value,
    pub canonical_hash: [u8; 32],
}

impl NegotiationRpcRdf {
    pub async fn from_raw(raw: NegotiationRpcRaw) -> Outcome<Self> {
        let expansion = EunomiaNegotiation::engine().expand(&raw.json).await?;
        Ok(Self {
            canonical_hash: expansion.canonical_hash(),
            expanded: expansion.expanded,
            raw,
        })
    }
}

/// The body's fields and the RPC type it declares; agreeing with the route is a shape rule.
#[derive(Debug, Clone)]
pub struct NegotiationRpcTyped {
    pub rdf: NegotiationRpcRdf,
    pub message: NegotiationRpcMessage,
    pub body: NegotiationRpcBody,
}

impl NegotiationRpcTyped {
    pub fn from_rdf(rdf: NegotiationRpcRdf) -> Outcome<Self> {
        let (message, body) = EunomiaNegotiation::read_expanded(&rdf.expanded)?;
        Ok(Self { rdf, message, body })
    }

    /// Runs the stages on a body built in process, such as the TCK's or the BFF's.
    pub async fn from_body(
        route: NegotiationRpcMessage,
        body: &NegotiationRpcBody,
    ) -> Outcome<Self> {
        let raw = NegotiationRpcRaw::new(route, body.to_json_ld(route));
        Self::from_rdf(NegotiationRpcRdf::from_raw(raw).await?)
    }

    pub fn route(&self) -> NegotiationRpcMessage {
        self.rdf.raw.route
    }

    /// The id of the existing process the body moves.
    pub fn process_id(&self) -> Option<&str> {
        self.body.id.as_deref()
    }
}

/// The typed body plus everything resolved to send it: process, role, peer and the message.
pub struct NegotiationRpcDomain {
    pub typed: NegotiationRpcTyped,
    /// The process the message moves; none when it opens one.
    pub process: Option<NegotiationProcess>,
    pub role: NegotiationRole,
    pub owner: Owner,
    pub peer_id: String,
    /// The peer's participant record, when this connector knows it.
    pub peer_mate: Option<Mates>,
    /// The peer's DSP base, stored with a new process.
    pub peer_address: String,
    pub target_url: String,
    pub outgoing: RdfMessage<DspNegotiation>,
    /// The target of the offer that opened the process; the TCK names its tests by it.
    pub opening_target: Option<String>,
}

impl NegotiationRpcDomain {
    pub fn current_state(&self) -> Option<NegotiationState> {
        self.process.as_ref().map(|p| p.state)
    }

    pub fn existing(&self) -> Outcome<&NegotiationProcess> {
        self.process
            .as_ref()
            .ok_or_else(|| Errors::crazy("the message needs a stored process", None))
    }
}

/// What sending produced: what was asked, the peer's answer, the process as it stands, and the
/// role and offer the TCK continues from.
#[derive(Debug)]
pub struct NegotiationRpcOutcome {
    pub request: Value,
    pub response: Value,
    pub process: NegotiationProcess,
    pub role: NegotiationRole,
    pub opening_target: Option<String>,
}

impl NegotiationRpcOutcome {
    /// What sending `domain`'s message produced, given the peer's `reply` and the process it left.
    pub fn sent(
        domain: NegotiationRpcDomain,
        reply: &Value,
        process: NegotiationProcess,
    ) -> Self {
        Self {
            request: domain.typed.rdf.raw.json,
            response: reply["raw"].clone(),
            role: domain.role,
            opening_target: domain.opening_target,
            process,
        }
    }
}
