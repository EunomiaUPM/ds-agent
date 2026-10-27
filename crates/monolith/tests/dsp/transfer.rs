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

//! Transfer over DSP after a finalized negotiation: the consumer, as a raw DSP peer, opens a
//! transfer on the provider and terminates it, through the transfer flow end to end.

use common::eunomia::{EUNOMIA_CONNECTOR_CONTEXT_URL, EUNOMIA_CONTEXT_URL};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::negotiation::negotiate_to_finalized;
use crate::support::dataspace::{Dataspace, PROVIDER_DID, TO_CONSUMER, TO_PROVIDER};
use crate::support::participant::Participant;

const TRANSFERS: &str = "/dsp/current/transfers";

/// Gives the provider's distribution a pull connector, as an operator would before offering it.
async fn install_pull_connector(provider: &Participant, distribution_id: &Value) {
    let template = json!({
        "@context": EUNOMIA_CONNECTOR_CONTEXT_URL,
        "@type": "ConnectorTemplate",
        "name": "weather-pull",
        "version": "1.0",
        "authentication": {"type": "NO_AUTH"},
        "interaction": {
            "mode": "PULL",
            "dataAccess": {
                "protocol": "HTTP",
                "urlTemplate": "{{__ACCESS_URL__}}",
                "method": "{{__ALLOWED_METHODS__}}",
                "headers": "{{__ALLOWED_HEADERS__}}",
                "bodyTemplate": ""
            }
        },
        "parameters": [
            {"paramType": "STRING", "name": "ACCESS_URL", "title": "Access URL", "required": true},
            {"paramType": "VEC<STRING>", "name": "ALLOWED_METHODS", "title": "Methods", "required": true},
            {"paramType": "MAP<STRING,STRING>", "name": "ALLOWED_HEADERS", "title": "Headers", "required": true}
        ]
    });
    provider.post("/api/v1/connector/templates", template).await;
    provider
        .post(
            "/api/v1/connector/instances",
            json!({
                "@context": EUNOMIA_CONNECTOR_CONTEXT_URL,
                "@type": "ConnectorInstance",
                "templateName": "weather-pull",
                "templateVersion": "1.0",
                "distributionId": distribution_id,
                "parameters": {
                    "ACCESS_URL": "http://127.0.0.1:9/weather",
                    "ALLOWED_METHODS": ["GET"],
                    "ALLOWED_HEADERS": {}
                }
            }),
        )
        .await;
}

fn request(consumer_pid: &str, agreement_id: &Value, callback: &str) -> Value {
    json!({
        "@context": ["https://w3id.org/dspace/2025/1/context.jsonld"],
        "@type": "TransferRequestMessage",
        "consumerPid": consumer_pid,
        "agreementId": agreement_id,
        "format": "http+pull",
        "callbackAddress": callback
    })
}

fn termination(consumer_pid: &str, provider_pid: &str) -> Value {
    json!({
        "@context": ["https://w3id.org/dspace/2025/1/context.jsonld"],
        "@type": "TransferTerminationMessage",
        "consumerPid": consumer_pid,
        "providerPid": provider_pid,
        "code": "STOPPED",
        "reason": ["integration test done"]
    })
}

/// A request opens a REQUESTED process (201), a retry replays it, a malformed one is rejected
/// with every reason, and the consumer can terminate what it opened.
#[tokio::test]
#[ignore = "needs DATABASE_URL and REDIS_URL"]
async fn transfer_request_opens_a_process_and_the_consumer_terminates_it() {
    assert!(
        std::env::var("REDIS_URL").is_ok(),
        "transfer needs REDIS_URL for its dataplane"
    );
    let ds = Dataspace::start().await;
    let negotiated = negotiate_to_finalized(&ds).await;
    install_pull_connector(&ds.provider, &negotiated.offer.distribution_id).await;

    let consumer_pid = format!("urn:uuid:{}", Uuid::new_v4());
    let callback = ds.consumer.url("/dsp/current");
    let body = request(&consumer_pid, &negotiated.agreement["id"], &callback);
    let (status, ack) = ds
        .provider
        .post_dsp(&format!("{TRANSFERS}/request"), TO_PROVIDER, &body)
        .await;
    assert_eq!(status, 201, "{ack}");
    assert_eq!(ack["@type"], "TransferProcess");
    assert_eq!(ack["state"], "REQUESTED");
    assert_eq!(ack["consumerPid"], consumer_pid);
    let provider_pid = ack["providerPid"]
        .as_str()
        .expect("providerPid")
        .to_string();

    let (status, replay) = ds
        .provider
        .post_dsp(&format!("{TRANSFERS}/request"), TO_PROVIDER, &body)
        .await;
    assert_eq!(
        (status, &replay),
        (201, &ack),
        "a retry replays the first answer"
    );

    let mut broken = body.clone();
    broken["agreementId"] = json!("not-a-urn");
    broken.as_object_mut().unwrap().remove("format");
    let (status, error) = ds
        .provider
        .post_dsp(&format!("{TRANSFERS}/request"), TO_PROVIDER, &broken)
        .await;
    assert_eq!(status, 400, "{error}");
    assert_eq!(error["@type"], "TransferError");
    assert!(
        error["reason"].as_array().is_some_and(|r| r.len() >= 2),
        "{error}"
    );

    let (status, process) = ds
        .provider
        .get_dsp(&format!("{TRANSFERS}/{provider_pid}"), TO_PROVIDER)
        .await;
    assert_eq!(
        (status, process["state"].clone()),
        (200, json!("REQUESTED")),
        "{process}"
    );

    let (status, answer) = ds
        .provider
        .post_dsp(
            &format!("{TRANSFERS}/{provider_pid}/termination"),
            TO_PROVIDER,
            &termination(&consumer_pid, &provider_pid),
        )
        .await;
    assert_eq!(status, 200, "{answer}");
    let (_, process) = ds
        .provider
        .get_dsp(&format!("{TRANSFERS}/{provider_pid}"), TO_PROVIDER)
        .await;
    assert_eq!(process["state"], "TERMINATED", "{process}");

    let unknown = format!("urn:uuid:{}", Uuid::new_v4());
    let (status, _) = ds
        .provider
        .post_dsp(
            &format!("{TRANSFERS}/{unknown}/termination"),
            TO_PROVIDER,
            &termination(&format!("urn:uuid:{}", Uuid::new_v4()), &unknown),
        )
        .await;
    assert_eq!(status, 404, "an unknown process answers 404");
}

