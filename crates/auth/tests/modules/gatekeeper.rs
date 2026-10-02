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

//! GateKeeperModule: answering a peer's GNAP grant request and its continuation, which
//! registers the verified peer with a fresh token.

use std::sync::{Arc, Mutex};

use auth::modules::GateKeeperModule;
use axum::body::Bytes;
use axum::http::HeaderMap;
use common::test_utils::scopes::TestScopes;
use ymir::errors::Errors;
use ymir::types::gnap::grant_response::{GrantResponse, GrantResponseKind};
use ymir::types::gnap::GrantStatus;

use crate::support::builders::{
    grant_request, participant, participant_plan, recv_grant, recv_grant_plan, recv_interaction,
    recv_interaction_plan, recv_verification, recv_verification_plan, resource_req,
};
use crate::support::mocks::Doubles;

/// The peer's continuation for grant `g-1` of `tenant-1`, verified with `holder`.
fn continuation(d: &mut Doubles, holder: Option<&'static str>) {
    d.repos
        .recv_interaction
        .expect_get_by_cont_id()
        .withf(|id| id == "cont-1")
        .returning(|_| Ok(recv_interaction("tenant-1", "g-1")));
    d.gatekeeper
        .expect_validate_cont_req()
        .returning(|_, _, _| Ok(()));
    d.repos
        .recv_grant
        .expect_get_by_id()
        .withf(|id| id == "g-1")
        .returning(|id| Ok(recv_grant("tenant-1", id)));
    d.repos
        .recv_verification
        .expect_get_by_id()
        .returning(move |id| Ok(recv_verification("tenant-1", id, holder)));
}

/// A valid request stores grant, interaction, resource request and verification, and is
/// answered as pending with the OID4VP URI the peer must present to.
#[tokio::test]
async fn grant_request_is_stored_and_answered_with_the_verification_uri() {
    let mut d = Doubles::default();
    d.gatekeeper
        .expect_validate_grant_req()
        .withf(|tenant, _, _| tenant == "tenant-1")
        .returning(|_, _, _| Ok(grant_request()));
    d.gatekeeper
        .expect_build_grant_plan()
        .withf(|tenant, class| tenant == "tenant-1" && class.as_deref() == Some("agent"))
        .returning(|tenant, _| Ok(recv_grant_plan(tenant, "g-1")));
    d.gatekeeper
        .expect_build_interaction_plan()
        .returning(|tenant, id, _, _| Ok(recv_interaction_plan(tenant, id)));
    d.gatekeeper
        .expect_build_resource_req_plan()
        .returning(|tenant, id, _| Ok(resource_req(tenant, id)));
    d.repos
        .recv_grant
        .expect_create()
        .times(1)
        .returning(|plan| Ok(recv_grant(&plan.tenant_id, &plan.id)));
    d.repos
        .recv_interaction
        .expect_create()
        .times(1)
        .returning(|plan| Ok(recv_interaction(&plan.tenant_id, &plan.id)));
    d.repos.resource_req.expect_create().times(1).returning(Ok);
    d.verifier
        .expect_build_vp_plan()
        .withf(|tenant, id| tenant == "tenant-1" && id == "g-1")
        .returning(|tenant, id| Ok(recv_verification_plan(tenant, id)));
    d.repos
        .recv_verification
        .expect_create()
        .times(1)
        .returning(|plan| Ok(recv_verification(&plan.tenant_id, &plan.id, None)));
    d.verifier
        .expect_generate_verification_uri()
        .returning(|_| "openid4vp://me?state=state-1".to_string());

    let response = d
        .core()
        .manage_grant_req("tenant-1".to_string(), Bytes::new(), HeaderMap::new())
        .await;

    let GrantResponse::Pending(pending) = response else {
        panic!("expected a pending answer, got {response:?}");
    };
    assert_eq!(
        pending.interact.oid4vp.as_deref(),
        Some("openid4vp://me?state=state-1")
    );
    assert_eq!(pending.instance_id.as_deref(), Some("g-1"));
}

