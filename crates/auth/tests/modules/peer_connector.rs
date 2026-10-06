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

//! PeerConnectorModule: onboarding with a peer through GNAP, presenting over OID4VP when the
//! peer asks, following the peer's callback, and the token a user presents to a peer.

use auth::modules::PeerConnectorModule;
use auth::types::response::TokenWhatResponse;
use common::test_utils::scopes::TestUsers;
use ymir::errors::Errors;
use ymir::types::gnap::grant_request::interact::InteractAction;
use ymir::types::gnap::{ApprovedCallbackBody, CallbackBody, GrantStatus, RejectedCallbackBody};
use ymir::types::verification::VerificationStatus;
use ymir::types::wallet::OidcUri;

use crate::support::builders::{
    issued_token_response, participant, participant_plan, reach_provider, relation, resource_req,
    sent_grant, sent_grant_plan, sent_interaction, sent_interaction_plan, sent_verification,
    sent_verification_plan,
};
use crate::support::mocks::Doubles;

/// Grant `g-1` Ana (`/admin/upm`) sends to the peer, which answers `what`.
fn connection(d: &mut Doubles, auto: bool, what: TokenWhatResponse) {
    d.peer_connector
        .expect_build_grant_plan()
        .withf(|user, reach| user.id() == "ana" && reach.id == "did:web:peer")
        .returning(|user, _| sent_grant_plan(user.id(), user.role().as_str(), "g-1"));
    d.peer_connector
        .expect_build_interaction_plan()
        .returning(sent_interaction_plan);
    d.peer_connector
        .expect_build_resource_req_plan()
        .withf(|_, actions| actions == &[InteractAction::Talk])
        .returning(|id, _| resource_req(id));
    d.repos.sent_grant.expect_create().returning(move |plan| {
        Ok(sent_grant(&plan.user_id, plan.role.as_str(), &plan.id, auto))
    });
    d.repos
        .sent_interaction
        .expect_create()
        .returning(|plan| Ok(sent_interaction(&plan.id)));
    d.repos.resource_req.expect_create().returning(Ok);
    d.peer_connector
        .expect_send_grant_req()
        .returning(|_, _, req| Ok(issued_token_response("peer-token", req)));
    d.peer_connector
        .expect_manage_grant_resp()
        .return_once(move |_, _, _| Ok(what));
    d.repos.sent_grant.expect_update().returning(Ok);
    d.repos.sent_interaction.expect_update().returning(Ok);
}

/// The presentation the peer asks for, recorded for grant `g-1`.
fn presentation_requested(d: &mut Doubles) {
    d.peer_connector
        .expect_build_verification_plan()
        .withf(|uri, id| uri == "openid4vp://peer" && id == "g-1")
        .returning(|_, id| Ok(sent_verification_plan(id)));
    d.repos
        .sent_verification
        .expect_create()
        .times(1)
        .returning(|plan| Ok(sent_verification(&plan.id)));
}

fn ana() -> ymir::types::oauth::UserInfo {
    TestUsers::user("ana", "/admin/upm")
}

// ==========================================================================================
// Onboarding (users)
// ==========================================================================================

/// A peer that grants access right away is stored (if new) with Ana's relation to it.
#[tokio::test]
async fn completed_grant_stores_the_peer_and_the_users_relation() {
    let mut d = Doubles::default();
    connection(&mut d, true, TokenWhatResponse::Completed);
    d.peer_connector
        .expect_build_mate_plan()
        .returning(|grant| participant_plan(&grant.participant_id));
    d.repos
        .participant
        .expect_create_if_absent()
        .withf(|plan| plan.participant_id == "did:web:peer")
        .times(1)
        .returning(|plan| Ok(participant(&plan.participant_id)));
    d.peer_connector
        .expect_build_mate_relation()
        .returning(|grant| relation(&grant.user_id, grant.role.as_str(), &grant.participant_id));
    d.repos
        .participant_relation
        .expect_force_update()
        .withf(|relation| relation.user_id == "ana" && relation.role.as_str() == "/admin/upm")
        .times(1)
        .returning(Ok);

    d.core()
        .req_peer_connection(&ana(), reach_provider())
        .await
        .unwrap();
}

/// With `auto`, the presentation the peer asks for goes out through the wallet at once.
#[tokio::test]
async fn automatic_grant_presents_through_the_wallet() {
    let mut d = Doubles::default();
    connection(
        &mut d,
        true,
        TokenWhatResponse::Presentation("openid4vp://peer".to_string()),
    );
    presentation_requested(&mut d);
    d.wallet
        .expect_process_oid4vp()
        .withf(|uri| uri == "openid4vp://peer")
        .times(1)
        .returning(|_| Ok(()));
    d.repos
        .sent_verification
        .expect_update()
        .withf(|v| v.status == VerificationStatus::Verified && v.ended_at.is_some())
        .times(1)
        .returning(Ok);

    d.core()
        .req_peer_connection(&ana(), reach_provider())
        .await
        .unwrap();
}

