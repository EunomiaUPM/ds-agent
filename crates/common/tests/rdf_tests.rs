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

use common::dsp_common::rdf::{DSP_CONTEXT_URL, DSP_ODRL_PROFILE_URL};
use common::rdf::{ExpandedDoc, FromRdf, RdfCanonicalizer, RdfEngine, RdfNode};
use serde_json::{json, Value};
use ymir::errors::Outcome;

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

fn sample_transfer_request() -> serde_json::Value {
    json!({
        "@context": ["https://w3id.org/dspace/2025/1/context.jsonld"],
        "@type": "TransferRequestMessage",
        "agreementId": "urn:uuid:e8dc8655-44c2-46ef-b701-4cffdc2faa44",
        "callbackAddress": "https://example.com/callback",
        "consumerPid": "urn:uuid:32541fe6-c580-409e-85a8-8a9a32fbe833",
        "dataAddress": {
            "@type": "DataAddress",
            "endpoint": "http://example.com",
            "endpointProperties": [
                {"@type": "EndpointProperty", "name": "authorization", "value": "TOKEN-ABCDEFG"},
                {"@type": "EndpointProperty", "name": "authType", "value": "bearer"}
            ],
            "endpointType": "https://w3id.org/idsa/v4.1/HTTP"
        },
        "format": "example:HTTP_PUSH"
    })
}

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

#[tokio::test]
async fn dsp_engine_preloads_canonical_dsp_context() {
    let engine = RdfEngine::dsp();
    assert!(engine.loader().is_cached(DSP_CONTEXT_URL).await);
}

