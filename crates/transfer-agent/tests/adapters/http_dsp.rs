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

//! The DSP transfer routes end to end through the flow runner: real adapter, rules, loader and
//! handlers over an in-memory process store, with the agreement, connector and dataplane mocked.

use crate::support::context::TestContext;
use axum::extract::Request;
use axum::response::Response;
use axum::Router;
use chrono::Utc;
use common::dsp_common::odrl::OdrlAgreement;
use common::facades::catalog::{CatalogDataset, CatalogDistribution, MockCatalogFacadeTrait};
use common::facades::grants_facade::{MockGrantsFacadeTrait, VerifiedPeer};
use common::facades::mates_facade::MockMatesFacadeTrait;
use common::facades::negotiation::MockNegotiationFacadeTrait;
use connector::{
    AuthenticationConfig, ConnectorInstanceDto, ConnectorMetadata, HttpSpec, InteractionConfig,
    MockConnectorInstanceFacadeTrait, ProtocolSpec, PullLifecycle, TemplateVecString,
};
use http::StatusCode;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;
use transfer_agent::data::repo::transfer_process::MockTransferProcessRepoTrait;
use transfer_agent::entities::ids::{MessageId, TransferProcessId};
use transfer_agent::entities::message_envelope::MessageEnvelope;
use transfer_agent::entities::protocol::{
    ProtocolId, ProtocolMessageType, TransferCorrelation, CONSUMER_PID_KEY, PROVIDER_PID_KEY,
};
use transfer_agent::entities::transfer_message::Direction;
use transfer_agent::entities::transfer_process::TransferProcess;
use transfer_agent::protocols::dsp::facades::dataplane_facade::MockDataPlaneFacadeTrait;
use transfer_agent::protocols::dsp::http::dsp::DspRouter;
use transfer_agent::protocols::dsp::setup::DspContext;
use transfer_agent::services::transfer_message::views::TransferMessageView;
use transfer_agent::services::transfer_message::MockTransferMessageServiceTrait;
use transfer_agent::services::transfer_process::views::TransferProcessView;
use transfer_agent::services::transfer_process::MockTransferProcessServiceTrait;
use transfer_agent::setup::context::AppContext;
use urn::Urn;
use ymir::errors::Errors;
use ymir::types::oauth::RolePath;
use ymir::types::participants::Visibility;

const CONSUMER_PID: &str = "urn:uuid:32541fe6-c580-409e-85a8-8a9a32fbe833";

const REQUEST_BODY: &str = r#"{
        "@context": ["https://w3id.org/dspace/2025/1/context.jsonld"],
        "@type": "TransferRequestMessage",
        "consumerPid": "urn:uuid:32541fe6-c580-409e-85a8-8a9a32fbe833",
        "agreementId": "urn:uuid:e8dc8655-44c2-46ef-b701-4cffdc2faa44",
        "format": "example:HTTP_PULL",
        "callbackAddress": "https://example.com/callback"
    }"#;

fn peer(id: &str) -> VerifiedPeer {
    VerifiedPeer {
        participant_id: id.into(),
        role: RolePath::root(),
        visibility: Visibility::Public,
    }
}

type Store = Arc<Mutex<Vec<TransferProcess>>>;

/// Processes the fake repo and service share, so a write is visible to the next read.
fn store() -> Store {
    Arc::new(Mutex::new(Vec::new()))
}

fn find_by_pid(store: &Store, pid: &str) -> Option<TransferProcess> {
    store
        .lock()
        .unwrap()
        .iter()
        .find(|p| {
            let c = p.correlation();
            c.consumer_pid.as_deref() == Some(pid) || c.provider_pid.as_deref() == Some(pid)
        })
        .cloned()
}

fn repo(store: &Store) -> MockTransferProcessRepoTrait {
    let mut repo = MockTransferProcessRepoTrait::new();
    let s = store.clone();
    repo.expect_get_transfer_process_by_key_value()
        .returning(move |_, pid| Ok(find_by_pid(&s, pid)));
    let s = store.clone();
    repo.expect_get_transfer_process_by_id()
        .returning(move |_, id| {
            Ok(s.lock()
                .unwrap()
                .iter()
                .find(|p| p.id().as_urn() == id)
                .cloned())
        });
    repo
}

