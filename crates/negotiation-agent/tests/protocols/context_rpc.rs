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

//! The outbound RPC stages: the eunomia body built, expanded and read back, with no doubles.

use negotiation_agent::protocols::dsp::entities::context_rpc::{
    NegotiationRpcBody, NegotiationRpcRaw, NegotiationRpcRdf, NegotiationRpcTyped,
};
use negotiation_agent::protocols::dsp::entities::message_types::{
    NegotiationMessageType, NegotiationRpcMessage,
};
use serde_json::json;

/// Every RPC type reads back from its eunomia name and sends its DSP message.
#[test]
fn rpc_types_round_trip_and_send_their_message() {
    for message in NegotiationRpcMessage::ALL {
        assert_eq!(
            NegotiationRpcMessage::from_type(&message.to_string()),
            Some(message)
        );
    }
    assert_eq!(
        NegotiationRpcMessage::Finalization.message(),
        NegotiationMessageType::EventFinalized
    );
    assert!(NegotiationRpcMessage::OfferInit.opens());
    assert!(!NegotiationRpcMessage::Offer.opens());
}

/// An opening request keeps its peer, addresses and the offer as a JSON object.
#[tokio::test]
async fn a_request_init_body_reads_back() {
    let body = NegotiationRpcBody {
        associated_agent_peer: Some("did:web:provider".into()),
        provider_address: Some("http://provider/dsp/current".into()),
        offer: Some(json!({"@id": "urn:offer:1", "@type": "Offer", "target": "urn:dataset:1"})),
        ..Default::default()
    };
    let typed = NegotiationRpcTyped::from_body(NegotiationRpcMessage::RequestInit, &body)
        .await
        .unwrap();
    assert_eq!(typed.message, NegotiationRpcMessage::RequestInit);
    assert_eq!(
        typed.body.associated_agent_peer.as_deref(),
        Some("did:web:provider")
    );
    assert_eq!(
        typed.body.provider_address.as_deref(),
        Some("http://provider/dsp/current")
    );
    assert_eq!(typed.body.offer.unwrap()["target"], "urn:dataset:1");
}

/// A termination keeps the id of its process, code and reasons.
#[tokio::test]
async fn a_termination_body_reads_back() {
    let raw = NegotiationRpcRaw::new(
        NegotiationRpcMessage::Termination,
        json!({
            "@context": "https://w3id.org/eunomia/v1/context.jsonld",
            "@type": "RpcNegotiationTerminationMessage",
            "@id": "urn:negotiation-process:5c1a7a2e-0000-4000-8000-000000000001",
            "code": "stop",
            "reason": ["no deal"]
        }),
    );
    let typed =
        NegotiationRpcTyped::from_rdf(NegotiationRpcRdf::from_raw(raw).await.unwrap()).unwrap();
    assert_eq!(typed.process_id(), Some("urn:negotiation-process:5c1a7a2e-0000-4000-8000-000000000001"));
    assert_eq!(typed.body.code.as_deref(), Some("stop"));
    assert_eq!(typed.body.reason, Some(vec!["no deal".to_string()]));
}

/// A body outside the eunomia context declares no RPC.
#[tokio::test]
async fn a_body_without_the_eunomia_context_is_refused() {
    let raw = NegotiationRpcRaw::new(
        NegotiationRpcMessage::Acceptance,
        json!({"@id": "urn:negotiation-process:5c1a7a2e-0000-4000-8000-000000000001"}),
    );
    let read = NegotiationRpcTyped::from_rdf(NegotiationRpcRdf::from_raw(raw).await.unwrap());
    assert!(read.is_err());
}
