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

//! JSON-LD expansion, canonicalization and hashing with the DSP and generic engines.

use serde_json::json;

use super::sample_transfer_request;
use crate::dsp_common::rdf::{DSP_CONTEXT_URL, DSP_ODRL_PROFILE_URL};
use crate::rdf::RdfEngine;

async fn canon(v: serde_json::Value) -> String {
    RdfEngine::dsp().canonicalize(&v).await.unwrap()
}

/// The DSP engine has the DSP context cached from the start.
#[tokio::test]
async fn dsp_engine_preloads_canonical_dsp_context() {
    let engine = RdfEngine::dsp();
    assert!(engine.loader().is_cached(DSP_CONTEXT_URL).await);
}

/// The generic engine canonicalizes JSON-LD that has nothing to do with DSP.
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

/// The hash of the sample message is pinned, so any change in canonicalization shows up.
#[tokio::test]
async fn canonical_hash_is_stable() {
    let engine = RdfEngine::dsp();
    let hash = engine.hash(&sample_transfer_request()).await.unwrap();
    assert_eq!(
        hash,
        "2a74c50cad115b2b9e21b3d5c580d7263cc754328e5eb4b159416f71f2ef5b1a"
    );
}

/// Expansion uses full IRIs and types each term as the DSP context says (`@id` or literal).
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

/// `dspace:`-prefixed terms and bare DSP terms expand to the same IRI.
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

/// Canonical form ignores key order but changes with any value.
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

/// `xsd:` resolves even when the message does not declare the prefix.
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

/// ODRL constraint operands keep their `xsd:dateTime` and `xsd:integer` datatypes.
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