/// State a participant's peer reads for `pid` over DSP, with the token that peer presents.
async fn state_of(participant: &Participant, token: &str, pid: &str) -> Value {
    let (_, process) = participant
        .get_dsp(&format!("{TRANSFERS}/{pid}"), token)
        .await;
    process["state"].clone()
}

/// `participant`'s own id of the transfer whose provider pid is `provider_pid`.
async fn process_id_by_provider_pid(participant: &Participant, provider_pid: &str) -> Value {
    let processes = participant
        .get("/api/v1/transfer-agent-ref/transfer-processes")
        .await;
    processes["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["correlation"]["providerPid"] == provider_pid)
        .map(|p| p["id"].clone())
        .unwrap_or_else(|| panic!("no transfer for {provider_pid}: {processes}"))
}

/// Our consumer and provider run a pull transfer over RPC end to end: request, start,
/// suspension, restart and termination, each side moving as the other's messages arrive.
#[tokio::test]
#[ignore = "needs DATABASE_URL and REDIS_URL"]
async fn consumer_and_provider_drive_a_transfer_over_rpc() {
    assert!(
        std::env::var("REDIS_URL").is_ok(),
        "transfer needs REDIS_URL for its dataplane"
    );
    let ds = Dataspace::start().await;
    let negotiated = negotiate_to_finalized(&ds).await;
    install_pull_connector(&ds.provider, &negotiated.offer.distribution_id).await;
    let (provider, consumer) = (&ds.provider, &ds.consumer);

    let requested = consumer
        .post(
            &format!("{TRANSFERS}/rpc/setup-request"),
            json!({
                "@context": EUNOMIA_CONTEXT_URL,
                "@type": "RpcTransferRequestMessage",
                "associatedAgentPeer": PROVIDER_DID,
                "agreementId": negotiated.agreement["id"],
                "format": "http+pull",
                "providerAddress": provider.url("/dsp/current"),
            }),
        )
        .await;
    assert_eq!(requested["@type"], "RpcTransferResult", "{requested}");
    let model = &requested["transferAgentModel"];
    assert_eq!(model["state"], "REQUESTED", "{requested}");
    assert_eq!(
        requested["response"]["@type"], "TransferProcess",
        "{requested}"
    );
    let consumer_pid = model["correlation"]["consumerPid"]
        .as_str()
        .unwrap()
        .to_string();
    let provider_pid = model["correlation"]["providerPid"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(
        state_of(provider, TO_PROVIDER, &provider_pid).await,
        "REQUESTED"
    );
    // Each side names the transfer by its own process id, never by the DSP pids.
    let mine = model["id"].clone();
    let theirs = process_id_by_provider_pid(provider, &provider_pid).await;

    let steps: [(&Participant, &str, &str, &str); 4] = [
        (provider, "setup-start", "RpcTransferStartMessage", "STARTED"),
        (provider, "setup-suspension", "RpcTransferSuspensionMessage", "SUSPENDED"),
        (provider, "setup-start", "RpcTransferStartMessage", "STARTED"),
        (consumer, "setup-termination", "RpcTransferTerminationMessage", "TERMINATED"),
    ];
    for (who, step, rpc_type, expected) in steps {
        let id = if std::ptr::eq(who, provider) { &theirs } else { &mine };
        let body = json!({
            "@context": EUNOMIA_CONTEXT_URL,
            "@type": rpc_type,
            "@id": id,
        });
        let answer = who.post(&format!("{TRANSFERS}/rpc/{step}"), body).await;
        assert_eq!(
            answer["transferAgentModel"]["state"], expected,
            "{step}: {answer}"
        );
        assert_eq!(
            state_of(provider, TO_PROVIDER, &provider_pid).await,
            expected,
            "provider after {step}"
        );
        assert_eq!(
            state_of(consumer, TO_CONSUMER, &consumer_pid).await,
            expected,
            "consumer after {step}"
        );
    }
}
