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

//! The inbound DSP stages: reading the request, JSON, RDF canonicalization, typed field
//! extraction and the idempotency key.

use axum::extract::Request;
use common::facades::grants_facade::VerifiedPeer;
use ymir::types::oauth::RolePath;
use transfer_agent::entities::protocol::ProtocolId;
use transfer_agent::protocols::dsp::entities::context_dsp::{
    TransferDspParsed, TransferDspRaw, TransferDspRdf, TransferDspTyped,
};
use transfer_agent::protocols::dsp::entities::message_types::TransferDSPMessageType;
use ymir::errors::Outcome;

fn mate() -> VerifiedPeer {
    VerifiedPeer {
        participant_id: "did:example:provider".into(),
        role: RolePath::root(),
        visibility: ymir::types::participants::Visibility::Public,
    }
}

fn request_with_mate(body: &'static str) -> Request {
    Request::builder()
        .method("POST")
        .uri("/transfers/123/start")
        .body(axum::body::Body::from(body))
        .unwrap()
}

async fn typed_from(request: Request, route: TransferDSPMessageType) -> Outcome<TransferDspTyped> {
    let raw = TransferDspRaw::from_request(mate(), request).await?;
    let parsed = TransferDspParsed::from_raw(raw, &ProtocolId::Dsp2025_1, &route)?;
    TransferDspTyped::from_rdf(TransferDspRdf::from_parsed(parsed).await?)
}

async fn try_input(body: &'static str, route: TransferDSPMessageType) -> Outcome<TransferDspTyped> {
    typed_from(request_with_mate(body), route).await
}

async fn rdf_from_typed(body: &'static str, route: TransferDSPMessageType) -> TransferDspTyped {
    try_input(body, route).await.unwrap()
}

async fn rdf_from(body: &'static str) -> TransferDspTyped {
    rdf_from_typed(body, TransferDSPMessageType::TransferStartMessage).await
}

/// The stages keep the peer the auth layer resolved and the route's message type.
#[tokio::test]
async fn stages_keep_the_resolved_peer_and_the_route() {
    let input = rdf_from(
        r#"{"@context":"https://w3id.org/dspace/2025/1/context.jsonld","@type":"TransferStartMessage","providerPid":"urn:uuid:pp","consumerPid":"urn:uuid:cc"}"#,
    )
    .await;
    assert_eq!(input.peer().participant_id, "did:example:provider");
    assert_eq!(input.route(), &TransferDSPMessageType::TransferStartMessage);
}

/// Expansion resolves everything to full IRIs, so field order does not change the canonical
/// hash, and different content does.
#[tokio::test]
async fn canonical_hash_is_deterministic_and_key_order_independent() {
    let a = rdf_from(
        r#"{"@context":"https://w3id.org/dspace/2025/1/context.jsonld","@type":"TransferStartMessage","providerPid":"urn:uuid:pp","consumerPid":"urn:uuid:cc"}"#,
    )
    .await;
    let b = rdf_from(
        r#"{"@context":"https://w3id.org/dspace/2025/1/context.jsonld","consumerPid":"urn:uuid:cc","@type":"TransferStartMessage","providerPid":"urn:uuid:pp"}"#,
    )
    .await;
    assert!(
        !a.rdf.canonical_n_quads.is_empty(),
        "expansion must yield quads"
    );
    assert_eq!(
        a.rdf.canonical_hash, b.rdf.canonical_hash,
        "canonicalization must be key-order independent"
    );

    // A different provider pid must give a different canonical hash.
    let c = rdf_from(
        r#"{"@context":"https://w3id.org/dspace/2025/1/context.jsonld","@type":"TransferStartMessage","providerPid":"urn:uuid:XX","consumerPid":"urn:uuid:cc"}"#,
    )
    .await;
    assert_ne!(
        a.rdf.canonical_hash, c.rdf.canonical_hash,
        "different content must hash differently"
    );
}

