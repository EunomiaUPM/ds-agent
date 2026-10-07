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

//! GateKeeperModule: answering a peer's GNAP grant request and its continuation, which approves
//! the grant with a fresh token for the verified peer; who sees the received grants; and the
//! token check the other agents ask for.

use std::sync::{Arc, Mutex};

use auth::modules::GateKeeperModule;
use auth::types::token::IssuedToken;
use axum::body::Bytes;
use axum::http::HeaderMap;
use common::test_utils::scopes::TestUsers;
use ymir::errors::Errors;
use ymir::types::gnap::access_token::{BoundToken, TokenManagement};
use ymir::types::gnap::grant_response::{GrantResponse, GrantResponseKind};
use ymir::types::gnap::GrantStatus;
use ymir::types::participants::Visibility;
use ymir::utils::hash_token;

use crate::support::builders::{
    grant_request, participant, participant_plan, recv_grant, recv_grant_plan, recv_interaction,
    recv_interaction_plan, recv_verification, recv_verification_plan, relation, resource_req,
};
use crate::support::mocks::Doubles;

/// The peer's continuation `cont-1` for grant `g-1`, handled by the root, verified with `holder`.
fn continuation(d: &mut Doubles, holder: Option<&'static str>) {
    d.repos
        .recv_interaction
        .expect_get_by_continuation_id()
        .withf(|id| id == "cont-1")
        .returning(|_| Ok(recv_interaction("g-1")));
    d.gatekeeper
        .expect_validate_cont_req()
        .returning(|_, _, _| Ok(()));
    d.repos
        .recv_grant
        .expect_get_by_id()
        .withf(|id| id == "g-1")
        .returning(|id| Ok(recv_grant("/admin", id)));
    d.repos
        .recv_verification
        .expect_get_by_id()
        .returning(move |id| Ok(recv_verification(id, holder)));
}

// ==========================================================================================
// Grant request and continuation (peers)
// ==========================================================================================

/// A valid request stores grant, interaction, resource request and verification, and is
/// answered as pending with the OID4VP URI the peer must present to. Without rules for what
/// the peer asks, the grant gets no role nor visibility (the root, public).
#[tokio::test]
async fn grant_request_is_stored_and_answered_with_the_verification_uri() {
    let mut d = Doubles::default();
    d.gatekeeper
        .expect_validate_grant_req()
        .returning(|_, _| Ok(grant_request()));
    d.gatekeeper
        .expect_build_grant_plan()
        .withf(|role, visibility, class| {
            role.is_none() && visibility.is_none() && class.as_deref() == Some("agent")
        })
        .returning(|_, _, _| Ok(recv_grant_plan("/admin", "g-1")));
    d.gatekeeper
        .expect_build_interaction_plan()
        .returning(|id, _, _| Ok(recv_interaction_plan(id)));
    d.gatekeeper
        .expect_build_resource_req_plan()
        .returning(|id, _| Ok(resource_req(id)));
    d.repos
        .recv_grant
        .expect_create()
        .times(1)
        .returning(|plan| Ok(recv_grant(plan.role.as_str(), &plan.id)));
    d.repos
        .recv_interaction
        .expect_create()
        .times(1)
        .returning(|plan| Ok(recv_interaction(&plan.id)));
    d.repos.resource_req.expect_create().times(1).returning(Ok);
    d.verifier
        .expect_build_vp_plan()
        .withf(|id| id == "g-1")
        .returning(|id| Ok(recv_verification_plan(id)));
    d.repos
        .recv_verification
        .expect_create()
        .times(1)
        .returning(|plan| Ok(recv_verification(&plan.id, None)));
    d.verifier
        .expect_generate_verification_uri()
        .returning(|_| "openid4vp://me?state=state-1".to_string());

    let response = d
        .core()
        .manage_grant_req(Bytes::new(), HeaderMap::new())
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
        .returning(|_, _| Err(Errors::unauthorized("bad signature", None)));

    let response = d
        .core()
        .manage_grant_req(Bytes::new(), HeaderMap::new())
        .await;

    assert!(matches!(response, GrantResponse::Error(_)));
}

