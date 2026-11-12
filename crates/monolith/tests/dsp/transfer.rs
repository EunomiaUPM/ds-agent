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

use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use common::eunomia::{
    EUNOMIA_CATALOG_CONTEXT_URL, EUNOMIA_CONNECTOR_CONTEXT_URL, EUNOMIA_CONTEXT_URL,
};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use uuid::Uuid;
use ymir::services::client::{ClientService, ClientTrait};
use ymir::types::http::HttpBody;
use ymir::utils::{bearer_headers, http_client};

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

    let steps: [(&Participant, &str, &str, &str); 6] = [
        (provider, "setup-start", "RpcTransferStartMessage", "STARTED"),
        // Option A: Provider suspends -> Consumer resume fails -> Provider resumes
        (provider, "setup-suspension", "RpcTransferSuspensionMessage", "SUSPENDED"),
        (provider, "setup-start", "RpcTransferStartMessage", "STARTED"),
        // Option B: Consumer suspends -> Provider resume fails -> Consumer resumes
        (consumer, "setup-suspension", "RpcTransferSuspensionMessage", "SUSPENDED"),
        (consumer, "setup-start", "RpcTransferStartMessage", "STARTED"),
        (consumer, "setup-termination", "RpcTransferTerminationMessage", "TERMINATED"),
    ];
    for (idx, (who, step, rpc_type, expected)) in steps.into_iter().enumerate() {
        // Before Option A resume (idx == 2), Consumer attempting setup-start must fail.
        if idx == 2 {
            let (status, err) = consumer
                .post_dsp(
                    &format!("{TRANSFERS}/rpc/setup-start"),
                    "owner",
                    &json!({
                        "@context": EUNOMIA_CONTEXT_URL,
                        "@type": "RpcTransferStartMessage",
                        "@id": &mine,
                    }),
                )
                .await;
            assert!(status >= 400, "consumer must not resume provider suspension: {status} {err}");
        }
        // Before Option B resume (idx == 4), Provider attempting setup-start must fail.
        if idx == 4 {
            let (status, err) = provider
                .post_dsp(
                    &format!("{TRANSFERS}/rpc/setup-start"),
                    "owner",
                    &json!({
                        "@context": EUNOMIA_CONTEXT_URL,
                        "@type": "RpcTransferStartMessage",
                        "@id": &theirs,
                    }),
                )
                .await;
            assert!(status >= 400, "provider must not resume consumer suspension: {status} {err}");
        }
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

/// Polls a participant's transfer process over DSP until it reaches `expected` (or times out).
async fn wait_for_dsp_state(
    participant: &Participant,
    token: &str,
    pid: &str,
    expected: &str,
) -> Value {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let (_, process) = participant
                .get_dsp(&format!("{TRANSFERS}/{pid}"), token)
                .await;
            if process["state"] == expected {
                return process;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("timed out waiting for {pid} to reach {expected}"))
}

/// Issues a single `GET` (no client-side retries) against a dataplane proxy URL, returning
/// `(http_status, body_text)` across `2xx`, `4xx`, and `5xx` responses.
async fn proxy_get(url: &str, token: Option<&str>) -> (u16, String) {
    let client = ClientService::builder().build();
    let headers = token.map(|t| bearer_headers(t).unwrap());
    match client.get(url, headers).await {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            (status, text)
        }
        Err(ymir::errors::Errors::PetitionError { ctx, reason, .. }) => {
            let status = ctx.http_code.map(|c| c.as_u16()).unwrap_or(502);
            (status, reason)
        }
        Err(other) => panic!("unexpected GET error on {url}: {other:?}"),
    }
}

/// Extracts `(proxy_url, optional_ingress_token)` from a participant's local dataplane transfer.
async fn local_proxy_endpoint(
    participant: &Participant,
    process_id: &str,
) -> (String, Option<String>) {
    let dp = participant
        .get(&format!(
            "/api/v1/transfer-agent/dataplane/dataplane-processes/transfer-process/{process_id}"
        ))
        .await;
    let listener = &dp["ingressConfig"]["HttpListener"];
    let path = listener["path"]
        .as_str()
        .unwrap_or_else(|| panic!("missing HttpListener.path in {dp}"));
    let token = listener["token"].as_str().map(ToString::to_string);
    (participant.url(path), token)
}

