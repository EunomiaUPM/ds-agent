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

//! Rules of the outbound RPC flow: what the RPC body needs, what our role and state allow, the
//! shape of the message as it will go out, and the peer's answer.

use common::validation::{Stage, ValidationRegistry, Validator};
use serde_json::Value;

use crate::protocols::dsp::entities::context_rpc::{NegotiationRpcDomain, NegotiationRpcTyped};
use crate::protocols::dsp::entities::message_types::NegotiationRpcMessage;
use crate::protocols::dsp::entities::protocol_fields::NegotiationProtocolFields;
use crate::protocols::dsp::services::validator::{NegotiationDspRules, NegotiationRules};

/// Rule registries of the outbound negotiation flow.
pub struct NegotiationRpcRules;

impl NegotiationRpcRules {
    /// What the RPC body must carry before anything is looked up.
    pub fn input_rules() -> ValidationRegistry<NegotiationRpcMessage, NegotiationRpcTyped> {
        NegotiationRpcMessage::ALL.into_iter().fold(
            ValidationRegistry::new("negotiation.rpc.input")
                .with_any(Stage::Shape, Self::route_matches_body()),
            |rules, message| rules.with(message, Stage::Shape, Self::body_shape(message)),
        )
    }

    /// Our role and state must allow sending it, and the message must have the DSP shape.
    pub fn fact_rules() -> ValidationRegistry<NegotiationRpcMessage, NegotiationRpcDomain> {
        NegotiationRpcMessage::ALL.into_iter().fold(
            ValidationRegistry::new("negotiation.rpc.facts").with_any(
                Stage::Domain,
                Validator::new()
                    .rule(|f: &NegotiationRpcDomain| {
                        NegotiationRules::may_send(f.role, f.outgoing.message, "role")
                    })
                    .rule(|f: &NegotiationRpcDomain| {
                        NegotiationRules::state_transition(
                            f.current_state().as_ref(),
                            &f.outgoing.message,
                            "state",
                        )
                    }),
            ),
            |rules, message| {
                rules.with(
                    message,
                    Stage::Shape,
                    Self::outgoing(NegotiationDspRules::message_shape(message.message())),
                )
            },
        )
    }

    /// The answer to an opening message is the peer's `ContractNegotiation`, with its new pid.
    pub fn reply_rules() -> ValidationRegistry<NegotiationRpcMessage, Value> {
        let answered = |pid: &'static str| {
            Validator::new()
                .ensure(
                    "@type",
                    "the peer must answer with a ContractNegotiation",
                    |v: &Value| v["type"] == "ContractNegotiation",
                )
                .ensure_not_empty(pid, move |v: &Value| v["fields"][pid].as_str())
        };
        NegotiationRpcMessage::ALL.into_iter().fold(
            ValidationRegistry::new("negotiation.rpc.reply"),
            |rules, message| {
                let validator = match message {
                    NegotiationRpcMessage::RequestInit => answered("providerPid"),
                    NegotiationRpcMessage::OfferInit => answered("consumerPid"),
                    _ => Validator::new(),
                };
                rules.with(message, Stage::Shape, validator)
            },
        )
    }

    fn body_shape(message: NegotiationRpcMessage) -> Validator<NegotiationRpcTyped> {
        let offer = |v: Validator<NegotiationRpcTyped>| {
            v.ensure(
                "offer",
                "an offer is required",
                |t: &NegotiationRpcTyped| t.body.offer.as_ref().is_some_and(Value::is_object),
            )
        };
        let peer = Validator::new()
            .ensure_not_empty("associatedAgentPeer", |t: &NegotiationRpcTyped| {
                t.body.associated_agent_peer.as_deref()
            });
        let pid = Validator::new()
            .ensure_urn("@id", |t: &NegotiationRpcTyped| t.process_id());
        match message {
            NegotiationRpcMessage::RequestInit => offer(
                peer.ensure_not_empty("providerAddress", |t: &NegotiationRpcTyped| {
                    t.body.provider_address.as_deref()
                }),
            ),
            NegotiationRpcMessage::OfferInit => offer(
                peer.ensure_not_empty("consumerAddress", |t: &NegotiationRpcTyped| {
                    t.body.consumer_address.as_deref()
                }),
            ),
            NegotiationRpcMessage::Request | NegotiationRpcMessage::Offer => offer(pid),
            _ => pid,
        }
    }

    /// The body's `@type` must be the RPC the route serves.
    fn route_matches_body() -> Validator<NegotiationRpcTyped> {
        Validator::new().ensure(
            "@type",
            "RPC type does not match the endpoint",
            |t: &NegotiationRpcTyped| t.message == t.route(),
        )
    }

    /// Runs a message-shape validator on the message as it will be sent.
    fn outgoing(fields: Validator<NegotiationProtocolFields>) -> Validator<NegotiationRpcDomain> {
        Validator::new().rule(move |f: &NegotiationRpcDomain| fields.validate(&f.outgoing.fields))
    }
}
