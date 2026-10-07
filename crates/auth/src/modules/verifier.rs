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

use crate::services::{HasGateKeeper, HasRepo, MayHaveEventBus};
use crate::types::events::{recv_owner, GrantEvent, VerificationEvent};
use async_trait::async_trait;
use chrono::Utc;
use ymir::data::entities::received::verification;
use ymir::errors::Outcome;
use ymir::services::HasVerifier;
use ymir::types::gnap::{GrantStatus, InteractionFinishResponse};
use ymir::types::vcs::VPDef;
use ymir::types::verification::VerifyPayload;

/// Verifying the presentations peers send during onboarding.
#[async_trait]
pub trait VerifierModule:
    HasGateKeeper + HasVerifier + HasRepo + MayHaveEventBus + Send + Sync + 'static
{
    /// Presentation definition of the verification opened with `state`.
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn get_vpd(&self, state: String) -> Outcome<VPDef> {
        let verification = self.repo().recv_verification().get_by_state(&state).await?;
        self.verifier().generate_vpd(&verification)
    }
    /// Verifies the presentation and finishes the GNAP interaction with the result.
    #[tracing::instrument(level = "info", skip_all, err)]
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

        let action = match verification_result {
            Ok(_) => "verified",
            Err(_) => "failed",
        };
        let verification = self.repo().recv_verification().update(verification).await?;
        self.verifier_event(&verification, action).await;
        if verification_result.is_err() {
            self.finalize_denied(&verification.id).await?;
        }

        self.gatekeeper()
            .finish_interaction(&interaction, verification_result)
            .await
    }

    async fn finalize_denied(&self, id: &str) -> Outcome<()> {
        let mut grant = self.repo().recv_grant().get_by_id(id).await?;
        if grant.status != GrantStatus::Pending {
            return Ok(());
        }
        grant.status = GrantStatus::Finalized;
        grant.ended_at = Some(Utc::now());
        let grant = self.repo().recv_grant().update(grant).await?;
        let owner = recv_owner(&grant);
        let payload = GrantEvent::from(&grant);
        events::emit_action!(
            self.event_bus(),
            &owner,
            crate::EVENT_PREFIX,
            "gate",
            "finalized",
            &payload
        );
        Ok(())
    }

    async fn verifier_event(&self, verification: &verification::Model, action: &str) {
        if self.event_bus().is_none() {
            return;
        }
        let grant = match self.repo().recv_grant().get_by_id(&verification.id).await {
            Ok(grant) => grant,
            Err(e) => {
                tracing::warn!("No grant for verification {}: {e}", verification.id);
                return;
            }
        };
        let owner = recv_owner(&grant);
        let payload = VerificationEvent::from(verification);
        events::emit_action!(
            self.event_bus(),
            &owner,
            crate::EVENT_PREFIX,
            "verifier",
            action,
            &payload
        );
    }
}