/// E2E 1: A `PULL` + `FINITE` transfer streams the payload once from the provider's backend
/// through both proxies, automatically transitions both participants to `COMPLETED` via
/// `DataplaneSignal::Completed`, and rejects any subsequent pull attempt.
#[tokio::test]
#[ignore = "needs DATABASE_URL and REDIS_URL"]
async fn pull_finite_transfer_completes_both_participants_after_single_fetch() {
    assert!(
        std::env::var("REDIS_URL").is_ok(),
        "transfer needs REDIS_URL for its dataplane"
    );
    let backend_hits = Arc::new(Mutex::new(0u32));
    let hits_clone = backend_hits.clone();
    let backend_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let backend_addr = backend_listener.local_addr().unwrap();
    let backend_router = Router::new().route(
        "/weather",
        get(move || {
            let hits = hits_clone.clone();
            async move {
                *hits.lock().unwrap() += 1;
                Json(json!({ "temperature": 22.5, "unit": "C" }))
            }
        }),
    );
    let backend_task = tokio::spawn(async move {
        let _ = axum::serve(backend_listener, backend_router).await;
    });

    let ds = Dataspace::start().await;
    let negotiated = negotiate_to_finalized(&ds).await;
    let (provider, consumer) = (&ds.provider, &ds.consumer);

    provider
        .post(
            "/api/v1/connector/templates",
            json!({
                "@context": EUNOMIA_CONNECTOR_CONTEXT_URL,
                "@type": "ConnectorTemplate",
                "name": "weather-pull-finite",
                "version": "1.0",
                "authentication": {"type": "NO_AUTH"},
                "interaction": {
                    "mode": "PULL",
                    "lifetime": "FINITE",
                    "dataAccess": {
                        "protocol": "HTTP",
                        "urlTemplate": "{{__ACCESS_URL__}}",
                        "method": ["GET"]
                    }
                },
                "parameters": [
                    {"paramType": "STRING", "name": "ACCESS_URL", "title": "Access URL", "required": true}
                ]
            }),
        )
        .await;
    provider
        .post(
            "/api/v1/connector/instances",
            json!({
                "@context": EUNOMIA_CONNECTOR_CONTEXT_URL,
                "@type": "ConnectorInstance",
                "templateName": "weather-pull-finite",
                "templateVersion": "1.0",
                "distributionId": negotiated.offer.distribution_id,
                "parameters": {
                    "ACCESS_URL": format!("http://{backend_addr}/weather")
                }
            }),
        )
        .await;

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
    let model = &requested["transferAgentModel"];
    let consumer_id = model["id"].as_str().unwrap().to_string();
    let consumer_pid = model["correlation"]["consumerPid"]
        .as_str()
        .unwrap()
        .to_string();
    let provider_pid = model["correlation"]["providerPid"]
        .as_str()
        .unwrap()
        .to_string();
    let provider_id = process_id_by_provider_pid(provider, &provider_pid).await;

    provider
        .post(
            &format!("{TRANSFERS}/rpc/setup-start"),
            json!({
                "@context": EUNOMIA_CONTEXT_URL,
                "@type": "RpcTransferStartMessage",
                "@id": provider_id,
            }),
        )
        .await;

    let (consumer_proxy_url, consumer_token) = local_proxy_endpoint(consumer, &consumer_id).await;

    // 1st pull through Consumer's dataplane proxy -> Provider's proxy -> Mock backend: 200 OK.
    let (status1, text1) = proxy_get(&consumer_proxy_url, consumer_token.as_deref()).await;
    assert_eq!(status1, 200, "{text1}");
    let payload: Value = serde_json::from_str(&text1).unwrap();
    assert_eq!(payload["temperature"], json!(22.5));

    // Both Provider and Consumer must automatically reach COMPLETED via DataplaneSignal::Completed.
    wait_for_dsp_state(provider, TO_PROVIDER, &provider_pid, "COMPLETED").await;
    wait_for_dsp_state(consumer, TO_CONSUMER, &consumer_pid, "COMPLETED").await;

    // A 2nd pull must be rejected (403 Forbidden once dataplane is Terminated, or 410 Gone during race),
    // and the backend must only have been hit once.
    let (status2, text2) = proxy_get(&consumer_proxy_url, consumer_token.as_deref()).await;
    assert!(
        matches!(status2, 403 | 410),
        "expected 403 or 410 on second pull, got {status2}: {text2}"
    );
    assert_eq!(*backend_hits.lock().unwrap(), 1);
    backend_task.abort();
}

