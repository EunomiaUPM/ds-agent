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

//! The outbound RPC registries: what a body needs, and the peer's answer to an opening message.

use negotiation_agent::protocols::dsp::entities::context_rpc::{
    NegotiationRpcBody, NegotiationRpcTyped,
};
use negotiation_agent::protocols::dsp::entities::message_types::NegotiationRpcMessage;
use negotiation_agent::protocols::dsp::services::validator::NegotiationRpcRules;
use serde_json::json;

async fn typed(
    route: NegotiationRpcMessage,
    kind: NegotiationRpcMessage,
    body: NegotiationRpcBody,
) -> NegotiationRpcTyped {
    let mut typed = NegotiationRpcTyped::from_body(kind, &body).await.unwrap();
    typed.rdf.raw.route = route;
    typed
}

/// An opening request needs its peer, the provider's address and an offer: all reported.
#[tokio::test]
async fn an_empty_request_init_reports_every_missing_member() {
    let message = NegotiationRpcMessage::RequestInit;
    let input = typed(message, message, NegotiationRpcBody::default()).await;
    let violations = NegotiationRpcRules::input_rules()
        .validate(&message, &input)
        .unwrap_err()
        .to_reasons();
    assert_eq!(violations.len(), 3, "{violations:?}");
}

/// A body whose RPC type is not the route's is refused.
#[tokio::test]
async fn a_body_of_another_rpc_is_refused() {
    let body = NegotiationRpcBody {
        id: Some("urn:negotiation-process:5c1a7a2e-0000-4000-8000-000000000001".into()),
        ..Default::default()
    };
    let input = typed(
        NegotiationRpcMessage::Termination,
        NegotiationRpcMessage::Acceptance,
        body,
    )
    .await;
    assert!(NegotiationRpcRules::input_rules()
        .validate(&NegotiationRpcMessage::Termination, &input)
        .is_err());
}

/// A state change names the process it moves by `@id`, a URN.
#[tokio::test]
async fn a_state_change_needs_a_process_id() {
    let message = NegotiationRpcMessage::Acceptance;
    let rules = NegotiationRpcRules::input_rules();
    let without = typed(message, message, NegotiationRpcBody::default()).await;
    assert!(rules.validate(&message, &without).is_err());
    let body = NegotiationRpcBody {
        id: Some("urn:negotiation-process:5c1a7a2e-0000-4000-8000-000000000001".into()),
        ..Default::default()
    };
    let with = typed(message, message, body).await;
    assert!(rules.validate(&message, &with).is_ok());
}

/// The answer to an opening request is a ContractNegotiation with the provider's pid.
#[test]
fn an_opening_request_needs_the_providers_ack() {
    let rules = NegotiationRpcRules::reply_rules();
    let message = NegotiationRpcMessage::RequestInit;
    let ok = json!({"type": "ContractNegotiation", "fields": {"providerPid": "urn:uuid:p"}});
    assert!(rules.validate(&message, &ok).is_ok());
    let no_pid = json!({"type": "ContractNegotiation", "fields": {}});
    assert!(rules.validate(&message, &no_pid).is_err());
    let error =
        json!({"type": "ContractNegotiationError", "fields": {"providerPid": "urn:uuid:p"}});
    assert!(rules.validate(&message, &error).is_err());
}

/// State changes take whatever 200 body the peer sends.
#[test]
fn state_changes_take_any_answer() {
    let rules = NegotiationRpcRules::reply_rules();
    assert!(rules
        .validate(&NegotiationRpcMessage::Verification, &json!({"raw": null}))
        .is_ok());
}
