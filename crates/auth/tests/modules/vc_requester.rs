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

//! VcRequesterModule: requesting a credential from an authority and redeeming it over OID4VCI.

use std::sync::{Arc, Mutex};

use auth::modules::VcRequesterModule;
use auth::types::response::VcWhatResponse;
use common::test_utils::scopes::TestUsers;
use ymir::types::gnap::grant_response::GrantResponse;
use ymir::types::gnap::GrantStatus;
use ymir::types::oauth::UserInfo;
use ymir::types::participants::ParticipantType;
use ymir::types::wallet::OidcUri;

use crate::support::builders::{
    participant, participant_plan, reach_authority, relation, resource_req, sent_grant_plan,
    sent_interaction, sent_interaction_plan, vc_request,
};
use crate::support::mocks::Doubles;

fn ana() -> UserInfo {
    TestUsers::user("ana", "/admin/upm")
}

/// Credential request `g-1` Ana (`/admin/upm`) sends; the authority answers `what`. Returns the
/// status of every request update, in order.
fn request(d: &mut Doubles, auto: bool, what: VcWhatResponse) -> Arc<Mutex<Vec<GrantStatus>>> {
    d.vc_requester
        .expect_build_grant_plan()
        .withf(|user, reach| user.id() == "ana" && reach.id == "did:web:authority")
        .returning(|user, _| sent_grant_plan(user.id(), user.role().as_str(), "g-1"));
    d.vc_requester
        .expect_build_interaction_plan()
        .returning(|id, _| sent_interaction_plan(id));
    d.repos.sent_grant.expect_create().returning(move |plan| {
        Ok(vc_request(&plan.user_id, plan.role.as_str(), &plan.id, auto))
    });
    d.repos
        .sent_interaction
        .expect_create()
        .returning(|plan| Ok(sent_interaction(&plan.id)));
    d.vc_requester
        .expect_send_grant_req()
        .returning(|grant, _| Ok(GrantResponse::token_approved("t", &resource_req(&grant.id))));
    d.vc_requester
        .expect_manage_grant_resp()
        .return_once(move |_, _, _| Ok(what));
    let updates = Arc::new(Mutex::new(Vec::new()));
    let seen = updates.clone();
    d.repos.sent_grant.expect_update().returning(move |grant| {
        seen.lock().unwrap().push(grant.status.clone());
        Ok(grant)
    });
    d.repos.sent_interaction.expect_update().returning(Ok);
    updates
}

/// With `auto`, the offered credential is redeemed, the request finalized, and the authority
/// stored (if new) with Ana's relation to it.
#[tokio::test]
async fn automatic_issuance_redeems_and_stores_the_authority() {
    let mut d = Doubles::default();
    let updates = request(
        &mut d,
        true,
        VcWhatResponse::Issuance("openid-credential-offer://authority".to_string()),
    );
    d.wallet
        .expect_process_oid4vci()
        .withf(|uri| uri == "openid-credential-offer://authority")
        .times(1)
        .returning(|_| Ok(()));
    d.vc_requester
        .expect_build_authority_plan()
        .returning(|grant| {
            let mut plan = participant_plan(&grant.participant_id);
            plan.participant_type = ParticipantType::Authority;
            plan
        });
    d.repos
        .participant
        .expect_create_if_absent()
        .withf(|plan| plan.participant_type == ParticipantType::Authority)
        .times(1)
        .returning(|plan| Ok(participant(&plan.participant_id)));
    d.vc_requester
        .expect_build_auth_relation()
        .returning(|grant| relation(&grant.user_id, grant.role.as_str(), &grant.participant_id));
    d.repos
        .participant_relation
        .expect_force_update()
        .withf(|relation| relation.user_id == "ana" && relation.participant_id == "did:web:authority")
        .times(1)
        .returning(Ok);

    d.core().beg_vc(&ana(), reach_authority()).await.unwrap();

    assert_eq!(
        updates.lock().unwrap().last(),
        Some(&GrantStatus::Finalized)
    );
}

/// Without `auto`, the offer waits for the user and the wallet is not used.
#[tokio::test]
async fn manual_issuance_waits() {
    let mut d = Doubles::default();
    let updates = request(
        &mut d,
        false,
        VcWhatResponse::Issuance("openid-credential-offer://authority".to_string()),
    );

    d.core().beg_vc(&ana(), reach_authority()).await.unwrap();

    assert_eq!(updates.lock().unwrap().len(), 1);
}

/// The offer of a colleague's request (same role) is not reached: it fails before the wallet.
#[tokio::test]
async fn offer_of_a_colleagues_request_is_not_redeemed() {
    let mut d = Doubles::default();
    d.repos
        .sent_grant
        .expect_get_by_id()
        .returning(|id| Ok(vc_request("bea", "/admin/upm", id, false)));

    let result = d
        .core()
        .process_oid4vci(
            &ana(),
            "g-1",
            OidcUri {
                uri: "openid-credential-offer://authority".to_string(),
            },
        )
        .await;

    assert!(result.is_err());
}

/// The root reaches every request, so it can redeem any offer.
#[tokio::test]
async fn root_redeems_any_offer() {
    let mut d = Doubles::default();
    d.repos
        .sent_grant
        .expect_get_by_id()
        .returning(|id| Ok(vc_request("bea", "/admin/upm", id, false)));
    d.wallet.expect_process_oid4vci().times(1).returning(|_| Ok(()));
    d.repos.sent_grant.expect_update().returning(Ok);
    d.vc_requester
        .expect_build_authority_plan()
        .returning(|grant| participant_plan(&grant.participant_id));
    d.repos
        .participant
        .expect_create_if_absent()
        .returning(|plan| Ok(participant(&plan.participant_id)));
    d.vc_requester
        .expect_build_auth_relation()
        .returning(|grant| relation(&grant.user_id, grant.role.as_str(), &grant.participant_id));
    d.repos
        .participant_relation
        .expect_force_update()
        .returning(Ok);

    d.core()
        .process_oid4vci(
            &TestUsers::root(),
            "g-1",
            OidcUri {
                uri: "openid-credential-offer://authority".to_string(),
            },
        )
        .await
        .unwrap();
}
