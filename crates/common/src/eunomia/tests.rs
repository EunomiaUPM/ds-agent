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

//! The eunomia context through its engine: what an RPC message and an RPC result expand to.

use serde_json::json;

use crate::eunomia::{EunomiaProfile, EUNOMIA_CATALOG_CONTEXT_URL, EUNOMIA_CONTEXT_URL};
use crate::rdf::ExpandedDoc;

const DSPACE: &str = "https://w3id.org/dspace/2025/1/";

fn sample_request() -> serde_json::Value {
    json!({
        "@context": EUNOMIA_CONTEXT_URL,
        "@type": "RpcTransferRequestMessage",
        "associatedAgentPeer": "did:web:provider.example",
        "agreementId": "urn:uuid:e8dc8655-44c2-46ef-b701-4cffdc2faa44",
        "format": "HttpData-PULL",
        "providerAddress": "https://provider.example/dsp/current",
        "dataAddress": {
            "endpointType": "https://w3id.org/idsa/v4.1/HTTP",
            "endpointProperties": [{"name": "authorization", "value": "TOKEN"}]
        }
    })
}

/// The engine resolves the eunomia context without going to the network.
#[tokio::test]
async fn eunomia_context_is_embedded() {
    let engine = EunomiaProfile::shared();
    assert!(engine.loader().is_cached(EUNOMIA_CONTEXT_URL).await);
}

/// A request expands to the eunomia type, eunomia RPC terms and DSP terms for DSP concepts.
#[tokio::test]
async fn transfer_request_expands_to_eunomia_and_dsp_terms() {
    let expansion = EunomiaProfile::shared()
        .expand(&sample_request())
        .await
        .unwrap();
    let doc = ExpandedDoc::new(&expansion.expanded).unwrap();
    let node = doc.nodes().next().unwrap();

    assert!(node.has_type(&EunomiaProfile::term("RpcTransferRequestMessage")));
    assert_eq!(
        node.iri_or_literal(&EunomiaProfile::term("associatedAgentPeer")),
        Some("did:web:provider.example")
    );
    assert_eq!(
        node.iri_or_literal(&format!("{DSPACE}agreementId")),
        Some("urn:uuid:e8dc8655-44c2-46ef-b701-4cffdc2faa44")
    );
    assert_eq!(
        node.iri_or_literal("http://purl.org/dc/terms/format"),
        Some("HttpData-PULL")
    );
    let address = node.object(&format!("{DSPACE}dataAddress")).unwrap();
    let property = address
        .objects(&format!("{DSPACE}endpointProperties"))
        .next()
        .unwrap();
    assert_eq!(
        property.iri_or_literal(&format!("{DSPACE}name")),
        Some("authorization")
    );
}

/// Without the eunomia context the type is not a eunomia type and the terms are dropped.
#[tokio::test]
async fn body_without_the_context_has_no_eunomia_type() {
    let mut body = sample_request();
    body.as_object_mut().unwrap().remove("@context");
    let expansion = EunomiaProfile::shared().expand(&body).await.unwrap();
    let doc = ExpandedDoc::new(&expansion.expanded).unwrap();

    assert!(doc
        .nodes()
        .all(|n| !n.has_type(&EunomiaProfile::term("RpcTransferRequestMessage"))));
}

/// A result keeps the request, the peer's answer and the process as JSON literals.
#[tokio::test]
async fn transfer_result_keeps_json_members() {
    let result = json!({
        "@context": EUNOMIA_CONTEXT_URL,
        "@type": "RpcTransferResult",
        "request": sample_request(),
        "response": {"@type": "TransferProcess", "state": "REQUESTED"},
        "transferAgentModel": {"id": "urn:uuid:1", "state": "REQUESTED"}
    });
    let expansion = EunomiaProfile::shared().expand(&result).await.unwrap();
    let doc = ExpandedDoc::new(&expansion.expanded).unwrap();
    let node = doc.nodes().next().unwrap();

    let model = node
        .value(&EunomiaProfile::term("transferAgentModel"))
        .unwrap();
    assert_eq!(model["@type"], "@json");
    assert_eq!(model["@value"]["state"], "REQUESTED");
}

/// A catalog dataset expands with language-tagged literals and its catalog as a reverse link.
#[tokio::test]
async fn catalog_dataset_expands_with_languages_and_reverse_membership() {
    let dataset = json!({
        "@context": EUNOMIA_CATALOG_CONTEXT_URL,
        "@id": "urn:dataset:1",
        "@type": "Dataset",
        "title": {"es": "Lecturas", "en": "Readings"},
        "keyword": {"en": ["energy", "meter"]},
        "theme": ["http://publications.europa.eu/resource/authority/data-theme/ENER"],
        "inCatalog": "urn:catalog:1"
    });
    let expansion = EunomiaProfile::shared().expand(&dataset).await.unwrap();
    let node = &expansion.expanded[0];

    assert_eq!(node["@type"][0], "http://www.w3.org/ns/dcat#Dataset");
    let titles = node["http://purl.org/dc/terms/title"].as_array().unwrap();
    assert_eq!(titles.len(), 2);
    assert!(titles.iter().any(|t| t["@language"] == "es" && t["@value"] == "Lecturas"));
    assert_eq!(node["http://www.w3.org/ns/dcat#keyword"].as_array().unwrap().len(), 2);
    assert_eq!(
        node["@reverse"]["http://www.w3.org/ns/dcat#dataset"][0]["@id"],
        "urn:catalog:1"
    );
}

/// `inCatalog` means a different link on each class, as its type-scoped context says.
#[tokio::test]
async fn catalog_membership_term_depends_on_the_type() {
    let service = json!({
        "@context": EUNOMIA_CATALOG_CONTEXT_URL,
        "@type": "DataService",
        "endpointURL": "https://provider.example/dsp",
        "inCatalog": "urn:catalog:1"
    });
    let expansion = EunomiaProfile::shared().expand(&service).await.unwrap();
    let node = &expansion.expanded[0];

    assert_eq!(
        node["@reverse"]["http://www.w3.org/ns/dcat#service"][0]["@id"],
        "urn:catalog:1"
    );
    assert_eq!(
        node["http://www.w3.org/ns/dcat#endpointURL"][0]["@id"],
        "https://provider.example/dsp"
    );
}
