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

//! The eunomia RPC body through its stages (raw, RDF, typed): fields, data address, and
//! bodies that are not eunomia JSON-LD.

use serde_json::json;
use serde_json::Value;
use transfer_agent::protocols::dsp::entities::context_rpc::{
    TransferRpcBody, TransferRpcRaw, TransferRpcRdf, TransferRpcTyped,
};
use transfer_agent::protocols::dsp::entities::message_types::TransferDSPMessageType;
use ymir::errors::Outcome;
use transfer_agent::protocols::dsp::entities::message_types::TransferDSPMessageType::{
    TransferCompletionMessage, TransferRequestMessage, TransferStartMessage,
    TransferTerminationMessage,
};

const EUNOMIA: &str = "https://w3id.org/eunomia/v1/context.jsonld";

async fn read(route: &TransferDSPMessageType, json: Value) -> Outcome<TransferRpcTyped> {
    TransferRpcTyped::from_rdf(TransferRpcRdf::from_raw(TransferRpcRaw::new(route, json)).await?)
}

/// A request body is read into its fields, data address included.
#[tokio::test]
async fn a_request_body_is_read_through_rdf() {
    let body = json!({
        "@context": EUNOMIA,
        "@type": "RpcTransferRequestMessage",
        "associatedAgentPeer": "did:web:provider.example",
        "agreementId": "urn:uuid:e8dc8655-44c2-46ef-b701-4cffdc2faa44",
        "format": "HttpData-PUSH",
        "providerAddress": "https://provider.example/dsp/current",
        "dataAddress": {
            "endpointType": "HTTP",
            "endpoint": "https://sink.example/data",
            "endpointProperties": [{"name": "authorization", "value": "TOKEN"}]
        }
    });
    let command = read(&TransferRequestMessage, body.clone())
        .await
        .unwrap();

    assert_eq!(command.body.associated_agent_peer.as_deref(), Some("did:web:provider.example"));
    assert_eq!(command.body.format.as_deref(), Some("HttpData-PUSH"));
    assert_eq!(
        command.body.provider_address.as_deref(),
        Some("https://provider.example/dsp/current")
    );
    let address = command.body.data_address.unwrap();
    assert_eq!(address.endpoint.as_deref(), Some("https://sink.example/data"));
    assert_eq!(address.endpoint_properties.unwrap()[0].name, "authorization");
    assert_eq!(command.rdf.raw.json, body, "the JSON-LD is kept as sent");
}

/// Reasons of a termination come back as a list, code alongside.
#[tokio::test]
async fn a_termination_keeps_code_and_reasons() {
    let body = json!({
        "@context": EUNOMIA,
        "@type": "RpcTransferTerminationMessage",
        "@id": "urn:transfer-process:5c1a7a2e-0000-4000-8000-000000000001",
        "code": "STOPPED",
        "reason": ["no longer needed"]
    });
    let command = read(&TransferTerminationMessage, body)
        .await
        .unwrap();
    assert_eq!(command.body.code.as_deref(), Some("STOPPED"));
    assert_eq!(command.body.reason, Some(vec!["no longer needed".to_string()]));
}

/// Plain JSON without the eunomia context declares no RPC type and is rejected.
#[tokio::test]
async fn plain_json_is_rejected() {
    let body = json!({"@id": "urn:transfer-process:5c1a7a2e-0000-4000-8000-000000000001"});
    assert!(read(&TransferStartMessage, body).await.is_err());
}

/// A DSP message is not an RPC body, even with the right fields.
#[tokio::test]
async fn a_dsp_message_is_rejected() {
    let body = json!({
        "@context": "https://w3id.org/dspace/2025/1/context.jsonld",
        "@type": "TransferStartMessage",
        "providerPid": "urn:uuid:p",
        "consumerPid": "urn:uuid:c"
    });
    assert!(read(&TransferStartMessage, body).await.is_err());
}

/// A body built in process renders JSON-LD that reads back to the same fields.
#[tokio::test]
async fn an_in_process_command_round_trips() {
    let built = TransferRpcTyped::from_body(
        &TransferStartMessage,
        &TransferRpcBody {
            id: Some("urn:transfer-process:5c1a7a2e-0000-4000-8000-000000000001".into()),
            ..TransferRpcBody::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(built.rdf.raw.json["@type"], "RpcTransferStartMessage");
    assert_eq!(built.rdf.raw.json["@id"], "urn:transfer-process:5c1a7a2e-0000-4000-8000-000000000001");
    assert_eq!(built.message, TransferStartMessage);
    assert_eq!(built.process_id(), Some("urn:transfer-process:5c1a7a2e-0000-4000-8000-000000000001"));
}

/// A continuation names its process by `@id`; DSP pids are not RPC terms and are dropped.
#[tokio::test]
async fn a_continuation_names_its_process_by_id() {
    let body = json!({
        "@context": EUNOMIA,
        "@type": "RpcTransferCompletionMessage",
        "@id": "urn:transfer-process:5c1a7a2e-0000-4000-8000-000000000001",
        "providerPid": "urn:uuid:p"
    });
    let command = read(&TransferCompletionMessage, body).await.unwrap();
    assert_eq!(command.process_id(), Some("urn:transfer-process:5c1a7a2e-0000-4000-8000-000000000001"));
}
