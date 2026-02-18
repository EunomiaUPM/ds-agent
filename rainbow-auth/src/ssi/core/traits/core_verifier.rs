/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use std::sync::Arc;

use async_trait::async_trait;
use ymir::services::verifier::VerifierTrait;
use ymir::types::vcs::VPDef;
use ymir::types::verifying::VerifyPayload;

use crate::ssi::services::business::BusinessTrait;
use crate::ssi::services::repo::repo_trait::AuthRepoTrait;

#[async_trait]
pub trait CoreVerifierTrait: Send + Sync + 'static {
    fn verifier(&self) -> Arc<dyn VerifierTrait>;
    fn repo(&self) -> Arc<dyn AuthRepoTrait>;
    fn business(&self) -> Arc<dyn BusinessTrait>;
    async fn get_vpd(&self, state: String) -> anyhow::Result<VPDef> {
        let ver_model = self.repo().verification_rcv().get_by_state(&state).await?;
        Ok(self.verifier().generate_vpd(ver_model))
    }
    async fn verify(
        &self,
        state: String,
        payload: VerifyPayload,
    ) -> anyhow::Result<Option<String>> {
        let mut ver_model = self.repo().verification_rcv().get_by_state(&state).await?;
        let result = self.verifier().verify_all(&mut ver_model, payload.vp_token).await;
        match self.repo().interaction_rcv().get_by_some_id(&ver_model.id).await? {
            Some(int_model) => {
                self.repo().verification_rcv().update(ver_model).await?;
                result?;
                self.verifier().end_verification(&int_model).await
            }
            None => {
                let ver_model = self.repo().verification_rcv().update(ver_model).await?;
                result?;
                let business_mate = self.business().end(&ver_model)?;
                self.repo().business_mates().create(business_mate).await?;
                Ok(None)
            }
        }
    }
}
