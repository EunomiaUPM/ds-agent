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
use ymir::types::gnap::grant_request::GrantRequest;
use ymir::types::gnap::grant_response::GrantResponse;
use ymir::types::gnap::{AccessToken, RefBody};

use crate::ssi::services::gatekeeper::GateKeeperTrait;
use crate::ssi::services::repo::repo_trait::AuthRepoTrait;

#[async_trait]
pub trait CoreGateKeeperTrait: Send + Sync + 'static {
    fn gatekeeper(&self) -> Arc<dyn GateKeeperTrait>;
    fn verifier(&self) -> Arc<dyn VerifierTrait>;
    fn repo(&self) -> Arc<dyn AuthRepoTrait>;

    async fn manage_req(&self, payload: GrantRequest) -> anyhow::Result<GrantResponse> {
        let (req_model, int_model, token_model) = self.gatekeeper().start(&payload)?;
        let req_model = self.repo().request_rcv().create(req_model).await?;
        let int_model = self.repo().interaction_rcv().create(int_model).await?;
        let _token_model = self.repo().token_requirements().create(token_model).await?;
        let ver_model = self.verifier().start_vp(&req_model.id)?;
        let ver_model = self.repo().verification_rcv().create(ver_model).await?;
        let uri = self.verifier().generate_verification_uri(ver_model);
        Ok(self.gatekeeper().respond_req(&int_model, &uri))
    }

    async fn continue_req(
        &self,
        id: String,
        payload: RefBody,
        token: String,
    ) -> anyhow::Result<AccessToken> {
        let int_model = self.repo().interaction_rcv().get_by_cont_id(&id).await?;
        let mut req_model = self.repo().request_rcv().get_by_id(&int_model.id).await?;
        let ver_model = self.repo().verification_rcv().get_by_id(&int_model.id).await?;
        self.gatekeeper().validate_cont_req(&int_model, &payload, &token)?;
        let (mate, token_response) =
            self.gatekeeper().continue_req(&mut req_model, &int_model, &ver_model);
        let _mate = self.repo().mates().force_create(mate).await?;
        Ok(token_response)
    }
}