#[derive(Clone, Default)]
struct PushProducerState {
    callback_url: Arc<Mutex<Option<String>>>,
    unsubscribed_id: Arc<Mutex<Option<String>>>,
}

/// E2E 2: A `PUSH` + `FINITE` transfer subscribes on the backend producer, forwards a single
/// webhook event to the consumer's sink, automatically completes both participants via
/// `DataplaneSignal::Completed`, and invokes the producer's `unsubscribe` hook on teardown.
#[tokio::test]
#[ignore = "needs DATABASE_URL and REDIS_URL"]
async fn push_finite_transfer_completes_and_unsubscribes_after_single_webhook() {
    assert!(
        std::env::var("REDIS_URL").is_ok(),
        "transfer needs REDIS_URL for its dataplane"
    );
    let producer_state = PushProducerState::default();
    let producer_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let producer_addr = producer_listener.local_addr().unwrap();
    let producer_router = Router::new()
        .route(
            "/hooks",
            post(
                |State(st): State<PushProducerState>, Json(body): Json<Value>| async move {
                    *st.callback_url.lock().unwrap() =
                        body["callback"].as_str().map(ToString::to_string);
                    (StatusCode::CREATED, Json(json!({ "id": "sub-e2e-1" })))
                },
            ),
        )
        .route(
            "/hooks/{id}",
            delete(
                |State(st): State<PushProducerState>, Path(id): Path<String>| async move {
                    *st.unsubscribed_id.lock().unwrap() = Some(id);
                    StatusCode::NO_CONTENT
                },
            ),
        )
        .with_state(producer_state.clone());
    let producer_task = tokio::spawn(async move {
        let _ = axum::serve(producer_listener, producer_router).await;
    });

    let sink_events: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));
    let sink_captured = sink_events.clone();
    let sink_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let sink_addr = sink_listener.local_addr().unwrap();
    let sink_router = Router::new().route(
        "/ingest",
        post(move |Json(body): Json<Value>| {
            let sink_captured = sink_captured.clone();
            async move {
                sink_captured.lock().unwrap().push(body);
                StatusCode::OK
            }
        }),
    );
    let sink_task = tokio::spawn(async move {
        let _ = axum::serve(sink_listener, sink_router).await;
    });

    let ds = Dataspace::start().await;
    let negotiated = negotiate_to_finalized(&ds).await;
    let (provider, consumer) = (&ds.provider, &ds.consumer);

    // Add an `http+push` distribution to the negotiated dataset and bind a FINITE PUSH connector.
    let service = provider
        .get("/api/v1/catalog-agent/data-services/main")
        .await;
    let push_distribution = provider
        .post(
            "/api/v1/catalog-agent/distributions",
            json!({
                "@context": EUNOMIA_CATALOG_CONTEXT_URL,
                "@type": "Distribution",
                "title": {"en": "Weather Webhook"},
                "accessService": [service["@id"]],
                "ofDataset": negotiated.offer.dataset_id,
                "format": "http+push"
            }),
        )
        .await;

    let callback_body_template = format!(
        r#"{{"callback":"{}{{{{__RUNTIME_INGRESS__}}}}"}}"#,
        provider.base_url
    );
    let unsubscribe_url_template = format!(
        "http://{producer_addr}/hooks/{{{{__RUNTIME_JSON_{{.subscription.id}}__}}}}"
    );

    provider
        .post(
            "/api/v1/connector/templates",
            json!({
                "@context": EUNOMIA_CONNECTOR_CONTEXT_URL,
                "@type": "ConnectorTemplate",
                "name": "weather-push-finite",
                "version": "1.0",
                "authentication": {"type": "NO_AUTH"},
                "interaction": {
                    "mode": "PUSH",
                    "lifetime": "FINITE",
                    "subscribe": {
                        "protocol": "HTTP",
                        "urlTemplate": format!("http://{producer_addr}/hooks"),
                        "method": ["POST"],
                        "bodyTemplate": callback_body_template
                    },
                    "unsubscribe": {
                        "protocol": "HTTP",
                        "urlTemplate": unsubscribe_url_template,
                        "method": ["DELETE"]
                    }
                },
                "parameters": []
            }),
        )
        .await;
    provider
        .post(
            "/api/v1/connector/instances",
            json!({
                "@context": EUNOMIA_CONNECTOR_CONTEXT_URL,
                "@type": "ConnectorInstance",
                "templateName": "weather-push-finite",
                "templateVersion": "1.0",
                "distributionId": push_distribution["@id"],
                "parameters": {}
            }),
        )
        .await;

    let requested = consumer
        .post(
            &format!("{TRANSFERS}/rpc/setup-request"),
            json!({
                "@context": EUNOMIA_CONTEXT_URL,
                "@type": "RpcTransferRequestMessage",
                "associatedAgentPeer": PROVIDER_DID,
                "agreementId": negotiated.agreement["id"],
                "format": "http+push",
                "providerAddress": provider.url("/dsp/current"),
                "dataAddress": {
                    "@type": "DataAddress",
                    "endpointType": "https://w3id.org/idsa/v4.1/HTTP",
                    "endpoint": format!("http://{sink_addr}/ingest")
                }
            }),
        )
        .await;
    let model = &requested["transferAgentModel"];
    let consumer_pid = model["correlation"]["consumerPid"]
        .as_str()
        .unwrap()
        .to_string();
    let provider_pid = model["correlation"]["providerPid"]
        .as_str()
        .unwrap()
        .to_string();
    let provider_id = process_id_by_provider_pid(provider, &provider_pid).await;

    provider
        .post(
            &format!("{TRANSFERS}/rpc/setup-start"),
            json!({
                "@context": EUNOMIA_CONTEXT_URL,
                "@type": "RpcTransferStartMessage",
                "@id": provider_id,
            }),
        )
        .await;

    let callback_url = producer_state
        .callback_url
        .lock()
        .unwrap()
        .clone()
        .expect("producer should have received subscribe callback URL");

    // Producer pushes 1 event into Provider's runtime ingress -> Consumer proxy -> Consumer sink.
    let push_resp = http_client()
        .post(
            &callback_url,
            None,
            HttpBody::Json(json!({ "alert": "storm", "seq": 1 })),
        )
        .await
        .expect("push event to provider ingress");
    assert_eq!(push_resp.status(), StatusCode::OK);

    // Both participants reach COMPLETED and the provider unsubscribes from the backend producer.
    wait_for_dsp_state(provider, TO_PROVIDER, &provider_pid, "COMPLETED").await;
    wait_for_dsp_state(consumer, TO_CONSUMER, &consumer_pid, "COMPLETED").await;

    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if producer_state.unsubscribed_id.lock().unwrap().is_some() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("producer unsubscribe hook should be invoked upon FINITE push completion");

    assert_eq!(
        producer_state.unsubscribed_id.lock().unwrap().as_deref(),
        Some("sub-e2e-1")
    );
    assert_eq!(sink_events.lock().unwrap().len(), 1);
    assert_eq!(sink_events.lock().unwrap()[0]["alert"], json!("storm"));

    producer_task.abort();
    sink_task.abort();
}

