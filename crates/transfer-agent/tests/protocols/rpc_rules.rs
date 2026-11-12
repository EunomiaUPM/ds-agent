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

//! Outbound transfer rules: what the RPC body needs, what each role may send and to whom.

use common::dsp_common::odrl::OdrlAgreement;
use transfer_agent::entities::protocol::TransferRole::{Consumer, Provider, Relay};
use transfer_agent::protocols::dsp::entities::message_types::TransferDSPMessageType;
use transfer_agent::protocols::dsp::entities::message_types::TransferDSPMessageType::{
    TransferRequestMessage, TransferStartMessage, TransferTerminationMessage,
};
use transfer_agent::protocols::dsp::entities::state::TransferDSPState::{
    REQUESTED, STARTED, SUSPENDED,
};
use transfer_agent::protocols::dsp::entities::state_metadata::TransferDSPStateAttribute::{
    ByConsumer, ByProvider, OnRequest,
};
use transfer_agent::protocols::dsp::services::validator::TransferRpcRules;
use serde_json::json;
use transfer_agent::protocols::dsp::entities::context_rpc::{
    TransferRpcBody, TransferRpcRaw, TransferRpcRdf, TransferRpcTyped,
};
use transfer_agent::protocols::dsp::services::validator::TransferRules;

async fn typed(route: TransferDSPMessageType, body: TransferRpcBody) -> TransferRpcTyped {
    TransferRpcTyped::from_body(&route, &body).await.unwrap()
}

/// A request without peer, address, format or agreement reports all four at once.
#[tokio::test]
async fn an_empty_request_reports_every_missing_field() {
    let command = typed(TransferRequestMessage, TransferRpcBody::default()).await;
    let err = TransferRpcRules::input_rules()
        .validate(&TransferRequestMessage, &command)
        .unwrap_err();
    assert_eq!(err.len(), 4, "{err}");
}

/// A state change needs the id of the process it moves.
#[tokio::test]
async fn a_state_change_needs_a_process_id() {
    let without = typed(TransferTerminationMessage, TransferRpcBody::default()).await;
    assert!(TransferRpcRules::input_rules()
        .validate(&TransferTerminationMessage, &without)
        .is_err());
    let with = typed(
        TransferTerminationMessage,
        TransferRpcBody {
            id: Some("urn:transfer-process:5c1a7a2e-0000-4000-8000-000000000001".into()),
            ..TransferRpcBody::default()
        },
    )
    .await;
    assert!(TransferRpcRules::input_rules()
        .validate(&TransferTerminationMessage, &with)
        .is_ok());
}

/// A body typed as another RPC than the route's is rejected by a shape rule.
#[tokio::test]
async fn a_body_for_another_route_is_rejected() {
    let raw = TransferRpcRaw::new(
        &TransferStartMessage,
        json!({
            "@context": "https://w3id.org/eunomia/v1/context.jsonld",
            "@type": "RpcTransferTerminationMessage",
            "@id": "urn:transfer-process:5c1a7a2e-0000-4000-8000-000000000001"
        }),
    );
    let command = TransferRpcTyped::from_rdf(TransferRpcRdf::from_raw(raw).await.unwrap()).unwrap();
    let err = TransferRpcRules::input_rules()
        .validate(&TransferStartMessage, &command)
        .unwrap_err();
    assert!(err.to_string().contains("@type"), "{err}");
}

/// Only a consumer requests, a relay sends nothing, and a consumer only sends Start to resume
/// a transfer it suspended itself.
#[test]
fn each_role_sends_only_its_messages() {
    assert!(TransferRules::role_may_send(&Consumer, &TransferRequestMessage, "role").is_ok());
    assert!(TransferRules::role_may_send(&Provider, &TransferRequestMessage, "role").is_err());
    assert!(TransferRules::role_may_send(&Provider, &TransferStartMessage, "role").is_ok());
    assert!(TransferRules::role_may_send(&Consumer, &TransferStartMessage, "role").is_ok());
    assert!(TransferRules::role_may_send(&Consumer, &TransferTerminationMessage, "role").is_ok());
    assert!(TransferRules::role_may_send(&Relay, &TransferTerminationMessage, "role").is_err());

    // Outbound start state gate: Consumer may only send Start in SUSPENDED
    assert!(TransferRules::start_sent_in(&Provider, Some(&REQUESTED), "state").is_ok());
    assert!(TransferRules::start_sent_in(&Provider, Some(&SUSPENDED), "state").is_ok());
    assert!(TransferRules::start_sent_in(&Consumer, Some(&REQUESTED), "state").is_err());
    assert!(TransferRules::start_sent_in(&Consumer, Some(&STARTED), "state").is_err());
    assert!(TransferRules::start_sent_in(&Consumer, Some(&SUSPENDED), "state").is_ok());
    assert!(TransferRules::start_sent_in(&Relay, Some(&SUSPENDED), "state").is_err());
}

/// Option A (Provider suspends -> Provider resumes; Consumer resume fails) and
/// Option B (Consumer suspends -> Consumer resumes; Provider resume fails) on outbound RPC.
#[test]
fn outbound_suspension_semaphore_enforces_who_suspended_resumes() {
    // Initial start from REQUESTED (OnRequest): only Provider may send
    assert!(
        TransferRules::outbound_semaphore(&OnRequest, &TransferStartMessage, &Provider, "sem")
            .is_ok()
    );
    assert!(
        TransferRules::outbound_semaphore(&OnRequest, &TransferStartMessage, &Consumer, "sem")
            .is_err()
    );

    // Option A: Provider suspended (ByProvider) -> Provider resumes, Consumer rejected
    assert!(
        TransferRules::outbound_semaphore(&ByProvider, &TransferStartMessage, &Provider, "sem")
            .is_ok()
    );
    assert!(
        TransferRules::outbound_semaphore(&ByProvider, &TransferStartMessage, &Consumer, "sem")
            .is_err()
    );

    // Option B: Consumer suspended (ByConsumer) -> Consumer resumes, Provider rejected
    assert!(
        TransferRules::outbound_semaphore(&ByConsumer, &TransferStartMessage, &Consumer, "sem")
            .is_ok()
    );
    assert!(
        TransferRules::outbound_semaphore(&ByConsumer, &TransferStartMessage, &Provider, "sem")
            .is_err()
    );
}

fn agreement() -> OdrlAgreement {
    OdrlAgreement {
        assigner: "did:example:provider".into(),
        assignee: "did:example:consumer".into(),
        ..Default::default()
    }
}

/// A consumer sends to the agreement's provider, a provider to its consumer.
#[test]
fn the_peer_is_the_agreements_other_party() {
    let agreement = agreement();
    let party = |role, peer| TransferRules::agreement_counterparty(&agreement, role, peer, "peer");
    assert!(party(&Consumer, "did:example:provider").is_ok());
    assert!(party(&Provider, "did:example:consumer").is_ok());
    assert!(party(&Consumer, "did:example:consumer").is_err());
    assert!(party(&Consumer, "did:example:intruder").is_err());
}
