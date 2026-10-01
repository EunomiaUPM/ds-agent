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

//! Typed extraction through `FromRdf`, with sample DSP message types.

use serde_json::json;
use ymir::errors::Outcome;

use super::sample_transfer_request;
use crate::rdf::{FromRdf, RdfEngine, RdfNode};

const DSPACE_TRANSFER_REQUEST: &str = "https://w3id.org/dspace/2025/1/TransferRequestMessage";
const DSPACE_DATA_ADDRESS: &str = "https://w3id.org/dspace/2025/1/DataAddress";
const DSPACE_ENDPOINT_PROPERTY: &str = "https://w3id.org/dspace/2025/1/EndpointProperty";

#[derive(Debug, PartialEq, Eq)]
struct TestEndpointProperty {
    name: String,
    value: String,
}

impl FromRdf for TestEndpointProperty {
    const TYPE_IRI: Option<&'static str> = Some(DSPACE_ENDPOINT_PROPERTY);

    fn from_rdf(node: &RdfNode<'_, '_>) -> Outcome<Self> {
        Ok(Self {
            name: node.get_string("https://w3id.org/dspace/2025/1/name")?,
            value: node.get_string("https://w3id.org/dspace/2025/1/value")?,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
struct TestDataAddress {
    endpoint_type: String,
    endpoint: Option<String>,
    properties: Vec<TestEndpointProperty>,
}

impl FromRdf for TestDataAddress {
    const TYPE_IRI: Option<&'static str> = Some(DSPACE_DATA_ADDRESS);

    fn from_rdf(node: &RdfNode<'_, '_>) -> Outcome<Self> {
        Ok(Self {
            endpoint_type: node.get_string("https://w3id.org/dspace/2025/1/endpointType")?,
            endpoint: node.get_opt_string("https://w3id.org/dspace/2025/1/endpoint")?,
            properties: node.get_list("https://w3id.org/dspace/2025/1/endpointProperties")?,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
struct TestTransferRequest {
    consumer_pid: String,
    agreement_id: String,
    callback_address: Option<String>,
    format: Option<String>,
    data_address: Option<TestDataAddress>,
}

impl FromRdf for TestTransferRequest {
    const TYPE_IRI: Option<&'static str> = Some(DSPACE_TRANSFER_REQUEST);

    fn from_rdf(node: &RdfNode<'_, '_>) -> Outcome<Self> {
        Ok(Self {
            consumer_pid: node.get_string("https://w3id.org/dspace/2025/1/consumerPid")?,
            agreement_id: node.get_string("https://w3id.org/dspace/2025/1/agreementId")?,
            callback_address: node
                .get_opt_string("https://w3id.org/dspace/2025/1/callbackAddress")?,
            format: node.get_opt_string("http://purl.org/dc/terms/format")?,
            data_address: node.get_opt_object("https://w3id.org/dspace/2025/1/dataAddress")?,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
struct TestTerminationMessage {
    consumer_pid: String,
    provider_pid: String,
    code: u64,
    reasons: Vec<String>,
}

impl FromRdf for TestTerminationMessage {
    const TYPE_IRI: Option<&'static str> =
        Some("https://w3id.org/dspace/2025/1/TransferTerminationMessage");

    fn from_rdf(node: &RdfNode<'_, '_>) -> Outcome<Self> {
        Ok(Self {
            consumer_pid: node.get_string("https://w3id.org/dspace/2025/1/consumerPid")?,
            provider_pid: node.get_string("https://w3id.org/dspace/2025/1/providerPid")?,
            code: node.get_u64("https://w3id.org/dspace/2025/1/code")?,
            reasons: node.get_strings("https://w3id.org/dspace/2025/1/reason"),
        })
    }
}

/// A full transfer request extracts with its nested data address and properties.
#[tokio::test]
async fn extracts_typed_transfer_request_successfully() {
    let engine = RdfEngine::dsp();
    let msg: TestTransferRequest = engine
        .extract(&sample_transfer_request())
        .await
        .expect("successful extraction");

    assert_eq!(
        msg.consumer_pid,
        "urn:uuid:32541fe6-c580-409e-85a8-8a9a32fbe833"
    );
    assert_eq!(
        msg.agreement_id,
        "urn:uuid:e8dc8655-44c2-46ef-b701-4cffdc2faa44"
    );
    assert_eq!(
        msg.callback_address.as_deref(),
        Some("https://example.com/callback")
    );
    assert_eq!(msg.format.as_deref(), Some("example:HTTP_PUSH"));

    let data_addr = msg.data_address.expect("dataAddress present");
    assert_eq!(data_addr.endpoint.as_deref(), Some("http://example.com"));
    assert_eq!(data_addr.endpoint_type, "https://w3id.org/idsa/v4.1/HTTP");
    assert_eq!(data_addr.properties.len(), 2);
}

/// Extraction can also return the canonical SHA-256 of the message.
#[tokio::test]
async fn extracts_with_canonical_hash() {
    let engine = RdfEngine::dsp();
    let (msg, hash) = engine
        .extract_with_hash::<TestTransferRequest>(&sample_transfer_request())
        .await
        .expect("extraction with hash");

    assert_eq!(
        msg.consumer_pid,
        "urn:uuid:32541fe6-c580-409e-85a8-8a9a32fbe833"
    );
    assert_eq!(hash.len(), 64);
}

/// Inside an `@graph` the root is the node of the expected type, not the first one.
#[tokio::test]
async fn resolves_root_node_inside_graph_payload() {
    let graph_payload = json!({
        "@context": ["https://w3id.org/dspace/2025/1/context.jsonld"],
        "@graph": [
            {
                "@id": "_:extra_auxiliary_node",
                "@type": "https://example.org/AuxiliaryType",
                "https://example.org/note": [{"@value": "auxiliary note"}]
            },
            {
                "@id": "_:main_transfer_request",
                "@type": "TransferRequestMessage",
                "agreementId": "urn:uuid:agreement-in-graph",
                "consumerPid": "urn:uuid:consumer-in-graph"
            }
        ]
    });

    let engine = RdfEngine::dsp();
    let msg: TestTransferRequest = engine
        .extract(&graph_payload)
        .await
        .expect("resolves root in graph");

    assert_eq!(msg.consumer_pid, "urn:uuid:consumer-in-graph");
    assert_eq!(msg.agreement_id, "urn:uuid:agreement-in-graph");
    assert!(msg.data_address.is_none());
}

/// A missing required predicate fails the extraction.
#[tokio::test]
async fn reports_missing_required_predicate() {
    let incomplete = json!({
        "@context": ["https://w3id.org/dspace/2025/1/context.jsonld"],
        "@type": "TransferRequestMessage",
        "consumerPid": "urn:uuid:consumer-only"
    });

    let engine = RdfEngine::dsp();
    let result = engine.extract::<TestTransferRequest>(&incomplete).await;
    assert!(result.is_err(), "missing agreementId must fail");
}

/// Numeric literals parse to integers and repeated terms collect into a list.
#[tokio::test]
async fn extracts_numbers_and_multi_value_strings() {
    let termination_json = json!({
        "@context": ["https://w3id.org/dspace/2025/1/context.jsonld"],
        "@type": "TransferTerminationMessage",
        "consumerPid": "urn:uuid:cc",
        "providerPid": "urn:uuid:pp",
        "code": "403",
        "reason": ["Policy violated", "Agreement expired"]
    });

    let engine = RdfEngine::dsp();
    let term: TestTerminationMessage = engine
        .extract(&termination_json)
        .await
        .expect("extract termination");

    assert_eq!(term.code, 403);
    assert_eq!(term.reasons.len(), 2);
}
