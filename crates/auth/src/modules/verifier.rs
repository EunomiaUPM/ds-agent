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

use crate::services::{HasEventSink, HasGateKeeper, HasRepo};
use crate::types::events::AuthEvents;
use events::schemas::identity::{GrantAction, VerificationAction};
use async_trait::async_trait;
use chrono::Utc;
use ymir::data::entities::received::verification;
use ymir::errors::{Errors, Outcome};
use ymir::services::HasVerifier;
use ymir::types::gnap::{GrantStatus, InteractionFinishResponse};
use ymir::types::vcs::VPDef;
use ymir::types::verification::{VerificationStatus, VerifyPayload};

/// Verifying the presentations peers send during onboarding.
#[async_trait]
pub trait VerifierModule:
    HasGateKeeper + HasVerifier + HasRepo + HasEventSink + Send + Sync + 'static
{
    /// Presentation definition of the verification opened with `state`.
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn get_vpd(&self, state: String) -> Outcome<VPDef> {
        let verification = self.repo().recv_verification().get_by_state(&state).await?;
        self.verifier().generate_vpd(&verification)
    }
    /// Signed Request Object of the verification opened with `state` (OpenID4VP 1.0
    /// `request_uri`), echoing the wallet's `wallet_nonce`, if any.
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn request_object(&self, state: String, wallet_nonce: Option<String>) -> Outcome<String> {
        let verification = self.repo().recv_verification().get_by_state(&state).await?;
        self.verifier().request_object(&verification, wallet_nonce).await
    }
    /// Verifies the presentation and finishes the GNAP interaction with the result.
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn verify(
        &self,
        state: String,
        payload: VerifyPayload,
    ) -> Outcome<InteractionFinishResponse> {
        let mut verification = self.repo().recv_verification().get_by_state(&state).await?;
        let verification_result = match (payload.vp_token(), payload.refusal()) {
            (Some(vp_token), _) => self
                .verifier()
                .verify_all(&mut verification, vp_token)
                .await
                .map(|_| ()),
            (None, refusal) => {
                verification.ended_at = Some(Utc::now());
                verification.status = VerificationStatus::Refused;
                Err(refusal.unwrap_or_else(|| Errors::forbidden("no presentation", None)))
            }
        };

        let interaction = self
            .repo()
            .recv_interaction()
            .get_latest_by_grant(&verification.grant_id)
            .await?;

        let (action, failure_reason) = match &verification_result {
            Ok(_) => (VerificationAction::Verified, None),
            Err(err) => (VerificationAction::Failed, Some(err.to_string())),
        };
        let verification = self.repo().recv_verification().update(verification).await?;
        self.verifier_event(&verification, action, failure_reason)
            .await;
        if verification_result.is_err() {
            self.finalize_denied(&verification.grant_id).await?;
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
        AuthEvents::recv_grant(
            self.event_sink().as_ref(),
            &grant,
            GrantAction::Finalized,
            None,
            Some("verification failed".to_string()),
        )
        .await;
        Ok(())
    }

    async fn verifier_event(
        &self,
        verification: &verification::Model,
        action: VerificationAction,
        failure_reason: Option<String>,
    ) {
        let grant = match self.repo().recv_grant().get_by_id(&verification.grant_id).await {
            Ok(grant) => grant,
            Err(e) => {
                tracing::warn!("No grant for verification {}: {e}", verification.id);
                return;
            }
        };
        AuthEvents::recv_verification(
            self.event_sink().as_ref(),
            &grant,
            verification,
            action,
            failure_reason,
        )
        .await;
    }
}
