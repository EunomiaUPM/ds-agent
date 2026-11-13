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

//! Unit tests for `DataplaneSignalWorker` and `RequestHandler` orphan cleanup.

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use axum::extract::Request;
use axum::routing::post;
use axum::{Json, Router};
use chrono::Utc;
use common::dsp_common::odrl::OdrlAgreement;
use common::facades::grants_facade::{MockGrantsFacadeTrait, PeerToken, VerifiedPeer};
use common::facades::mates_facade::MockMatesFacadeTrait;
use common::facades::negotiation::MockNegotiationFacadeTrait;
use common::test_utils::scopes::TestUsers;
use compact_str::CompactString;
use dataplane::{DataplaneSignal, DataplaneSignalSender};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use transfer_agent::data::repo::transfer_message::MockTransferMessageRepoTrait;
use transfer_agent::data::repo::transfer_process::MockTransferProcessRepoTrait;
use transfer_agent::data::repo::transfer_process_identifier::MockTransferIdentifierRepoTrait;
use transfer_agent::entities::ids::{MessageId, ParticipantId, TransferProcessId};
use transfer_agent::entities::message_envelope::MessageEnvelope;
use transfer_agent::entities::protocol::{
    ProtocolId, ProtocolMessageType, ProtocolState, StateMetadata, TransferCorrelation,
    TransferDirection, TransferRole, CONSUMER_PID_KEY, PROVIDER_PID_KEY,
};
use transfer_agent::entities::transfer_message::Direction;
use transfer_agent::entities::transfer_process::TransferProcess;
use transfer_agent::protocols::dsp::entities::context_common::{
    TransferContextConnectorRole, TransferContextProcessSlot, DIRECTION_PROPERTY,
};
use transfer_agent::protocols::dsp::entities::context_dsp::{
    TransferDspDomain, TransferDspParsed, TransferDspRaw, TransferDspRdf, TransferDspTyped,
};
use transfer_agent::protocols::dsp::entities::message_types::TransferDSPMessageType;
use transfer_agent::protocols::dsp::facades::dataplane_facade::MockDataPlaneFacadeTrait;
use transfer_agent::protocols::dsp::services::dataplane_signals::DataplaneSignalWorker;
use transfer_agent::protocols::dsp::services::dsp_flow::handlers::request::RequestHandler;
use transfer_agent::protocols::dsp::services::dsp_persistence::service::TransferDspPersistence;
use transfer_agent::protocols::dsp::setup::DspContext;
use transfer_agent::services::transfer_message::MockTransferMessageServiceTrait;
use transfer_agent::services::transfer_message::views::TransferMessageView;
use transfer_agent::services::transfer_process::MockTransferProcessServiceTrait;
use transfer_agent::services::transfer_process::views::TransferProcessView;
use urn::Urn;
use ymir::boot::workers::BackgroundWorker;
use ymir::errors::Errors;
use ymir::types::oauth::RolePath;
use ymir::types::participants::Visibility;

use crate::support::context::TestContext;

fn sample_urn(suffix: u32) -> Urn {
    Urn::from_str(&format!("urn:uuid:{suffix:08x}-0000-4000-8000-000000000000")).unwrap()
}

fn sample_agreement(id: Urn) -> OdrlAgreement {
    OdrlAgreement {
        id,
        assigner: "did:example:provider".into(),
        assignee: "did:example:consumer".into(),
        ..Default::default()
    }
}

fn sample_process(
    process_urn: Urn,
    state: &str,
    callback_url: &str,
    agreement_urn: Urn,
) -> TransferProcess {
    let now = Utc::now();
    let consumer_pid = "urn:uuid:11111111-1111-4111-8111-111111111111".to_string();
    let provider_pid = "urn:uuid:22222222-2222-4222-8222-222222222222".to_string();
    let mut corr = TransferCorrelation::empty();
    corr.consumer_pid = Some(consumer_pid.clone());
    corr.provider_pid = Some(provider_pid.clone());
    corr.identifiers
        .insert(CONSUMER_PID_KEY.to_string(), consumer_pid);
    corr.identifiers
        .insert(PROVIDER_PID_KEY.to_string(), provider_pid);
    corr.agreement_id = Some(agreement_urn);
    corr.callback_address = Some(url::Url::parse(callback_url).unwrap());
    corr.peer_participant_id = Some(ParticipantId::new("did:example:consumer"));

    TransferProcess::rehydrate(
        TransferProcessId::new(process_urn),
        TestUsers::owner("tenant-1"),
        TransferRole::Provider,
        now,
        now,
        0,
        ProtocolId::Dsp2025_1,
        ProtocolState(CompactString::from(state)),
        StateMetadata::empty(),
        corr,
        json!({ DIRECTION_PROPERTY: TransferDirection::Pull }),
        None,
    )
}

