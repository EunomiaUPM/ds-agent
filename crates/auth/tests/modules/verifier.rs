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

//! VerifierModule: checking a peer's presentation and closing its GNAP interaction with the
//! result, whether it passed or not.

use auth::modules::VerifierModule;
use ymir::errors::Errors;
use ymir::types::gnap::InteractionFinishResponse;
use ymir::types::verification::{VerificationStatus, VerifyPayload};

use crate::support::builders::{recv_interaction, recv_verification};
use crate::support::mocks::Doubles;

fn payload() -> VerifyPayload {
    VerifyPayload {
        vp_token: "vp-token".to_string(),
        presentation_submission: "{}".to_string(),
    }
}

/// The verification opened with `state-1`, for grant `g-1`.
fn opened(d: &mut Doubles) {
    d.repos
        .recv_verification
        .expect_get_by_state()
        .withf(|state| state == "state-1")
        .returning(|_| Ok(recv_verification("g-1", None)));
    d.repos
        .recv_interaction
        .expect_get_by_id()
        .withf(|id| id == "g-1")
        .returning(|id| Ok(recv_interaction(id)));
}

/// The verification is stored as the verifier left it and the interaction finishes with success.
#[tokio::test]
async fn valid_presentation_finishes_the_interaction() {
    let mut d = Doubles::default();
    opened(&mut d);
    d.verifier
        .expect_verify_all()
        .withf(|_, token| token == "vp-token")
        .returning(|verification, _| {
            verification.holder = Some("did:web:peer".to_string());
            verification.status = VerificationStatus::Verified;
            Ok(())
        });
    d.repos
        .recv_verification
        .expect_update()
        .withf(|v| v.status == VerificationStatus::Verified && v.holder.is_some())
        .times(1)
        .returning(Ok);
    d.gatekeeper
        .expect_finish_interaction()
        .withf(|_, result| result.is_ok())
        .returning(|_, _| Ok(InteractionFinishResponse::Success(None)));

    let response = d
        .core()
        .verify("state-1".to_string(), payload())
        .await
        .unwrap();

    assert!(matches!(response, InteractionFinishResponse::Success(_)));
}

/// A rejected presentation is still stored and the peer's interaction finishes with the failure.
#[tokio::test]
async fn rejected_presentation_finishes_the_interaction_with_failure() {
    let mut d = Doubles::default();
    opened(&mut d);
    d.verifier
        .expect_verify_all()
        .returning(|_, _| Err(Errors::unauthorized("expired credential", None)));
    d.repos
        .recv_verification
        .expect_update()
        .times(1)
        .returning(Ok);
    d.gatekeeper
        .expect_finish_interaction()
        .withf(|_, result| result.is_err())
        .returning(|_, _| Ok(InteractionFinishResponse::Failure(None)));

    let response = d
        .core()
        .verify("state-1".to_string(), payload())
        .await
        .unwrap();

    assert!(matches!(response, InteractionFinishResponse::Failure(_)));
}
