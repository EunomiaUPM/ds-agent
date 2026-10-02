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

//! DspTransfer: typed DSP transfer fields extracted from the expanded RDF.

use common::rdf::ExpandedDoc;
use transfer_agent::protocols::dsp::entities::message_types::TransferDSPMessageType;
use transfer_agent::protocols::dsp::entities::protocol_fields::TransferProtocolFields;

use common::dsp_common::rdf::DspProfile;
use serde_json::json;
use transfer_agent::protocols::dsp::entities::rdf_extractor_dsp::*;

async fn fields_of(
    message: serde_json::Value,
    kind: TransferDSPMessageType,
) -> TransferProtocolFields {
    let expansion = DspProfile::shared()
        .expand(&message)
        .await
        .expect("expansion");
    let doc = ExpandedDoc::new(&expansion.expanded).expect("array");
    let type_iri = DspTransfer::type_iri(&kind);
    let node = doc.root_message_node(&type_iri, &kind).expect("node");
    DspTransfer::extract(&node).expect("fields")
}

fn request(consumer_pid: serde_json::Value) -> serde_json::Value {
    json!({
        "@context": "https://w3id.org/dspace/2025/1/context.jsonld",
        "@type": "TransferRequestMessage",
        "consumerPid": consumer_pid,
        "agreementId": "urn:uuid:ag",
        "callbackAddress": "https://example.com/callback",
        "format": "example:HTTP_PUSH",
        "dataAddress": {
            "@type": "DataAddress",
            "endpointType": "https://w3id.org/idsa/v4.1/HTTP",
            "endpoint": "http://example.com",
            "endpointProperties": [
                {"@type": "EndpointProperty", "name": "authorization", "value": "TOKEN-ABCDEFG"},
                {"@type": "EndpointProperty", "name": "authType", "value": "bearer"}
            ]
        }
    })
}

/// Every field of a transfer request is read.
#[tokio::test]
async fn reads_every_request_field() {
    let f = fields_of(
        request(json!("urn:uuid:cc")),
        TransferDSPMessageType::TransferRequestMessage,
    )
    .await;
    assert_eq!(f.consumer_pid.as_deref(), Some("urn:uuid:cc"));
    assert_eq!(f.agreement_id.as_deref(), Some("urn:uuid:ag"));
    assert_eq!(
        f.callback_address.as_deref(),
        Some("https://example.com/callback")
    );
    assert_eq!(
        f.format.as_deref(),
        Some("example:HTTP_PUSH"),
        "format is dct:, not dspace:"
    );
    assert!(f.provider_pid.is_none(), "a request carries no providerPid");

    let a = f.data_address.unwrap();
    assert_eq!(a.endpoint.as_deref(), Some("http://example.com"));
    let mut props: Vec<_> = a
        .endpoint_properties
        .iter()
        .map(|p| (p.name.clone(), p.value.clone()))
        .collect();
    props.sort();
    assert_eq!(props[0].0, "authType");
}

/// `{"@id": …}` is the same message as the bare string and hashes alike, so it
/// must read alike.
#[tokio::test]
async fn a_pid_reads_the_same_as_an_id_or_a_string() {
    let as_id = fields_of(
        request(json!({"@id": "urn:uuid:cc"})),
        TransferDSPMessageType::TransferRequestMessage,
    )
    .await;
    assert_eq!(as_id.consumer_pid.as_deref(), Some("urn:uuid:cc"));
}

/// `code` and `reason` are only in scope for suspension and termination, and
/// `reason` is `@container: @set`.
#[tokio::test]
async fn termination_reads_code_and_every_reason() {
    let f = fields_of(
        json!({
            "@context": "https://w3id.org/dspace/2025/1/context.jsonld",
            "@type": "TransferTerminationMessage",
            "consumerPid": "urn:uuid:cc",
            "providerPid": "urn:uuid:pp",
            "code": "99",
            "reason": ["Policy violation", "Agreement expired"]
        }),
        TransferDSPMessageType::TransferTerminationMessage,
    )
    .await;
    assert_eq!(f.code.as_deref(), Some("99"));
    let mut reasons = f.reason.clone();
    reasons.sort();
    assert_eq!(reasons, vec!["Agreement expired", "Policy violation"]);
}

/// A data address without endpoint type is malformed.
#[tokio::test]
async fn a_data_address_without_an_endpoint_type_is_malformed() {
    let expansion = DspProfile::shared()
        .expand(&json!({
            "@context": "https://w3id.org/dspace/2025/1/context.jsonld",
            "@type": "TransferRequestMessage",
            "dataAddress": {"@type": "DataAddress", "endpoint": "http://example.com"}
        }))
        .await
        .unwrap();
    let doc = ExpandedDoc::new(&expansion.expanded).unwrap();
    let kind = TransferDSPMessageType::TransferRequestMessage;
    let type_iri = DspTransfer::type_iri(&kind);
    let node = doc.root_message_node(&type_iri, &kind).unwrap();
    assert!(DspTransfer::extract(&node).is_err());
}