fn processes(store: &Store) -> MockTransferProcessServiceTrait {
    let mut svc = MockTransferProcessServiceTrait::new();
    let s = store.clone();
    svc.expect_create().returning(move |_, cmd| {
        let ids = cmd.identifiers.clone().unwrap_or_default();
        let mut correlation = TransferCorrelation::empty();
        correlation.consumer_pid = ids.get(CONSUMER_PID_KEY).cloned();
        correlation.provider_pid = ids.get(PROVIDER_PID_KEY).cloned();
        correlation.agreement_id = Some(cmd.agreement_id.clone());
        correlation.callback_address = cmd.callback_address.clone();
        correlation.peer_participant_id = Some(cmd.peer_participant_id.clone());
        let process = TransferProcess::rehydrate(
            cmd.id.clone().unwrap_or_else(TransferProcessId::generate),
            cmd.owner.clone().unwrap(),
            cmd.role,
            Utc::now(),
            Utc::now(),
            0,
            cmd.protocol.clone(),
            cmd.initial_state.clone(),
            cmd.initial_state_metadata.clone(),
            correlation,
            cmd.properties.clone().unwrap_or_default(),
            None,
        );
        s.lock().unwrap().push(process.clone());
        Ok(TransferProcessView::assemble(process, ids))
    });
    let s = store.clone();
    svc.expect_edit().returning(move |_, id, cmd| {
        let mut all = s.lock().unwrap();
        let process = all.iter_mut().find(|p| p.id().as_urn() == id).unwrap();
        process.apply_edit(cmd.clone());
        Ok(TransferProcessView::assemble(
            process.clone(),
            HashMap::new(),
        ))
    });
    svc
}

fn messages() -> MockTransferMessageServiceTrait {
    let mut svc = MockTransferMessageServiceTrait::new();
    svc.expect_create().returning(|_, cmd| {
        Ok(TransferMessageView {
            id: MessageId::generate(),
            transfer_process_id: cmd.transfer_process_id.clone(),
            user_id: "peer".into(),
            user_role: RolePath::root(),
            visibility: Visibility::Public,
            direction: Direction::Inbound,
            protocol: ProtocolId::Dsp2025_1,
            message_type: ProtocolMessageType(cmd.message_type.0.clone()),
            state_transition_from: cmd.state_transition_from.0.to_string(),
            state_transition_to: cmd.state_transition_to.0.to_string(),
            envelope: MessageEnvelope::from_canonical(serde_json::json!({}), None),
            occurred_at: Utc::now(),
        })
    });
    svc
}

fn dataplane() -> MockDataPlaneFacadeTrait {
    let mut dp = MockDataPlaneFacadeTrait::new();
    dp.expect_on_transfer_request_post().returning(|_| Ok(None));
    dp.expect_on_transfer_termination_pre()
        .returning(|_| Ok(None));
    dp.expect_on_transfer_termination_post()
        .returning(|_| Ok(None));
    dp
}

fn connector() -> ConnectorInstanceDto {
    ConnectorInstanceDto {
        id: Urn::from_str("urn:connector-instance:1").unwrap(),
        user_id: "provider".into(),
        metadata: ConnectorMetadata {
            name: None,
            author: None,
            description: None,
            version: None,
            created_at: None,
        },
        authentication_config: AuthenticationConfig::NoAuth,
        interaction: InteractionConfig::Pull(PullLifecycle {
            data_access: ProtocolSpec::Http(HttpSpec {
                url_template: "http://backend/data".into(),
                method: TemplateVecString::Value(vec!["GET".into()]),
                headers: None,
                body_template: None,
            }),
        }),
        distribution_id: Urn::from_str("urn:distribution:1").unwrap(),
    }
}