/// Without `auto`, the presentation is only recorded and waits for the user.
#[tokio::test]
async fn manual_grant_waits_for_the_presentation() {
    let mut d = Doubles::default();
    connection(
        &mut d,
        false,
        TokenWhatResponse::Presentation("openid4vp://peer".to_string()),
    );
    presentation_requested(&mut d);

    d.core()
        .req_peer_connection(&ana(), reach_provider())
        .await
        .unwrap();
}

/// A wallet failure marks the presentation as failed instead of failing the request.
#[tokio::test]
async fn wallet_failure_marks_the_presentation_failed() {
    let mut d = Doubles::default();
    d.repos
        .sent_grant
        .expect_get_by_id()
        .returning(|id| Ok(sent_grant("ana", "/admin/upm", id, true)));
    d.repos
        .sent_verification
        .expect_get_by_id()
        .returning(|id| Ok(sent_verification(id)));
    d.wallet
        .expect_process_oid4vp()
        .returning(|_| Err(Errors::crazy("wallet down", None)));
    d.repos
        .sent_verification
        .expect_update()
        .withf(|v| v.status == VerificationStatus::Failed && v.ended_at.is_some())
        .times(1)
        .returning(Ok);

    PeerConnectorModule::process_oid4vp(
        &d.core(),
        &ana(),
        "g-1",
        OidcUri {
            uri: "openid4vp://peer".to_string(),
        },
    )
    .await
    .unwrap();
}

/// A grant of a colleague with the same role is not reached: its presentation is refused and
/// the wallet is not used.
#[tokio::test]
async fn presentation_of_a_colleagues_grant_is_refused() {
    let mut d = Doubles::default();
    d.repos
        .sent_grant
        .expect_get_by_id()
        .returning(|id| Ok(sent_grant("bea", "/admin/upm", id, true)));

    let result = PeerConnectorModule::process_oid4vp(
        &d.core(),
        &ana(),
        "g-1",
        OidcUri {
            uri: "openid4vp://peer".to_string(),
        },
    )
    .await;

    assert!(result.is_err());
}

// ==========================================================================================
// Callbacks (the peer)
// ==========================================================================================

/// A rejection callback closes the grant as rejected.
#[tokio::test]
async fn rejection_callback_rejects_the_grant() {
    let mut d = Doubles::default();
    d.repos
        .sent_grant
        .expect_get_by_id()
        .returning(|id| Ok(sent_grant("ana", "/admin/upm", id, true)));
    d.repos
        .sent_grant
        .expect_update()
        .withf(|g| g.status == GrantStatus::Rejected && g.ended_at.is_some())
        .times(1)
        .returning(Ok);

    PeerConnectorModule::manage_interaction_finish(
        &d.core(),
        "g-1",
        CallbackBody::Rejected(RejectedCallbackBody {
            rejected: "no".to_string(),
        }),
    )
    .await
    .unwrap();
}

/// A callback whose hash does not match is kept on the interaction but never continued.
#[tokio::test]
async fn forged_callback_is_not_continued() {
    let mut d = Doubles::default();
    d.repos
        .sent_interaction
        .expect_get_by_id()
        .returning(|id| Ok(sent_interaction(id)));
    d.repos
        .sent_grant
        .expect_get_by_id()
        .returning(|id| Ok(sent_grant("ana", "/admin/upm", id, true)));
    d.callback
        .expect_apply_callback()
        .returning(|interaction, body| {
            interaction.interact_ref = Some(body.interact_ref.clone());
        });
    d.callback
        .expect_check_callback()
        .returning(|_, _| Err(Errors::unauthorized("hash mismatch", None)));
    d.repos
        .sent_interaction
        .expect_update()
        .withf(|i| i.interact_ref.as_deref() == Some("ref-1"))
        .times(1)
        .returning(Ok);

    let result = PeerConnectorModule::manage_interaction_finish(
        &d.core(),
        "g-1",
        CallbackBody::Approved(ApprovedCallbackBody {
            interact_ref: "ref-1".to_string(),
            hash: "forged".to_string(),
        }),
    )
    .await;

    assert!(result.is_err());
}

// ==========================================================================================
// Token towards a peer (other agents, through the grants facade)
// ==========================================================================================

/// The token a user presents to a peer is the one of its own approved grant with it.
#[tokio::test]
async fn peer_token_is_the_one_of_the_users_own_grant() {
    let mut d = Doubles::default();
    d.repos
        .sent_grant
        .expect_get_active_access()
        .withf(|user_id, peer| user_id == "ana" && peer == "did:web:peer")
        .returning(|_, _| {
            let mut grant = sent_grant("ana", "/admin/upm", "g-1", true);
            grant.status = GrantStatus::Approved;
            grant.final_token = Some("peer-token".to_string());
            Ok(Some(grant))
        });

    let token = d.core().peer_token(&ana(), "did:web:peer").await.unwrap();

    assert_eq!(token.as_deref(), Some("peer-token"));
}

/// Without a grant of its own with the peer, the user has no token.
#[tokio::test]
async fn no_own_grant_no_token() {
    let mut d = Doubles::default();
    d.repos
        .sent_grant
        .expect_get_active_access()
        .returning(|_, _| Ok(None));

    let token = d.core().peer_token(&ana(), "did:web:peer").await.unwrap();

    assert!(token.is_none());
}