/// The verified holder is stored as a participant with the private relation of the
/// verification, and the grant is approved with the token the peer gets back, recording which
/// peer it belongs to.
#[tokio::test]
async fn continuation_approves_the_grant_with_a_token_for_the_peer() {
    let mut d = Doubles::default();
    continuation(&mut d, Some("did:web:peer"));
    d.gatekeeper
        .expect_build_mate_plan()
        .withf(|holder, nick, url| {
            holder == "did:web:peer" && nick == "peer" && url == "http://peer/callback"
        })
        .returning(|holder, _, _| participant_plan(holder));
    d.repos
        .participant
        .expect_create_if_absent()
        .times(1)
        .returning(|plan| Ok(participant(&plan.participant_id)));
    d.gatekeeper
        .expect_build_mate_rel_plan()
        .withf(|role, holder| role.as_str() == "/admin" && holder == "did:web:peer")
        .returning(|role, holder| relation("verification", role.as_str(), holder));
    d.repos
        .participant_relation
        .expect_force_update()
        .times(1)
        .returning(Ok);
    d.gatekeeper.expect_issue_token().times(1).returning(|grant, _| {
        grant.final_token_hash = Some(hash_token("peer-token"));
        IssuedToken {
            final_token: "peer-token".to_string(),
            final_expires_in: 3600,
            manage: TokenManagement::new(
                "http://me/gate/token/g-1",
                BoundToken::new("managing-token"),
            ),
        }
    });
    let stored_token = Arc::new(Mutex::new(None));
    let seen = stored_token.clone();
    d.repos
        .recv_grant
        .expect_update()
        .withf(|grant| {
            grant.status == GrantStatus::Approved
                && grant.participant_id.as_deref() == Some("did:web:peer")
        })
        .times(1)
        .returning(move |grant| {
            *seen.lock().unwrap() = grant.final_token_hash.clone();
            Ok(grant)
        });
    d.repos
        .resource_req
        .expect_get_by_id()
        .returning(|id| Ok(resource_req(id)));

    let response = d
        .core()
        .manage_continue_req("cont-1", Bytes::new(), HeaderMap::new())
        .await;

    let GrantResponse::Approved(approved) = response else {
        panic!("expected an approved answer, got {response:?}");
    };
    let GrantResponseKind::AccessToken { access_token } = approved.kind else {
        panic!("expected an access token");
    };
    assert_eq!(
        stored_token.lock().unwrap().as_deref(),
        Some(hash_token(&access_token.value).as_str())
    );
    assert_eq!(access_token.expires_in, Some(3600));
    assert!(access_token.manage.is_some());
}

/// Without a verified holder the continuation fails and no peer is stored.
#[tokio::test]
async fn continuation_without_a_presentation_registers_nobody() {
    let mut d = Doubles::default();
    continuation(&mut d, None);

    let response = d
        .core()
        .manage_continue_req("cont-1", Bytes::new(), HeaderMap::new())
        .await;

    assert!(matches!(response, GrantResponse::Error(_)));
}

/// An unknown continuation id is answered with an error, before checking the request.
#[tokio::test]
async fn unknown_continuation_is_rejected() {
    let mut d = Doubles::default();
    d.repos
        .recv_interaction
        .expect_get_by_continuation_id()
        .returning(|id| Err(Errors::missing_resource(id, "unknown continuation", None)));

    let response = d
        .core()
        .manage_continue_req("cont-1", Bytes::new(), HeaderMap::new())
        .await;

    assert!(matches!(response, GrantResponse::Error(_)));
}

// ==========================================================================================
// Received grants: who sees them (users)
// ==========================================================================================

/// A private grant handled by another branch is not found for the caller.
#[tokio::test]
async fn private_grant_of_another_branch_is_not_visible() {
    let mut d = Doubles::default();
    d.repos.recv_grant.expect_get_by_id().returning(|id| {
        let mut grant = recv_grant("/admin/acme", id);
        grant.visibility = Visibility::Private;
        Ok(grant)
    });

    let result =
        GateKeeperModule::get_by_id(&d.core(), &TestUsers::user("ana", "/admin/upm"), "g-1")
            .await;

    assert!(result.is_err());
}

/// A public grant is seen by any user, whatever role handles it.
#[tokio::test]
async fn public_grant_is_visible_to_anyone() {
    let mut d = Doubles::default();
    d.repos
        .recv_grant
        .expect_get_by_id()
        .returning(|id| Ok(recv_grant("/admin/acme", id)));

    let result =
        GateKeeperModule::get_by_id(&d.core(), &TestUsers::user("ana", "/admin/upm"), "g-1")
            .await;

    assert!(result.is_ok());
}

/// A private grant is seen by the role that handles it.
#[tokio::test]
async fn private_grant_is_visible_to_its_role() {
    let mut d = Doubles::default();
    d.repos.recv_grant.expect_get_by_id().returning(|id| {
        let mut grant = recv_grant("/admin/upm", id);
        grant.visibility = Visibility::Private;
        Ok(grant)
    });

    let result =
        GateKeeperModule::get_by_id(&d.core(), &TestUsers::user("ana", "/admin/upm"), "g-1")
            .await;

    assert!(result.is_ok());
}

// ==========================================================================================
// Token check (other agents, through the grants facade)
// ==========================================================================================

/// A token of an approved grant gives the peer it was issued to and the role that handles it.
#[tokio::test]
async fn token_of_an_approved_grant_gives_the_peer_and_its_role() {
    let mut d = Doubles::default();
    d.repos
        .recv_grant
        .expect_get_valid_by_final_hash()
        .withf(|hash, _| hash == hash_token("peer-token"))
        .returning(|_, _| {
            let mut grant = recv_grant("/admin/upm", "g-1");
            grant.status = GrantStatus::Approved;
            grant.participant_id = Some("did:web:peer".to_string());
            grant.final_token_hash = Some(hash_token("peer-token"));
            Ok(grant)
        });

    let peer = d.core().verify_token("peer-token").await.unwrap();

    assert_eq!(peer.participant_id, "did:web:peer");
    assert_eq!(peer.role.as_str(), "/admin/upm");
}

/// A token this connector did not issue (or not approved) is rejected.
#[tokio::test]
async fn unknown_token_is_rejected() {
    let mut d = Doubles::default();
    d.repos
        .recv_grant
        .expect_get_valid_by_final_hash()
        .returning(|_, _| Err(Errors::missing_resource("token", "no approved grant", None)));

    let result = d.core().verify_token("forged").await;

    assert!(result.is_err());
}
