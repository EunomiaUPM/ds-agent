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
use rainbow_common::auth::business::RainbowBusinessLoginRequest;
use ymir::services::verifier::VerifierTrait;

use crate::ssi::services::business::BusinessTrait;
use crate::ssi::services::repo::repo_trait::AuthRepoTrait;
use crate::ssi::types::business::BusinessResponse;

#[async_trait]
pub trait CoreBusinessTrait: Send + Sync + 'static {
    fn business(&self) -> Arc<dyn BusinessTrait>;
    fn repo(&self) -> Arc<dyn AuthRepoTrait>;
    fn verifier(&self) -> Arc<dyn VerifierTrait>;
    async fn login(&self, payload: RainbowBusinessLoginRequest) -> anyhow::Result<String> {
        let (req_model, ver_model) = self.business().start(&payload);
        self.repo().request_rcv().create(req_model).await?;
        let ver_model = self.repo().verification_rcv().create_from_basic(ver_model).await?;
        let uri = self.verifier().generate_verification_uri(ver_model);
        Ok(uri)
    }
    async fn token(
        &self,
        payload: RainbowBusinessLoginRequest,
    ) -> anyhow::Result<BusinessResponse> {
        let bus_model = self.repo().business_mates().get_by_id(&payload.auth_request_id).await?;
        let mate = self.repo().mates().get_by_id(&bus_model.participant_id).await?;
        self.business().get_token(&mate, &bus_model)
    }
}
