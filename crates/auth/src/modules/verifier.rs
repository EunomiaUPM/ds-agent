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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use crate::services::{HasGateKeeper, HasRepo};
use async_trait::async_trait;
use ymir::errors::Outcome;
use ymir::services::HasVerifier;
use ymir::types::gnap::InteractionFinishResponse;
use ymir::types::vcs::VPDef;
use ymir::types::verification::VerifyPayload;

#[async_trait]
pub trait VerifierModule: HasGateKeeper + HasVerifier + HasRepo + Send + Sync + 'static {
    async fn get_vpd(&self, state: String) -> Outcome<VPDef> {
        let verification = self.repo().recv_verification().get_by_state(&state).await?;
        self.verifier().generate_vpd(&verification)
    }
    async fn verify(
        &self,
        state: String,
        payload: VerifyPayload,
    ) -> Outcome<InteractionFinishResponse> {
        println!("{:#?}", payload.vp_token);
        let mut verification = self.repo().recv_verification().get_by_state(&state).await?;
        let verification_result = self
            .verifier()
            .verify_all(&mut verification, &payload.vp_token)
            .await;

        let interaction = self
            .repo()
            .recv_interaction()
            .get_by_id(&verification.id)
            .await?;

        self.repo().recv_verification().update(verification).await?;

        self.gatekeeper()
            .finish_interaction(&interaction, verification_result)
            .await
    }
}
