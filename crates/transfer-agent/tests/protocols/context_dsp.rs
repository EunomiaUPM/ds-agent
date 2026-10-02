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

//! The DSP request context: reading the raw request, RDF canonicalization and typed field
//! extraction.

use transfer_agent::entities::protocol::ProtocolId;
use transfer_agent::protocols::dsp::entities::auth::TransferDSPAuthn;
use transfer_agent::protocols::dsp::entities::context_common::TransferContextRaw;
use transfer_agent::protocols::dsp::entities::message_types::TransferDSPMessageType;
use ymir::data::entities::shared::participant::Model as Mates;

use axum::extract::Request;
use chrono::Utc;
use serde_json::Value;
use transfer_agent::entities::transfer_message::Direction;
use transfer_agent::protocols::dsp::entities::context_dsp::*;
use ymir::types::participants::ParticipantType;

fn mate() -> Mates {
    let t = Utc::now();
    Mates {
        tenant_id: "default".to_string(),
        participant_id: "did:example:provider".into(),
        participant_type: ParticipantType::Agent,
        participant_nick: "provider".to_string(),
        base_url: "http://127.0.0.1:122".to_string(),
        token: None,
        saved_at: t,
        last_interaction: t,
        extra_fields: Value::Null,
    }
}

fn request_with_mate(body: &'static str) -> Request {
    let mut req = Request::builder()
        .method("POST")
        .uri("/transfers/123/start?x=1")
        .header("x-request-id", "req-1")
        .header("authorization", "Bearer eyJabc")
        .body(axum::body::Body::from(body))
        .unwrap();
    // The auth middleware would have inserted this before we run.
    req.extensions_mut().insert(mate());
    req
}

/// The raw context reads request id, paths, token and the participant the auth
/// middleware resolved.
#[tokio::test]
async fn from_request_reads_wire_fields_and_resolved_participant() {
    let raw = TransferContextRaw::<TransferDSPAuthn>::from_request(request_with_mate("{}"))
        .await
        .unwrap();
    assert_eq!(raw.request_id.as_str(), "req-1");
    assert_eq!(raw.request_path, "/transfers/123/start");
    assert_eq!(raw.request_full_path, "/transfers/123/start?x=1");
    assert_eq!(raw.authn.token_type, "Bearer");
    assert_eq!(raw.authn.token_content, "eyJabc");
    assert_eq!(
        raw.authn.associated_participant.participant_nick,
        "provider"
    );
    assert!(matches!(raw.direction, Direction::Inbound));
}

/// Without Mates in the extensions this is a wiring error, not a silent None.
#[tokio::test]
async fn from_request_fails_without_auth_middleware() {
    let req = Request::builder().body(axum::body::Body::empty()).unwrap();
    assert!(
        TransferContextRaw::<TransferDSPAuthn>::from_request(req)
            .await
            .is_err()
    );
}

/// Real JSON-LD expansion resolves everything to full IRIs, so the same
/// TransferStartMessage with fields in different source order canonicalizes
/// to identical n-quads.
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
        !a.canonical_n_quads.is_empty(),
        "expansion must yield quads"
    );
    assert_eq!(
        a.canonical_hash, b.canonical_hash,
        "canonicalization must be key-order independent"
    );

    // A different provider pid must give a different canonical hash.
    let c = rdf_from(
        r#"{"@context":"https://w3id.org/dspace/2025/1/context.jsonld","@type":"TransferStartMessage","providerPid":"urn:uuid:XX","consumerPid":"urn:uuid:cc"}"#,
    )
    .await;
    assert_ne!(
        a.canonical_hash, c.canonical_hash,
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
    let typed = TransferDSPContextTyped::from_rdf(rdf).unwrap();
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
    let typed = TransferDSPContextTyped::from_rdf(rdf).unwrap();
    assert_eq!(typed.fields.agreement_id.as_deref(), Some("urn:uuid:ag"));
    assert_eq!(
        typed.fields.callback_address.as_deref(),
        Some("https://example.com/callback")
    );
    assert_eq!(typed.fields.format.as_deref(), Some("example:HTTP_PUSH"));
}

/// The same message as one nested object and as an `@graph` of
/// mutually-referencing nodes. Both expand to the same RDF graph — the
/// canonicalizer gives them the same hash — so both must extract the same
/// fields. Requiring the expansion to be a single node rejected the second.
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
        nested.canonical_hash, graph.canonical_hash,
        "the two forms are the same graph"
    );

    let nested = TransferDSPContextTyped::from_rdf(nested).unwrap();
    let graph = TransferDSPContextTyped::from_rdf(graph).unwrap();

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
/// and disagreeing with the endpoint is the manager's to reject.
#[tokio::test]
async fn the_body_type_is_read_even_when_the_route_disagrees() {
    let rdf = rdf_from_typed(
        r#"{"@context":"https://w3id.org/dspace/2025/1/context.jsonld","@type":"TransferProcess","consumerPid":"urn:uuid:cc"}"#,
        TransferDSPMessageType::TransferRequestMessage,
    )
    .await;
    let typed = TransferDSPContextTyped::from_rdf(rdf).unwrap();
    assert_eq!(typed.message, TransferDSPMessageType::TransferProcess);
    assert_eq!(
        typed.rdf.parsed.dsp_message_type,
        TransferDSPMessageType::TransferRequestMessage,
        "the route's own type stays available to compare against"
    );
    assert_eq!(typed.fields.consumer_pid.as_deref(), Some("urn:uuid:cc"));
}

/// The one thing extraction still cannot do: build fields out of a body that
/// declares no DSP message type at all.
#[tokio::test]
async fn a_body_declaring_no_message_type_is_rejected() {
    let rdf = rdf_from(
        r#"{"@context":"https://w3id.org/dspace/2025/1/context.jsonld","@type":"DataAddress","endpointType":"https://w3id.org/idsa/v4.1/HTTP"}"#,
    )
    .await;
    assert!(TransferDSPContextTyped::from_rdf(rdf).is_err());
}

/// A message without dataAddress or pids extracts to None, not an error.
#[tokio::test]
async fn extractor_tolerates_missing_optional_fields() {
    let rdf =
        rdf_from(r#"{"@context":"https://w3id.org/dspace/2025/1/context.jsonld","@type":"TransferStartMessage"}"#).await;
    let typed = TransferDSPContextTyped::from_rdf(rdf).unwrap();
    assert!(typed.fields.provider_pid.is_none());
    assert!(typed.fields.data_address.is_none());
}

async fn rdf_from_typed(
    body: &'static str,
    message_type: TransferDSPMessageType,
) -> TransferDSPContextRdf {
    let raw = TransferContextRaw::<TransferDSPAuthn>::from_request(request_with_mate(body))
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&raw.body_bytes).unwrap();
    let parsed =
        TransferDSPContextParsed::from_raw(raw, &ProtocolId::Dsp2025_1, &message_type, json)
            .unwrap();
    TransferDSPContextRdf::from_parsed(parsed).await.unwrap()
}

async fn rdf_from(body: &'static str) -> TransferDSPContextRdf {
    let raw = TransferContextRaw::<TransferDSPAuthn>::from_request(request_with_mate(body))
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&raw.body_bytes).unwrap();
    let parsed = TransferDSPContextParsed::from_raw(
        raw,
        &ProtocolId::Dsp2025_1,
        &TransferDSPMessageType::TransferStartMessage,
        json,
    )
    .unwrap();
    TransferDSPContextRdf::from_parsed(parsed).await.unwrap()
}
