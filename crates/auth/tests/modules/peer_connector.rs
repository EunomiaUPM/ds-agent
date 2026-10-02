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
//! peer asks, and following the peer's callback.

use auth::modules::PeerConnectorModule;
use auth::types::response::TokenWhatResponse;
use common::test_utils::scopes::TestScopes;
use ymir::errors::Errors;
use ymir::types::gnap::grant_request::interact::InteractAction;
use ymir::types::gnap::grant_response::GrantResponse;
use ymir::types::gnap::{ApprovedCallbackBody, CallbackBody, GrantStatus, RejectedCallbackBody};
use ymir::types::verification::VerificationStatus;
use ymir::types::wallet::OidcUri;

use crate::support::builders::{
    participant, participant_plan, reach_provider, resource_req, sent_grant, sent_grant_plan,
    sent_interaction, sent_interaction_plan, sent_verification, sent_verification_plan,
};
use crate::support::mocks::Doubles;

/// Grant `g-1` sent to the peer for `tenant-1`, which answers `what`.
fn connection(d: &mut Doubles, auto: bool, what: TokenWhatResponse) {
    d.peer_connector
        .expect_build_grant_plan()
        .withf(|tenant, reach| tenant == "tenant-1" && reach.id == "did:web:peer")
        .returning(|tenant, _| sent_grant_plan(tenant, "g-1"));
    d.peer_connector
        .expect_build_interaction_plan()
        .returning(sent_interaction_plan);
    d.peer_connector
        .expect_build_resource_req_plan()
        .withf(|_, _, actions| actions == &[InteractAction::Talk])
        .returning(|tenant, id, _| resource_req(tenant, id));
    d.repos
        .sent_grant
        .expect_create()
        .returning(move |plan| Ok(sent_grant(&plan.tenant_id, &plan.id, auto)));
    d.repos
        .sent_interaction
        .expect_create()
        .returning(|plan| Ok(sent_interaction(&plan.tenant_id, &plan.id)));
    d.repos.resource_req.expect_create().returning(Ok);
    d.peer_connector
        .expect_send_grant_req()
        .returning(|_, _, req| Ok(GrantResponse::token_approved("peer-token", req)));
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
        .withf(|tenant, uri, id| tenant == "tenant-1" && uri == "openid4vp://peer" && id == "g-1")
        .returning(|tenant, _, id| Ok(sent_verification_plan(tenant, id)));
    d.repos
        .sent_verification
        .expect_create()
        .times(1)
        .returning(|plan| Ok(sent_verification(&plan.tenant_id, &plan.id)));
}

/// A peer that grants access right away is stored as a participant.
#[tokio::test]
async fn completed_grant_stores_the_peer() {
    let mut d = Doubles::default();
    connection(&mut d, true, TokenWhatResponse::Completed);
    d.peer_connector
        .expect_build_mate_plan()
        .returning(|grant| participant_plan(&grant.tenant_id, &grant.participant_id));
    d.repos
        .participant
        .expect_force_update()
        .withf(|plan| plan.tenant_id == "tenant-1" && plan.participant_id == "did:web:peer")
        .times(1)
        .returning(|plan| Ok(participant(&plan.tenant_id, &plan.participant_id)));

    d.core()
        .req_peer_connection(&TestScopes::owner("tenant-1"), reach_provider())
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
        .req_peer_connection(&TestScopes::owner("tenant-1"), reach_provider())
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
        .req_peer_connection(&TestScopes::owner("tenant-1"), reach_provider())
        .await
        .unwrap();
}

/// A reader cannot start an onboarding.
#[tokio::test]
async fn reader_cannot_connect() {
    let result = Doubles::default()
        .core()
        .req_peer_connection(&TestScopes::reader("tenant-1"), reach_provider())
        .await;
    assert!(result.is_err());
}

/// A wallet failure marks the presentation as failed instead of failing the request.
#[tokio::test]
async fn wallet_failure_marks_the_presentation_failed() {
    let mut d = Doubles::default();
    d.repos
        .sent_verification
        .expect_get_by_id()
        .returning(|id| Ok(sent_verification("tenant-1", id)));
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
        &TestScopes::owner("tenant-1"),
        "g-1".to_string(),
        OidcUri {
            uri: "openid4vp://peer".to_string(),
        },
    )
    .await
    .unwrap();
}

/// A presentation of another tenant is not found, and the wallet is not used.
#[tokio::test]
async fn presentation_of_another_tenant_is_not_visible() {
    let mut d = Doubles::default();
    d.repos
        .sent_verification
        .expect_get_by_id()
        .returning(|id| Ok(sent_verification("tenant-2", id)));

    let result = PeerConnectorModule::process_oid4vp(
        &d.core(),
        &TestScopes::owner("tenant-1"),
        "g-1".to_string(),
        OidcUri {
            uri: "openid4vp://peer".to_string(),
        },
    )
    .await;

    assert!(result.is_err());
}

/// A rejection callback closes the grant as rejected.
#[tokio::test]
async fn rejection_callback_rejects_the_grant() {
    let mut d = Doubles::default();
    d.repos
        .sent_grant
        .expect_get_by_id()
        .returning(|id| Ok(sent_grant("tenant-1", id, true)));
    d.repos
        .sent_grant
        .expect_update()
        .withf(|g| g.status == GrantStatus::Rejected && g.ended_at.is_some())
        .times(1)
        .returning(Ok);

    PeerConnectorModule::manage_interaction_finish(
        &d.core(),
        "g-1".to_string(),
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
        .returning(|id| Ok(sent_interaction("tenant-1", id)));
    d.repos
        .sent_grant
        .expect_get_by_id()
        .returning(|id| Ok(sent_grant("tenant-1", id, true)));
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
        "g-1".to_string(),
        CallbackBody::Approved(ApprovedCallbackBody {
            interact_ref: "ref-1".to_string(),
            hash: "forged".to_string(),
        }),
    )
    .await;

    assert!(result.is_err());
}
