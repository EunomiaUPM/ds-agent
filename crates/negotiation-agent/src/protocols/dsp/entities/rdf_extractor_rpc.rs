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

//! Reading an outbound negotiation RPC body, typed with the eunomia context, off its expansion.
//! DSP concepts use the DSP vocabulary there; the offer travels as a JSON literal.

use common::eunomia::{EunomiaProfile, EUNOMIA_CONTEXT_URL, EUNOMIA_NS};
use common::rdf::{RdfEngine, RdfNode, RdfProtocolTrait};
use serde_json::{json, Value};
use ymir::errors::Outcome;

use crate::protocols::dsp::entities::context_rpc::NegotiationRpcBody;
use crate::protocols::dsp::entities::message_types::NegotiationRpcMessage;

/// Namespace of the DSP terms the eunomia context reuses.
const DSPACE: &str = "https://w3id.org/dspace/2025/1/";

/// The negotiation RPC, as an extraction target.
#[derive(Debug, Clone)]
pub struct EunomiaNegotiation;

impl RdfProtocolTrait for EunomiaNegotiation {
    type MessageType = NegotiationRpcMessage;
    type Fields = NegotiationRpcBody;

    fn engine() -> &'static RdfEngine {
        EunomiaProfile::shared()
    }

    fn context() -> Value {
        json!(EUNOMIA_CONTEXT_URL)
    }

    fn compact_type(message: &NegotiationRpcMessage) -> String {
        message.to_string()
    }

    fn type_iri(message: &NegotiationRpcMessage) -> String {
        EunomiaProfile::term(&message.to_string())
    }

    fn message_type(node: &RdfNode<'_, '_>) -> Option<NegotiationRpcMessage> {
        node.types()
            .filter_map(|iri| iri.strip_prefix(EUNOMIA_NS))
            .find_map(NegotiationRpcMessage::from_type)
    }

    fn extract(node: &RdfNode<'_, '_>) -> Outcome<NegotiationRpcBody> {
        let reason = node.get_strings(&Self::dspace("reason"));
        Ok(NegotiationRpcBody {
            id: node.id().map(str::to_string),
            associated_agent_peer: Self::owned(node, &EunomiaProfile::term("associatedAgentPeer")),
            provider_address: Self::owned(node, &EunomiaProfile::term("providerAddress")),
            consumer_address: Self::owned(node, &EunomiaProfile::term("consumerAddress")),
            callback_address: Self::owned(node, &Self::dspace("callbackAddress")),
            offer: node
                .value(&Self::dspace("offer"))
                .and_then(|v| v.get("@value"))
                .cloned(),
            code: Self::owned(node, &Self::dspace("code")),
            reason: (!reason.is_empty()).then_some(reason),
        })
    }
}

impl EunomiaNegotiation {
    fn dspace(term: &str) -> String {
        format!("{DSPACE}{term}")
    }

    fn owned(node: &RdfNode<'_, '_>, predicate: &str) -> Option<String> {
        node.iri_or_literal(predicate).map(str::to_string)
    }
}