/// E2E 3: Transient upstream `503` responses stay in `STARTED` within `maxRetries: 1`, and once
/// consecutive failures exceed `maxRetries`, `DataplaneSignal::Failed` automatically terminates
/// both participants over DSP with `DATAPLANE_MAX_RETRIES_EXCEEDED`.
#[tokio::test]
#[ignore = "needs DATABASE_URL and REDIS_URL"]
async fn dataplane_egress_failure_terminates_both_participants() {
    assert!(
        std::env::var("REDIS_URL").is_ok(),
        "transfer needs REDIS_URL for its dataplane"
    );
    let backend_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let backend_addr = backend_listener.local_addr().unwrap();
    let backend_router = Router::new().route(
        "/unstable",
        get(|| async { (StatusCode::SERVICE_UNAVAILABLE, "upstream overloaded") }),
    );
    let backend_task = tokio::spawn(async move {
        let _ = axum::serve(backend_listener, backend_router).await;
    });

    let ds = Dataspace::start().await;
    let negotiated = negotiate_to_finalized(&ds).await;
    let (provider, consumer) = (&ds.provider, &ds.consumer);

    provider
        .post(
            "/api/v1/connector/templates",
            json!({
                "@context": EUNOMIA_CONNECTOR_CONTEXT_URL,
                "@type": "ConnectorTemplate",
                "name": "unstable-pull",
                "version": "1.0",
                "authentication": {"type": "NO_AUTH"},
                "interaction": {
                    "mode": "PULL",
                    "lifetime": "INFINITE",
                    "maxRetries": 1,
                    "dataAccess": {
                        "protocol": "HTTP",
                        "urlTemplate": "{{__ACCESS_URL__}}",
                        "method": ["GET"]
                    }
                },
                "parameters": [
                    {"paramType": "STRING", "name": "ACCESS_URL", "title": "Access URL", "required": true}
                ]
            }),
        )
        .await;
    provider
        .post(
            "/api/v1/connector/instances",
            json!({
                "@context": EUNOMIA_CONNECTOR_CONTEXT_URL,
                "@type": "ConnectorInstance",
                "templateName": "unstable-pull",
                "templateVersion": "1.0",
                "distributionId": negotiated.offer.distribution_id,
                "parameters": {
                    "ACCESS_URL": format!("http://{backend_addr}/unstable")
                }
            }),
        )
        .await;

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
    let model = &requested["transferAgentModel"];
    let consumer_id = model["id"].as_str().unwrap().to_string();
    let consumer_pid = model["correlation"]["consumerPid"]
        .as_str()
        .unwrap()
        .to_string();
    let provider_pid = model["correlation"]["providerPid"]
        .as_str()
        .unwrap()
        .to_string();
    let provider_id = process_id_by_provider_pid(provider, &provider_pid).await;

    provider
        .post(
            &format!("{TRANSFERS}/rpc/setup-start"),
            json!({
                "@context": EUNOMIA_CONTEXT_URL,
                "@type": "RpcTransferStartMessage",
                "@id": provider_id,
            }),
        )
        .await;

    let (consumer_proxy_url, consumer_token) = local_proxy_endpoint(consumer, &consumer_id).await;

    // Failure #1 (1 <= maxRetries=1): returns 503, transfer stays STARTED on both sides.
    let (status1, _) = proxy_get(&consumer_proxy_url, consumer_token.as_deref()).await;
    assert_eq!(status1, 503);
    tokio::time::sleep(Duration::from_millis(80)).await;
    assert_eq!(
        state_of(provider, TO_PROVIDER, &provider_pid).await,
        "STARTED"
    );

    // Failure #2 (2 > maxRetries=1): emits DataplaneSignal::Failed -> TERMINATED on both sides!
    let (status2, _) = proxy_get(&consumer_proxy_url, consumer_token.as_deref()).await;
    assert_eq!(status2, 503);

    wait_for_dsp_state(provider, TO_PROVIDER, &provider_pid, "TERMINATED").await;
    wait_for_dsp_state(consumer, TO_CONSUMER, &consumer_pid, "TERMINATED").await;

    // Verify stateMetadata.code on both sides records DATAPLANE_MAX_RETRIES_EXCEEDED.
    let provider_proc = provider
        .get(&format!(
            "/api/v1/transfer-agent-ref/transfer-processes/{}",
            provider_id.as_str().unwrap()
        ))
        .await;
    assert_eq!(
        provider_proc["stateMetadata"]["code"],
        json!("DATAPLANE_MAX_RETRIES_EXCEEDED")
    );
    let consumer_proc = consumer
        .get(&format!(
            "/api/v1/transfer-agent-ref/transfer-processes/{consumer_id}"
        ))
        .await;
    assert_eq!(
        consumer_proc["stateMetadata"]["code"],
        json!("DATAPLANE_MAX_RETRIES_EXCEEDED")
    );

    backend_task.abort();
}