/// Extraction reads the pids, the message type and the data address.
#[tokio::test]
async fn extractor_pulls_pids_message_and_data_address() {
    let rdf = rdf_from(
        r#"{"@context":"https://w3id.org/dspace/2025/1/context.jsonld","@type":"TransferStartMessage","providerPid":"urn:uuid:pp","consumerPid":"urn:uuid:cc","dataAddress":{"@type":"DataAddress","endpointType":"HttpData","endpoint":"http://example.com/data","endpointProperties":[]}}"#,
    )
    .await;
    let typed = rdf;
    assert_eq!(typed.fields.provider_pid.as_deref(), Some("urn:uuid:pp"));
    assert_eq!(typed.fields.consumer_pid.as_deref(), Some("urn:uuid:cc"));
    assert!(
        !typed.idempotency_key.as_str().is_empty(),
        "the effective key is always derived, never left empty"
    );
    assert_eq!(
        typed.fields.data_address.unwrap().endpoint.as_deref(),
        Some("http://example.com/data")
    );
    assert!(matches!(
        typed.message,
        TransferDSPMessageType::TransferStartMessage
    ));
}

/// A `TransferRequestMessage` carries `agreementId`, `callbackAddress` and
/// `format`; the extractor could already read them but nothing surfaced them.
#[tokio::test]
async fn typed_carries_the_request_message_fields() {
    let rdf = rdf_from_typed(
        r#"{"@context":"https://w3id.org/dspace/2025/1/context.jsonld","@type":"TransferRequestMessage","consumerPid":"urn:uuid:cc","agreementId":"urn:uuid:ag","callbackAddress":"https://example.com/callback","format":"example:HTTP_PUSH"}"#,
        TransferDSPMessageType::TransferRequestMessage,
    )
    .await;
    let typed = rdf;
    assert_eq!(typed.fields.agreement_id.as_deref(), Some("urn:uuid:ag"));
    assert_eq!(
        typed.fields.callback_address.as_deref(),
        Some("https://example.com/callback")
    );
    assert_eq!(typed.fields.format.as_deref(), Some("example:HTTP_PUSH"));
}

/// A message as one nested object and as an `@graph` of referencing nodes is the same graph,
/// so both hash and extract the same.
#[tokio::test]
async fn graph_form_extracts_the_same_as_the_nested_form() {
    const NESTED: &str = r#"{
        "@context": "https://w3id.org/dspace/2025/1/context.jsonld",
        "@type": "TransferRequestMessage",
        "consumerPid": "urn:uuid:cc",
        "agreementId": "urn:uuid:ag",
        "format": "example:HTTP_PUSH",
        "callbackAddress": "https://example.com/callback",
        "dataAddress": {
            "@type": "DataAddress",
            "endpointType": "https://w3id.org/idsa/v4.1/HTTP",
            "endpoint": "http://example.com",
            "endpointProperties": [
                {"@type": "EndpointProperty", "name": "authorization", "value": "TOKEN-ABCDEFG"},
                {"@type": "EndpointProperty", "name": "authType", "value": "bearer"}
            ]
        }
    }"#;
    const GRAPH: &str = r#"{
        "@context": "https://w3id.org/dspace/2025/1/context.jsonld",
        "@graph": [
            {"@id": "_:msg", "@type": "TransferRequestMessage",
             "consumerPid": "urn:uuid:cc",
             "agreementId": "urn:uuid:ag",
             "format": "example:HTTP_PUSH",
             "callbackAddress": "https://example.com/callback",
             "dataAddress": {"@id": "_:addr"}},
            {"@id": "_:addr", "@type": "DataAddress",
             "endpointType": "https://w3id.org/idsa/v4.1/HTTP",
             "endpoint": "http://example.com",
             "endpointProperties": [{"@id": "_:p1"}, {"@id": "_:p2"}]},
            {"@id": "_:p1", "@type": "EndpointProperty", "name": "authorization", "value": "TOKEN-ABCDEFG"},
            {"@id": "_:p2", "@type": "EndpointProperty", "name": "authType", "value": "bearer"}
        ]
    }"#;

    let nested = rdf_from_typed(NESTED, TransferDSPMessageType::TransferRequestMessage).await;
    let graph = rdf_from_typed(GRAPH, TransferDSPMessageType::TransferRequestMessage).await;
    assert_eq!(
        nested.rdf.canonical_hash, graph.rdf.canonical_hash,
        "the two forms are the same graph"
    );

    assert_eq!(graph.fields, nested.fields, "both forms extract the same");
    assert_eq!(graph.idempotency_key, nested.idempotency_key);

    // The dataAddress arrives as a bare `{"@id": …}` reference in the graph
    // form, so it only resolves if references are followed.
    let address = graph.fields.data_address.expect("dataAddress must resolve");
    assert_eq!(address.endpoint.as_deref(), Some("http://example.com"));
    assert_eq!(address.endpoint_type, "https://w3id.org/idsa/v4.1/HTTP");
    let mut properties: Vec<(String, String)> = address
        .endpoint_properties
        .iter()
        .map(|p| (p.name.clone(), p.value.clone()))
        .collect();
    properties.sort();
    assert_eq!(
        properties,
        vec![
            ("authType".to_string(), "bearer".to_string()),
            ("authorization".to_string(), "TOKEN-ABCDEFG".to_string()),
        ],
        "endpointProperties are references too"
    );
}

