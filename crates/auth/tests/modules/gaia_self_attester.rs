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

//! GaiaSelfAttesterModule: issuing the connector's own Gaia-X credentials into its wallet.

use auth::modules::GaiaSelfAttesterModule;
use common::test_utils::scopes::TestScopes;
use ymir::types::issuance::VcBody;

use crate::support::builders::{stored_vc, vc_claims};
use crate::support::mocks::Doubles;

/// Both credentials are signed and stored in the wallet as JWTs.
#[tokio::test]
async fn admin_signs_and_stores_both_credentials() {
    let mut d = Doubles::default();
    d.gaia
        .expect_generate_legal_person()
        .returning(|| Ok(vc_claims("LegalPerson")));
    d.gaia
        .expect_generate_terms_cons_vc()
        .returning(|| Ok(vc_claims("TermsAndConditions")));
    d.issuer
        .expect_sign_claims()
        .times(2)
        .returning(|_| Ok("signed.jwt".to_string()));
    d.wallet
        .expect_store_vc()
        .withf(|plan| matches!(&plan.vc_body, VcBody::Jwt(jwt) if jwt == "signed.jwt"))
        .times(2)
        .returning(|_| Ok(stored_vc()));

    d.core()
        .generate_gaia_vcs(&TestScopes::admin())
        .await
        .unwrap();
}

/// The credentials attest the shared connector identity, so a tenant owner cannot issue them.
#[tokio::test]
async fn owner_cannot_issue_them() {
    let result = Doubles::default()
        .core()
        .generate_gaia_vcs(&TestScopes::owner("tenant-1"))
        .await;
    assert!(result.is_err());
}