/// An invalid request is answered with a GNAP error and nothing is stored.
#[tokio::test]
async fn invalid_grant_request_is_answered_with_an_error() {
    let mut d = Doubles::default();
    d.gatekeeper
        .expect_validate_grant_req()
        .returning(|_, _, _| Err(Errors::unauthorized("bad signature", None)));

    let response = d
        .core()
        .manage_grant_req("tenant-1".to_string(), Bytes::new(), HeaderMap::new())
        .await;

    assert!(matches!(response, GrantResponse::Error(_)));
}

/// The verified holder is registered with the same token the peer gets back, and the grant is
/// approved.
#[tokio::test]
async fn continuation_registers_the_peer_with_its_token() {
    let mut d = Doubles::default();
    continuation(&mut d, Some("did:web:peer"));
    let stored_token = Arc::new(Mutex::new(None));
    d.gatekeeper
        .expect_build_mate_plan()
        .withf(|tenant, holder, nick, url, _| {
            tenant == "tenant-1"
                && holder == "did:web:peer"
                && nick == "peer"
                && url == "http://peer/callback"
        })
        .returning(|tenant, holder, _, _, token| {
            let mut plan = participant_plan(tenant, holder);
            plan.token = Some(token.to_string());
            plan
        });
    let seen = stored_token.clone();
    d.repos
        .participant
        .expect_force_update()
        .times(1)
        .returning(move |plan| {
            *seen.lock().unwrap() = plan.token.clone();
            Ok(participant(&plan.tenant_id, &plan.participant_id))
        });
    d.repos
        .recv_grant
        .expect_update()
        .withf(|grant| grant.status == GrantStatus::Approved && grant.token.is_some())
        .times(1)
        .returning(Ok);
    d.repos
        .resource_req
        .expect_get_by_id()
        .returning(|id| Ok(resource_req("tenant-1", id)));

    let response = d
        .core()
        .manage_continue_req(
            "tenant-1".to_string(),
            "cont-1".to_string(),
            Bytes::new(),
            HeaderMap::new(),
        )
        .await;

    let GrantResponse::Approved(approved) = response else {
        panic!("expected an approved answer, got {response:?}");
    };
    let GrantResponseKind::AccessToken { access_token } = approved.kind else {
        panic!("expected an access token");
    };
    assert_eq!(
        stored_token.lock().unwrap().as_deref(),
        Some(access_token.value.as_str())
    );
}

/// Without a verified holder the continuation fails and no peer is registered.
#[tokio::test]
async fn continuation_without_a_presentation_registers_nobody() {
    let mut d = Doubles::default();
    continuation(&mut d, None);

    let response = d
        .core()
        .manage_continue_req(
            "tenant-1".to_string(),
            "cont-1".to_string(),
            Bytes::new(),
            HeaderMap::new(),
        )
        .await;

    assert!(matches!(response, GrantResponse::Error(_)));
}

/// A continuation id of another tenant is answered as unknown, before checking the request.
#[tokio::test]
async fn continuation_of_another_tenant_is_rejected() {
    let mut d = Doubles::default();
    d.repos
        .recv_interaction
        .expect_get_by_cont_id()
        .returning(|_| Ok(recv_interaction("tenant-2", "g-1")));

    let response = d
        .core()
        .manage_continue_req(
            "tenant-1".to_string(),
            "cont-1".to_string(),
            Bytes::new(),
            HeaderMap::new(),
        )
        .await;

    assert!(matches!(response, GrantResponse::Error(_)));
}

/// A received grant of another tenant is not found for the caller.
#[tokio::test]
async fn grant_of_another_tenant_is_not_visible() {
    let mut d = Doubles::default();
    d.repos
        .recv_grant
        .expect_get_by_id()
        .returning(|id| Ok(recv_grant("tenant-2", id)));

    let result =
        GateKeeperModule::get_by_id(&d.core(), &TestScopes::owner("tenant-1"), "g-1".to_string())
            .await;

    assert!(result.is_err());
}
