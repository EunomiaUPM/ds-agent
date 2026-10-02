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

//! ParticipantModule: the peers and authorities a tenant knows, always within its own tenant.

use auth::modules::ParticipantModule;
use common::facades::VerifyTokenRequest;
use common::test_utils::scopes::TestScopes;
use serde_json::json;

use crate::support::builders::{participant, participant_plan};
use crate::support::mocks::Doubles;

/// A peer token of another tenant does not authenticate against this one.
#[tokio::test]
async fn token_of_another_tenant_is_not_visible() {
    let mut d = Doubles::default();
    d.repos
        .participant
        .expect_get_by_token()
        .withf(|token| token == "peer-token")
        .returning(|_| Ok(participant("tenant-2", "did:web:peer")));

    let result = d
        .core()
        .get_by_token(
            &TestScopes::owner("tenant-1"),
            VerifyTokenRequest {
                token: "peer-token".to_string(),
            },
        )
        .await;

    assert!(result.is_err());
}

/// New extra fields are merged into the stored ones.
#[tokio::test]
async fn extra_fields_are_merged() {
    let mut d = Doubles::default();
    d.repos
        .participant
        .expect_get_by_id()
        .withf(|tenant, id| tenant == "tenant-1" && id == "did:web:peer")
        .returning(|tenant, id| Ok(participant(tenant, id)));
    d.repos
        .participant
        .expect_update()
        .withf(|mate| mate.extra_fields == json!({"color": "blue", "size": 3}))
        .times(1)
        .returning(Ok);

    d.core()
        .update_extra_fields_by_id(
            &TestScopes::owner("tenant-1"),
            "did:web:peer".to_string(),
            json!({"size": 3}),
        )
        .await
        .unwrap();
}

/// A reader cannot change a participant.
#[tokio::test]
async fn reader_cannot_update_extra_fields() {
    let result = Doubles::default()
        .core()
        .update_extra_fields_by_id(
            &TestScopes::reader("tenant-1"),
            "did:web:peer".to_string(),
            json!({"size": 3}),
        )
        .await;
    assert!(result.is_err());
}

/// An owner always creates in its own tenant, whatever the payload says.
#[tokio::test]
async fn owner_creates_in_its_own_tenant() {
    let mut d = Doubles::default();
    d.repos
        .participant
        .expect_create()
        .withf(|plan| plan.tenant_id == "tenant-1")
        .times(1)
        .returning(|plan| Ok(participant(&plan.tenant_id, &plan.participant_id)));

    d.core()
        .create_participant(
            &TestScopes::owner("tenant-1"),
            participant_plan("tenant-2", "did:web:peer"),
        )
        .await
        .unwrap();
}

/// An admin creates in the tenant the payload names.
#[tokio::test]
async fn admin_creates_in_the_requested_tenant() {
    let mut d = Doubles::default();
    d.repos
        .participant
        .expect_create()
        .withf(|plan| plan.tenant_id == "tenant-2")
        .times(1)
        .returning(|plan| Ok(participant(&plan.tenant_id, &plan.participant_id)));

    d.core()
        .create_participant(
            &TestScopes::admin(),
            participant_plan("tenant-2", "did:web:peer"),
        )
        .await
        .unwrap();
}