/// The body's `@type` wins over the route's: extraction reports what arrived,
/// and disagreeing with the endpoint is an input rule's to reject.
#[tokio::test]
async fn the_body_type_is_read_even_when_the_route_disagrees() {
    let rdf = rdf_from_typed(
        r#"{"@context":"https://w3id.org/dspace/2025/1/context.jsonld","@type":"TransferProcess","consumerPid":"urn:uuid:cc"}"#,
        TransferDSPMessageType::TransferRequestMessage,
    )
    .await;
    let typed = rdf;
    assert_eq!(typed.message, TransferDSPMessageType::TransferProcess);
    assert_eq!(
        typed.route(),
        &TransferDSPMessageType::TransferRequestMessage,
        "the route's own type stays available to compare against"
    );
    assert_eq!(typed.fields.consumer_pid.as_deref(), Some("urn:uuid:cc"));
}

/// The one thing extraction still cannot do: build fields out of a body that
/// declares no DSP message type at all.
#[tokio::test]
async fn a_body_declaring_no_message_type_is_rejected() {
    let read = try_input(
        r#"{"@context":"https://w3id.org/dspace/2025/1/context.jsonld","@type":"DataAddress","endpointType":"https://w3id.org/idsa/v4.1/HTTP"}"#,
        TransferDSPMessageType::TransferStartMessage,
    )
    .await;
    assert!(read.is_err());
}

/// A message without dataAddress or pids extracts to None, not an error.
#[tokio::test]
async fn extractor_tolerates_missing_optional_fields() {
    let rdf =
        rdf_from(r#"{"@context":"https://w3id.org/dspace/2025/1/context.jsonld","@type":"TransferStartMessage"}"#).await;
    let typed = rdf;
    assert!(typed.fields.provider_pid.is_none());
    assert!(typed.fields.data_address.is_none());
}

/// DSP/RPC control-plane messages format diagnostic dataplane transition reasons consistently.
#[test]
fn dataplane_hook_reason_formats_dsp_lifecycle_and_error_fields() {
    use transfer_agent::protocols::dsp::facades::dataplane_facade::DataplaneHookContext;

    assert_eq!(
        DataplaneHookContext::format_message_reason(
            &TransferDSPMessageType::TransferStartMessage,
            "DSP peer",
            None,
            &[],
            false,
        )
        .as_deref(),
        Some("Transfer started (DSP peer: TransferStartMessage)")
    );
    assert_eq!(
        DataplaneHookContext::format_message_reason(
            &TransferDSPMessageType::TransferStartMessage,
            "DSP peer",
            None,
            &[],
            true,
        )
        .as_deref(),
        Some("Transfer resumed (DSP peer: TransferStartMessage)")
    );
    assert_eq!(
        DataplaneHookContext::format_message_reason(
            &TransferDSPMessageType::TransferTerminationMessage,
            "local RPC",
            Some(" DATAPLANE_FORBIDDEN_TARGET "),
            &[" ".into(), "private, loopback, or reserved target IP blocked".into()],
            false,
        )
        .as_deref(),
        Some(
            "Transfer terminated (local RPC: TransferTerminationMessage) [code=DATAPLANE_FORBIDDEN_TARGET]: private, loopback, or reserved target IP blocked"
        )
    );
    assert_eq!(
        DataplaneHookContext::format_message_reason(
            &TransferDSPMessageType::TransferCompletionMessage,
            "DSP peer",
            None,
            &[],
            false,
        )
        .as_deref(),
        Some("Transfer completed (DSP peer: TransferCompletionMessage)")
    );
    assert_eq!(
        DataplaneHookContext::format_message_reason(
            &TransferDSPMessageType::TransferError,
            "DSP peer",
            Some("ERR"),
            &["ignored".into()],
            false,
        ),
        None
    );
}