/// The agent context on `store`, whose auth facade accepts every token as `as_peer`, or
/// rejects them all; no peer is a known mate.
fn context(store: &Store, as_peer: Option<&str>) -> Arc<AppContext> {
    let mut catalog = MockCatalogFacadeTrait::new();
    catalog.expect_get_dataset().returning(|id| {
        Ok(CatalogDataset {
            id: id.to_string(),
            catalog_id: "urn:catalog:1".into(),
            dct_title: None,
            draft: false,
        })
    });
    catalog
        .expect_get_distribution_by_format()
        .returning(|dataset, format| {
            Ok(CatalogDistribution {
                id: "urn:distribution:1".into(),
                dataset_id: dataset.to_string(),
                dct_format: Some(format.into()),
                dcat_access_service: "urn:data-service:1".into(),
            })
        });
    let mut connectors = MockConnectorInstanceFacadeTrait::new();
    connectors
        .expect_get_instance_by_distribution()
        .returning(|_| Ok(Some(connector())));
    let mut negotiation = MockNegotiationFacadeTrait::new();
    negotiation.expect_get_agreement().returning(|id| {
        Ok(OdrlAgreement {
            id: id.clone(),
            ..Default::default()
        })
    });
    let mut grants = MockGrantsFacadeTrait::new();
    match as_peer.map(str::to_string) {
        Some(id) => {
            grants
                .expect_verify_token()
                .returning(move |_| Ok(peer(&id)));
        }
        None => {
            grants
                .expect_verify_token()
                .returning(|_| Err(Errors::unauthorized("nope", None)));
        }
    }
    let mut mates = MockMatesFacadeTrait::new();
    mates
        .expect_get_mate_by_id()
        .returning(|_, id| Err(Errors::missing_resource(id, "not a mate", None)));
    TestContext::new()
        .process_repo(repo(store))
        .processes(processes(store))
        .messages(messages())
        .dataplane(dataplane())
        .catalog(catalog)
        .connectors(connectors)
        .negotiation(negotiation)
        .grants(grants)
        .mates(mates)
        .build()
}

fn router_as(store: &Store, as_peer: Option<&str>) -> Router {
    let (ctx, _) = DspContext::build(context(store, as_peer));
    DspRouter::new(ctx).router()
}

fn router(authorized: bool) -> Router {
    router_as(&store(), authorized.then_some("did:example:consumer"))
}

fn post(uri: &str, body: &str, token: Option<&str>) -> Request {
    let mut builder = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json");
    if let Some(token) = token {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    builder
        .body(axum::body::Body::from(body.to_string()))
        .unwrap()
}

fn get(uri: &str) -> Request {
    Request::builder()
        .method("GET")
        .uri(uri)
        .header("authorization", "Bearer tok")
        .body(axum::body::Body::empty())
        .unwrap()
}

fn termination(provider_pid: &str) -> String {
    format!(
        r#"{{
        "@context": ["https://w3id.org/dspace/2025/1/context.jsonld"],
        "@type": "TransferTerminationMessage",
        "providerPid": "{provider_pid}",
        "consumerPid": "{CONSUMER_PID}",
        "code": "STOPPED",
        "reason": ["no longer needed"]
    }}"#
    )
}

async fn body_json(response: Response) -> serde_json::Value {
    let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

/// DSP 10.2.2: a request answers 201 with a TransferProcess body (9.3.1).
#[tokio::test]
async fn transfer_request_answers_201_with_a_transfer_process() {
    let response = router(true)
        .oneshot(post("/request", REQUEST_BODY, Some("tok")))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = body_json(response).await;
    assert_eq!(body["@type"], "TransferProcess");
    assert_eq!(body["state"], "REQUESTED");
    assert_eq!(body["consumerPid"], CONSUMER_PID);
    assert!(
        body["providerPid"]
            .as_str()
            .is_some_and(|p| p.starts_with("urn:uuid:")),
        "{body}"
    );
    assert!(body["@context"].is_array(), "@context is REQUIRED: {body}");
}

/// Pids are opaque: a bare UUID consumerPid, as the DSP TCK sends, opens the process and comes
/// back exactly as sent.
#[tokio::test]
async fn a_bare_uuid_consumer_pid_is_accepted_as_sent() {
    let body = REQUEST_BODY.replace(CONSUMER_PID, "2c066edf-0a95-49eb-bc5e-a85beea59645");
    let response = router(true)
        .oneshot(post("/request", &body, Some("tok")))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let ack = body_json(response).await;
    assert_eq!(
        ack["consumerPid"], "2c066edf-0a95-49eb-bc5e-a85beea59645",
        "{ack}"
    );
}

/// DSP 9.2.1: a repeated request for a known consumerPid gets the first answer, not a second process.
#[tokio::test]
async fn a_repeated_transfer_request_replays_the_same_ack() {
    let store = store();
    let app = router_as(&store, Some("did:example:consumer"));
    let first = app
        .clone()
        .oneshot(post("/request", REQUEST_BODY, Some("tok")))
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::CREATED);
    let first = body_json(first).await;

    let second = app
        .oneshot(post("/request", REQUEST_BODY, Some("tok")))
        .await
        .unwrap();
    assert_eq!(second.status(), StatusCode::CREATED);
    assert_eq!(first, body_json(second).await);
    assert_eq!(
        store.lock().unwrap().len(),
        1,
        "a retry must not mint a second process"
    );
}