#[tokio::test]
async fn generic_engine_handles_non_dsp_jsonld() {
    let generic_json = json!({
        "@context": {
            "name": "http://xmlns.com/foaf/0.1/name",
            "knows": {"@id": "http://xmlns.com/foaf/0.1/knows", "@type": "@id"}
        },
        "@id": "http://example.org/alice",
        "@type": "http://xmlns.com/foaf/0.1/Person",
        "name": "Alice",
        "knows": "http://example.org/bob"
    });

    let engine = RdfEngine::new();
    let n_quads = engine
        .canonicalize(&generic_json)
        .await
        .expect("generic canonicalize");
    assert!(n_quads.contains("<http://example.org/alice>"));
    assert!(n_quads.contains("<http://xmlns.com/foaf/0.1/name>"));
    assert!(n_quads.contains(r#""Alice""#));
}

async fn canon(v: serde_json::Value) -> String {
    RdfEngine::dsp().canonicalize(&v).await.unwrap()
}

#[tokio::test]
async fn canonical_hash_is_stable() {
    let engine = RdfEngine::dsp();
    let hash = engine.hash(&sample_transfer_request()).await.unwrap();
    assert_eq!(
        hash,
        "2a74c50cad115b2b9e21b3d5c580d7263cc754328e5eb4b159416f71f2ef5b1a"
    );
}

#[tokio::test]
async fn expanded_document_exposes_iris_and_term_kinds() {
    let expansion = RdfEngine::dsp()
        .expand(&sample_transfer_request())
        .await
        .unwrap();

    let node = &expansion.expanded.as_array().unwrap()[0];
    assert_eq!(
        node["https://w3id.org/dspace/2025/1/consumerPid"][0]["@id"],
        "urn:uuid:32541fe6-c580-409e-85a8-8a9a32fbe833",
        "consumerPid is @id-typed by the DSP context"
    );
    assert_eq!(
        node["https://w3id.org/dspace/2025/1/callbackAddress"][0]["@value"],
        "https://example.com/callback",
        "callbackAddress is a plain literal"
    );
    assert_eq!(
        node["http://purl.org/dc/terms/format"][0]["@id"],
        "example:HTTP_PUSH"
    );
}

#[tokio::test]
async fn prefixed_and_stripped_terms_expand_alike() {
    let stripped = RdfEngine::dsp()
        .expand(&json!({
            "@context": "https://w3id.org/dspace/2025/1/context.jsonld",
            "@type": "TransferStartMessage",
            "consumerPid": "urn:uuid:cc"
        }))
        .await
        .unwrap();

    let prefixed = RdfEngine::dsp()
        .expand(&json!({
            "@context": {"dspace": "https://w3id.org/dspace/2025/1/"},
            "@type": "dspace:TransferStartMessage",
            "dspace:consumerPid": {"@id": "urn:uuid:cc"}
        }))
        .await
        .unwrap();

    let pid = |v: &serde_json::Value| {
        v.as_array().unwrap()[0]["https://w3id.org/dspace/2025/1/consumerPid"][0]["@id"]
            .as_str()
            .unwrap()
            .to_string()
    };
    assert_eq!(pid(&stripped.expanded), pid(&prefixed.expanded));
}

#[tokio::test]
async fn deterministic_and_key_order_independent() {
    let a = canon(json!({
        "@context": DSP_CONTEXT_URL,
        "@type": "TransferStartMessage",
        "providerPid": "urn:uuid:pp",
        "consumerPid": "urn:uuid:cc"
    }))
    .await;

    let b = canon(json!({
        "@context": DSP_CONTEXT_URL,
        "consumerPid": "urn:uuid:cc",
        "@type": "TransferStartMessage",
        "providerPid": "urn:uuid:pp"
    }))
    .await;
    assert!(!a.is_empty(), "expansion must yield quads");
    assert_eq!(a, b, "canonicalization must be key-order independent");

    let c = canon(json!({
        "@context": DSP_CONTEXT_URL,
        "@type": "TransferStartMessage",
        "providerPid": "urn:uuid:XX",
        "consumerPid": "urn:uuid:cc"
    }))
    .await;
    assert_ne!(a, c, "different content must canonicalize differently");
}

#[tokio::test]
async fn undeclared_xsd_prefix_still_expands() {
    let out = canon(json!({
        "@context": {"ex": "http://example.org/"},
        "@id": "http://example.org/m1",
        "ex:issued": {"@value": "2026-09-02T00:00:00Z", "@type": "xsd:dateTime"}
    }))
    .await;
    assert!(
        out.contains("http://www.w3.org/2001/XMLSchema#dateTime"),
        "xsd: must resolve even when the message omits the prefix, got: {out}"
    );
    assert!(
        !out.contains("<xsd:"),
        "prefix must not survive as an IRI: {out}"
    );
}

#[tokio::test]
async fn odrl_offer_constraint_datatypes_expand() {
    let out = canon(json!({
        "@context": [DSP_ODRL_PROFILE_URL],
        "@id": "urn:policy:0000-00-0",
        "@type": "Offer",
        "permission": [{
            "@type": "Permission",
            "action": "use",
            "constraint": [{
                "leftOperand": "odrl:dateTime",
                "operator": "odrl:lteq",
                "rightOperand": {"@type": "xsd:dateTime", "@value": "2026-12-31T23:59:59Z"}
            }, {
                "leftOperand": "odrl:count",
                "operator": "odrl:lteq",
                "rightOperand": {"@type": "xsd:integer", "@value": "2"}
            }]
        }],
        "target": {"@id": "urn:dataset:0000-00-0"}
    }))
    .await;
    assert!(
        out.contains(r#""2026-12-31T23:59:59Z"^^<http://www.w3.org/2001/XMLSchema#dateTime>"#),
        "got: {out}"
    );
    assert!(
        out.contains(r#""2"^^<http://www.w3.org/2001/XMLSchema#integer>"#),
        "got: {out}"
    );
    assert!(!out.contains("<xsd:"), "got: {out}");
}

const MSG: &str = "https://example.org/Message";
const ADDR: &str = "https://example.org/address";
const PORT: &str = "https://example.org/port";
const PID: &str = "https://example.org/pid";

fn graph_form() -> Value {
    json!([
        {"@id": "_:msg", "@type": [MSG],
         PID: [{"@id": "urn:uuid:cc"}],
         ADDR: [{"@id": "_:a"}]},
        {"@id": "_:a", PORT: [{"@value": "8080"}]}
    ])
}

#[test]
fn finds_the_node_by_type() {
    let g = graph_form();
    let doc = ExpandedDoc::new(&g).unwrap();
    assert_eq!(doc.nodes_of_type(MSG).count(), 1);
    assert_eq!(doc.nodes_of_type("https://example.org/Other").count(), 0);
}

#[test]
fn follows_a_bare_reference_to_the_node_holding_the_data() {
    let g = graph_form();
    let doc = ExpandedDoc::new(&g).unwrap();
    let msg = doc.nodes_of_type(MSG).next().unwrap();
    let addr = msg.object(ADDR).expect("address resolves");
    assert_eq!(addr.iri_or_literal(PORT), Some("8080"));
}

#[test]
fn reads_a_pid_whether_it_is_an_id_or_a_literal() {
    let g = graph_form();
    let doc = ExpandedDoc::new(&g).unwrap();
    let msg = doc.nodes_of_type(MSG).next().unwrap();
    assert_eq!(msg.iri_or_literal(PID), Some("urn:uuid:cc"));

    let literal = json!([{"@type": [MSG], PID: [{"@value": "urn:uuid:cc"}]}]);
    let doc = ExpandedDoc::new(&literal).unwrap();
    let msg = doc.nodes_of_type(MSG).next().unwrap();
    assert_eq!(msg.iri_or_literal(PID), Some("urn:uuid:cc"));
}

#[test]
fn counts_the_values_of_a_term() {
    let doubled = json!([{
        "@type": [MSG],
        PID: [{"@value": "urn:uuid:a"}, {"@value": "urn:uuid:b"}]
    }]);
    let doc = ExpandedDoc::new(&doubled).unwrap();
    let msg = doc.nodes_of_type(MSG).next().unwrap();
    assert_eq!(msg.count(PID), 2);
    assert_eq!(msg.iri_or_literal(PID), Some("urn:uuid:a"));
}

#[test]
fn a_non_array_expansion_is_rejected() {
    assert!(ExpandedDoc::new(&json!({"@type": [MSG]})).is_none());
}