fn dummy_process_view(process: &TransferProcess) -> TransferProcessView {
    TransferProcessView::assemble(process.clone(), HashMap::new())
}

fn dummy_message_view(process_urn: Urn) -> TransferMessageView {
    let owner = TestUsers::owner("tenant-1");
    TransferMessageView {
        id: MessageId::new(
            Urn::from_str("urn:uuid:99999999-9999-4999-8999-999999999999").unwrap(),
        ),
        transfer_process_id: TransferProcessId::new(process_urn),
        user_id: owner.user_id,
        user_role: owner.role,
        visibility: owner.visibility,
        direction: Direction::Outbound,
        protocol: ProtocolId::Dsp2025_1,
        message_type: ProtocolMessageType(CompactString::from("TransferCompletionMessage")),
        state_transition_from: "STARTED".into(),
        state_transition_to: "COMPLETED".into(),
        envelope: MessageEnvelope::new(json!({}), None),
        occurred_at: Utc::now(),
    }
}

#[tokio::test]
async fn completed_signal_dispatches_transfer_completion_message_and_completes_process() {
    let received_bodies: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));
    let captured = received_bodies.clone();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let peer_addr = listener.local_addr().unwrap();
    let peer_router = Router::new().route(
        "/dsp/current/transfers/{pid}/completion",
        post(move |Json(body): Json<Value>| {
            let captured = captured.clone();
            async move {
                captured.lock().unwrap().push(body);
                Json(json!({}))
            }
        }),
    );
    let peer_task = tokio::spawn(async move {
        let _ = axum::serve(listener, peer_router).await;
    });

    let process_urn = sample_urn(1);
    let agreement_urn = sample_urn(99);
    let callback_url = format!("http://{peer_addr}/dsp/current");
    let started = sample_process(
        process_urn.clone(),
        "STARTED",
        &callback_url,
        agreement_urn.clone(),
    );
    let completed = sample_process(
        process_urn.clone(),
        "COMPLETED",
        &callback_url,
        agreement_urn.clone(),
    );

    let mut proc_repo = MockTransferProcessRepoTrait::new();
    let started_for_by_id = started.clone();
    proc_repo
        .expect_get_transfer_process_by_id()
        .returning(move |_, _| Ok(Some(started_for_by_id.clone())));
    proc_repo
        .expect_get_transfer_process_by_key_value()
        .returning(move |_, _| Ok(Some(completed.clone())));

    let mut proc_svc = MockTransferProcessServiceTrait::new();
    let completed_for_edit = sample_process(
        process_urn.clone(),
        "COMPLETED",
        &callback_url,
        agreement_urn.clone(),
    );
    proc_svc
        .expect_edit()
        .times(1)
        .withf(|_, _, cmd| cmd.state.as_ref().map(|s| s.0.as_str()) == Some("COMPLETED"))
        .returning(move |_, _, _| Ok(dummy_process_view(&completed_for_edit)));

    let mut msg_svc = MockTransferMessageServiceTrait::new();
    let p_for_msg = process_urn.clone();
    msg_svc
        .expect_create()
        .times(1)
        .returning(move |_, _| Ok(dummy_message_view(p_for_msg.clone())));

    let mut negotiation = MockNegotiationFacadeTrait::new();
    let agr = sample_agreement(agreement_urn);
    negotiation
        .expect_get_agreement()
        .returning(move |_| Ok(agr.clone()));

    let mut mates = MockMatesFacadeTrait::new();
    mates
        .expect_get_mate_by_id()
        .returning(|_, _| Err(Errors::missing_resource("mate", "not found", None)));

    let mut grants = MockGrantsFacadeTrait::new();
    grants
        .expect_peer_token()
        .returning(|_, _, _, _| Ok(PeerToken::Ready("peer-gnap-token".into())));

    let mut dataplane = MockDataPlaneFacadeTrait::new();
    dataplane
        .expect_on_transfer_completion_pre()
        .times(1)
        .returning(|_| Ok(None));
    dataplane
        .expect_on_transfer_completion_post()
        .times(1)
        .returning(|_| Ok(None));

    let (signal_tx, signal_rx) = DataplaneSignalSender::open(16);
    let ctx = TestContext::new()
        .process_repo(proc_repo)
        .identifier_repo(MockTransferIdentifierRepoTrait::new())
        .message_repo(MockTransferMessageRepoTrait::new())
        .processes(proc_svc)
        .messages(msg_svc)
        .negotiation(negotiation)
        .mates(mates)
        .grants(grants)
        .dataplane(dataplane)
        .build();

    let (dsp, _) = DspContext::build(ctx.clone());
    let worker = Box::new(
        DataplaneSignalWorker::new(signal_rx, ctx, dsp.outbound.clone()).with_retry_delays(vec![5]),
    );
    let cancel = CancellationToken::new();
    let cancel_worker = cancel.clone();
    let handle = tokio::spawn(async move { worker.run(cancel_worker).await });

    signal_tx.emit(DataplaneSignal::Completed {
        transfer_process_id: process_urn,
    });

    tokio::time::timeout(std::time::Duration::from_secs(3), async {
        loop {
            if !received_bodies.lock().unwrap().is_empty() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("peer should receive TransferCompletionMessage");

    tokio::time::sleep(std::time::Duration::from_millis(30)).await;
    cancel.cancel();
    handle.await.unwrap().unwrap();
    peer_task.abort();

    let bodies = received_bodies.lock().unwrap();
    assert_eq!(bodies.len(), 1);
    assert_eq!(bodies[0]["@type"], "TransferCompletionMessage");
}

#[tokio::test]
async fn completed_signal_is_ignored_when_process_already_terminal() {
    let process_urn = sample_urn(2);
    let agreement_urn = sample_urn(99);
    let already_completed = sample_process(
        process_urn.clone(),
        "COMPLETED",
        "http://127.0.0.1:9/dsp/current",
        agreement_urn,
    );

    let mut proc_repo = MockTransferProcessRepoTrait::new();
    proc_repo
        .expect_get_transfer_process_by_id()
        .times(1)
        .returning(move |_, _| Ok(Some(already_completed.clone())));

    let ctx = TestContext::new().process_repo(proc_repo).build();
    let (dsp, _) = DspContext::build(ctx.clone());

    DataplaneSignalWorker::process_signal(
        ctx,
        dsp.outbound.clone(),
        DataplaneSignal::Completed {
            transfer_process_id: process_urn,
        },
        &[1],
    )
    .await;
}

#[tokio::test]
async fn failed_signal_forces_local_termination_and_dataplane_cleanup_when_peer_unreachable() {
    let process_urn = sample_urn(3);
    let agreement_urn = sample_urn(99);
    let started = sample_process(
        process_urn.clone(),
        "STARTED",
        "http://127.0.0.1:9/dsp/current",
        agreement_urn.clone(),
    );

    let mut proc_repo = MockTransferProcessRepoTrait::new();
    let started_clone = started.clone();
    proc_repo
        .expect_get_transfer_process_by_id()
        .returning(move |_, _| Ok(Some(started_clone.clone())));

    let mut negotiation = MockNegotiationFacadeTrait::new();
    let agr = sample_agreement(agreement_urn.clone());
    negotiation
        .expect_get_agreement()
        .returning(move |_| Ok(agr.clone()));

    let mut mates = MockMatesFacadeTrait::new();
    mates
        .expect_get_mate_by_id()
        .returning(|_, _| Err(Errors::missing_resource("mate", "not found", None)));

    let mut grants = MockGrantsFacadeTrait::new();
    grants
        .expect_peer_token()
        .returning(|_, _, _, _| Err(Errors::crazy("peer offline", None)));

    let mut proc_svc = MockTransferProcessServiceTrait::new();
    let terminated_view_proc = sample_process(
        process_urn.clone(),
        "TERMINATED",
        "http://127.0.0.1:9/dsp/current",
        agreement_urn,
    );
    proc_svc
        .expect_edit()
        .times(1)
        .withf(|_, _, cmd| {
            cmd.state.as_ref().map(|s| s.0.as_str()) == Some("TERMINATED")
                && cmd
                    .state_metadata
                    .as_ref()
                    .and_then(|m| m.code.as_deref())
                    == Some("DATAPLANE_MAX_RETRIES_EXCEEDED")
        })
        .returning(move |_, _, _| Ok(dummy_process_view(&terminated_view_proc)));

    let mut dataplane = MockDataPlaneFacadeTrait::new();
    dataplane
        .expect_on_transfer_termination_pre()
        .times(1)
        .returning(|_| Ok(None));
    dataplane
        .expect_on_transfer_termination_post()
        .times(1)
        .withf(|ctx| {
            ctx.reason.as_deref()
                == Some(
                    "Forced local termination [code=DATAPLANE_MAX_RETRIES_EXCEEDED]: Upstream 503 exceeded maxRetries",
                )
        })
        .returning(|_| Ok(None));

    let ctx = TestContext::new()
        .process_repo(proc_repo)
        .negotiation(negotiation)
        .mates(mates)
        .grants(grants)
        .processes(proc_svc)
        .dataplane(dataplane)
        .build();
    let (dsp, _) = DspContext::build(ctx.clone());

    DataplaneSignalWorker::process_signal(
        ctx,
        dsp.outbound.clone(),
        DataplaneSignal::Failed {
            transfer_process_id: process_urn,
            code: "DATAPLANE_MAX_RETRIES_EXCEEDED".into(),
            reason: "Upstream 503 exceeded maxRetries".into(),
        },
        &[1, 1],
    )
    .await;
}

#[tokio::test]
async fn request_handler_deletes_orphan_process_when_dataplane_init_fails() {
    let process_urn = sample_urn(4);
    let agreement_urn = sample_urn(99);
    let requested = sample_process(
        process_urn.clone(),
        "REQUESTED",
        "http://127.0.0.1:8080/dsp/current",
        agreement_urn.clone(),
    );

    let mut proc_repo = MockTransferProcessRepoTrait::new();
    let requested_for_reload = requested.clone();
    proc_repo
        .expect_get_transfer_process_by_key_value()
        .times(1)
        .returning(move |_, _| Ok(Some(requested_for_reload.clone())));

    let mut proc_svc = MockTransferProcessServiceTrait::new();
    let requested_for_create = requested.clone();
    proc_svc
        .expect_create()
        .times(1)
        .returning(move |_, _| Ok(dummy_process_view(&requested_for_create)));
    let expected_delete_urn = process_urn.clone();
    proc_svc
        .expect_delete()
        .times(1)
        .withf(move |_, id| id == &expected_delete_urn)
        .returning(|_, _| Ok(()));

    let mut msg_svc = MockTransferMessageServiceTrait::new();
    let p_for_msg = process_urn.clone();
    msg_svc
        .expect_create()
        .times(1)
        .returning(move |_, _| Ok(dummy_message_view(p_for_msg.clone())));

    let mut dataplane = MockDataPlaneFacadeTrait::new();
    dataplane
        .expect_on_transfer_request_post()
        .times(1)
        .returning(|_| Err(Errors::crazy("SetInit failed in dataplane", None)));

    let ctx = TestContext::new()
        .process_repo(proc_repo)
        .processes(proc_svc)
        .messages(msg_svc)
        .dataplane(dataplane)
        .build();
    let persistence = TransferDspPersistence::new(ctx.clone());

    let peer = VerifiedPeer {
        participant_id: "did:example:consumer".into(),
        role: RolePath::root(),
        visibility: Visibility::Public,
    };
    let req_json = json!({
        "@context": ["https://w3id.org/dspace/2025/1/context.jsonld"],
        "@type": "TransferRequestMessage",
        "consumerPid": "urn:uuid:11111111-1111-4111-8111-111111111111",
        "agreementId": agreement_urn.to_string(),
        "format": "HTTP+PULL",
        "callbackAddress": "http://127.0.0.1:8080/dsp/current"
    });
    let http_req = Request::builder()
        .method("POST")
        .uri("/transfers/request")
        .body(axum::body::Body::from(req_json.to_string()))
        .unwrap();
    let raw = TransferDspRaw::from_request(peer, http_req).await.unwrap();
    let parsed = TransferDspParsed::from_raw(
        raw,
        &ProtocolId::Dsp2025_1,
        &TransferDSPMessageType::TransferRequestMessage,
    )
    .unwrap();
    let rdf = TransferDspRdf::from_parsed(parsed).await.unwrap();
    let typed = TransferDspTyped::from_rdf(rdf).unwrap();
    let domain = TransferDspDomain {
        typed,
        process: TransferContextProcessSlot::New {
            consumer_pid: "urn:uuid:11111111-1111-4111-8111-111111111111".into(),
        },
        agreement: sample_agreement(agreement_urn),
        role: TransferRole::Provider,
        connector_instance: TransferContextConnectorRole::ConsumerNotHavingConnector,
        transfer_direction: TransferDirection::Pull,
        is_restart: false,
    };

    let err = RequestHandler::handle(&ctx, &persistence, domain)
        .await
        .expect_err("RequestHandler should fail when dataplane SetInit fails");
    assert!(format!("{err:?}").contains("SetInit failed in dataplane"));
}