/// Every shape failure comes back at once, in a TransferError, and nothing is stored.
#[tokio::test]
async fn a_malformed_request_reports_every_violation() {
    let store = store();
    let body = r#"{
        "@context": ["https://w3id.org/dspace/2025/1/context.jsonld"],
        "@type": "TransferRequestMessage",
        "agreementId": "not-a-urn",
        "callbackAddress": "https://example.com/callback"
    }"#;
    let response = router_as(&store, Some("did:example:consumer"))
        .oneshot(post("/request", body, Some("tok")))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let error = body_json(response).await;
    assert_eq!(error["@type"], "TransferError");
    let reasons = error["reason"].as_array().unwrap();
    assert!(
        reasons.len() >= 3,
        "consumerPid, agreementId and format: {error}"
    );
    assert!(store.lock().unwrap().is_empty());
}

/// A body whose type is not the endpoint's is rejected before anything is loaded.
#[tokio::test]
async fn a_body_for_another_endpoint_is_rejected() {
    let response = router(true)
        .oneshot(post("/request", &termination("urn:uuid:p"), Some("tok")))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let error = body_json(response).await;
    assert!(error["reason"].to_string().contains("@type"), "{error}");
}

/// A state change for a process nobody has answers 404 (DSP 10.1.2.2).
#[tokio::test]
async fn a_state_change_on_an_unknown_process_answers_404() {
    let response = router(true)
        .oneshot(post(
            "/urn:uuid:unknown/termination",
            &termination("urn:uuid:unknown"),
            Some("tok"),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

/// The consumer can terminate the process it requested; the provider then reports it terminated.
#[tokio::test]
async fn the_requesting_peer_can_terminate_its_process() {
    let store = store();
    let app = router_as(&store, Some("did:example:consumer"));
    let created = body_json(
        app.clone()
            .oneshot(post("/request", REQUEST_BODY, Some("tok")))
            .await
            .unwrap(),
    )
    .await;
    let provider_pid = created["providerPid"].as_str().unwrap().to_string();

    let terminated = app
        .clone()
        .oneshot(post(
            &format!("/{provider_pid}/termination"),
            &termination(&provider_pid),
            Some("tok"),
        ))
        .await
        .unwrap();
    assert_eq!(terminated.status(), StatusCode::OK);

    let process = app.oneshot(get(&format!("/{provider_pid}"))).await.unwrap();
    assert_eq!(process.status(), StatusCode::OK);
    assert_eq!(body_json(process).await["state"], "TERMINATED");
}

/// Another peer cannot see or act on the process: it answers as if it did not exist.
#[tokio::test]
async fn another_peer_cannot_touch_the_process() {
    let store = store();
    let owner = router_as(&store, Some("did:example:consumer"));
    let created = body_json(
        owner
            .oneshot(post("/request", REQUEST_BODY, Some("tok")))
            .await
            .unwrap(),
    )
    .await;
    let provider_pid = created["providerPid"].as_str().unwrap().to_string();

    let intruder = router_as(&store, Some("did:example:intruder"));
    let terminate = intruder
        .clone()
        .oneshot(post(
            &format!("/{provider_pid}/termination"),
            &termination(&provider_pid),
            Some("tok"),
        ))
        .await
        .unwrap();
    assert_eq!(terminate.status(), StatusCode::NOT_FOUND);
    let read = intruder
        .oneshot(get(&format!("/{provider_pid}")))
        .await
        .unwrap();
    assert_eq!(read.status(), StatusCode::NOT_FOUND);
}

/// DSP 10.1.2.3: an unauthorized client gets 404, not 401 — the same answer as
/// a missing process, so probing reveals nothing.
#[tokio::test]
async fn an_unauthorized_request_answers_404() {
    let response = router(false)
        .oneshot(post("/request", REQUEST_BODY, Some("bad")))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

/// A request without token also answers 404.
#[tokio::test]
async fn a_request_without_a_token_answers_404() {
    let response = router(true)
        .oneshot(post("/request", REQUEST_BODY, None))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
