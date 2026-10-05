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

//! ParticipantModule: the peers and authorities the connector knows (global), and who added
//! each one (a relation per user, with its visibility).

use auth::modules::ParticipantModule;
use common::test_utils::scopes::TestUsers;
use serde_json::json;
use ymir::types::participants::Visibility;

use crate::support::builders::{participant, participant_plan};
use crate::support::mocks::Doubles;

/// New extra fields are merged into the stored ones.
#[tokio::test]
async fn extra_fields_are_merged() {
    let mut d = Doubles::default();
    d.repos
        .participant
        .expect_get_visible()
        .withf(|_, id| id == "did:web:peer")
        .returning(|_, id| Ok(participant(id)));
    d.repos
        .participant
        .expect_update()
        .withf(|mate| mate.extra_fields == json!({"color": "blue", "size": 3}))
        .times(1)
        .returning(Ok);

    d.core()
        .update_extra_fields_by_id(&TestUsers::root(), "did:web:peer", json!({"size": 3}))
        .await
        .unwrap();
}

/// The participant is shared by the whole organization, so only the root changes it.
#[tokio::test]
async fn only_the_root_updates_extra_fields() {
    let result = Doubles::default()
        .core()
        .update_extra_fields_by_id(
            &TestUsers::user("ana", "/admin/upm"),
            "did:web:peer",
            json!({"size": 3}),
        )
        .await;
    assert!(result.is_err());
}

/// Adding a participant stores it (if new) and the relation of the user who added it, under
/// its role and with the visibility it chose.
#[tokio::test]
async fn adding_stores_the_participant_and_the_users_relation() {
    let mut d = Doubles::default();
    d.repos
        .participant
        .expect_create_if_absent()
        .withf(|plan| plan.participant_id == "did:web:peer")
        .times(1)
        .returning(|plan| Ok(participant(&plan.participant_id)));
    d.repos
        .participant_relation
        .expect_force_update()
        .withf(|relation| {
            relation.user_id == "ana"
                && relation.participant_id == "did:web:peer"
                && relation.role.as_str() == "/admin/upm"
                && relation.visibility == Visibility::Anonymous
        })
        .times(1)
        .returning(Ok);

    d.core()
        .create_participant(
            &TestUsers::user("ana", "/admin/upm"),
            participant_plan("did:web:peer"),
            Visibility::Anonymous,
        )
        .await
        .unwrap();
}

/// A participant the user does not see is not found.
#[tokio::test]
async fn participant_not_seen_is_not_found() {
    let mut d = Doubles::default();
    d.repos
        .participant
        .expect_get_visible()
        .returning(|_, id| Err(ymir::errors::Errors::missing_resource(id, "not found", None)));

    let result = ParticipantModule::get_by_id(
        &d.core(),
        &TestUsers::user("ana", "/admin/upm"),
        "did:web:peer",
    )
    .await;

    assert!(result.is_err());
}
