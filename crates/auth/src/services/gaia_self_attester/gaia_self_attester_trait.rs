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

use async_trait::async_trait;
use ymir::errors::Outcome;
use ymir::types::jwt::JwtClaims;
use ymir::types::vcs::VcPayload;

/// Building the Gaia-X credentials this participant attests about itself.
#[mockall::automock]
#[async_trait]
pub trait GaiaSelfAttesterTrait: Send + Sync + 'static {
    /// `gx:Issuer` credential: accepts the Gaia-X terms and conditions.
    async fn generate_terms_cons_vc(&self) -> Outcome<JwtClaims<VcPayload>>;
    /// Legal person credential, from the configured company data, linked to the subject of its
    /// registration number credential.
    async fn generate_legal_person(&self, registration_number: &str) -> Outcome<JwtClaims<VcPayload>>;
}
