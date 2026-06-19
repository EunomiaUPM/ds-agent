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

use crate::services::callback::CallbackTrait;
use crate::services::repo::repo_trait::AuthRepoTrait;
use crate::services::vc_requester::VcRequesterTrait;
use crate::services::{HasCallback, HasRepo, HasVcRequester};
use crate::types::entities::ReachAuthority;
use crate::types::response::VcWhatResponse;
use async_trait::async_trait;
use ymir::data::entities::sent::grant;
use ymir::errors::Outcome;
use ymir::services::HasWallet;
use ymir::types::gnap::grant_request::GrantKind;
use ymir::types::gnap::{ApprovedCallbackBody, CallbackBody, GrantStatus};
use ymir::types::verifying::VerificationStatus;

#[async_trait]
pub trait VcRequesterModuleTrait:
    HasVcRequester + HasRepo + HasCallback + HasWallet + Send + Sync + 'static
{
    async fn beg_vc(&self, payload: ReachAuthority) -> Outcome<()> {
        let start = payload.method.clone();
        let grant = self.vc_requester().build_grant_plan(payload);
        let interaction = self.vc_requester().build_interaction_plan(&grant.id, start);

        let mut grant = self.repo().sent_grant().create(grant).await?;
        let mut interaction = self.repo().sent_interaction().create(interaction).await?;

        let grant_resp = self
            .vc_requester()
            .send_grant_req(&grant, &interaction)
            .await?;

        let what_response =
            self.vc_requester()
                .manage_grant_resp(grant_resp, &mut grant, &mut interaction);

        let grant = self.repo().sent_grant().update(grant).await?;
        let _interaction = self.repo().sent_interaction().update(interaction).await?;

        self.manage_grant_resp(grant, what_response)
    }

    async fn manage_interaction_finish(&self, id: String, payload: CallbackBody) -> Outcome<()> {
        match payload {
            CallbackBody::Approved(payload) => self.req_vc_continuation(id, payload),
            CallbackBody::Rejected(_payload) => self.manage_rejection(id),
        }
    }

    async fn req_vc_continuation(&self, id: String, payload: ApprovedCallbackBody) -> Outcome<()> {
        let mut interaction = self.repo().sent_interaction().get_by_id(&id).await?;
        let mut grant = self.repo().sent_grant().get_by_id(&id).await?;
        self.callback().apply_callback(&mut interaction, &payload);

        let result = self.callback().check_callback(&interaction, &grant);
        let mut interaction = self.repo().sent_interaction().update(interaction).await?;
        result?;

        let grant_resp = self.callback().send_continue_req(&interaction).await?;

        let what_response =
            self.vc_requester()
                .manage_grant_resp(grant_resp, &mut grant, &mut interaction);

        let grant = self.repo().sent_grant().update(grant).await?;
        let _interaction = self.repo().sent_interaction().update(interaction).await?;

        self.manage_grant_resp(grant, what_response)
    }

    async fn manage_rejection(&self, id: String) -> Outcome<()> {
        let mut grant = self.repo().sent_grant().get_by_id(&id).await?;
        grant.status = GrantStatus::Rejected;
        grant.ended_at = Some(chrono::Utc::now());
        self.repo().sent_grant().update(grant).await?;
        Ok(())
    }

    async fn manage_grant_resp(
        &self,
        grant: grant::Model,
        vc_what_response: Outcome<VcWhatResponse>,
    ) -> Outcome<()> {
        match vc_what_response? {
            VcWhatResponse::Issuance(uri) => self.manage_oid4vci(grant, &uri),
            VcWhatResponse::Presentation(uri) => self.manage_oid4vp(&grant.id, grant.auto, &uri),
            VcWhatResponse::Wait => Ok(()),
        }
    }

    async fn manage_oid4vci(&self, mut grant: grant::Model, uri: &str) -> Outcome<()> {
        self.wallet().process_oid4vci(uri).await?;

        grant.status = GrantStatus::Finalized;
        let grant = self.repo().sent_grant().update(grant).await?;

        let authority = self.vc_requester().build_authority_plan(&grant);
        self.repo().participant().force_update(authority).await?;

        Ok(())
    }
    async fn manage_oid4vp(&self, id: &str, auto: bool, uri: &str) -> Outcome<()> {
        self.wallet().process_oid4vp(uri).await?;

        let verification = self.vc_requester().build_verification_plan(&uri, id)?;
        let mut verification = self.repo().sent_verification().create(verification).await?;

        if auto {
            match self.wallet().process_oid4vp(&uri).await {
                Ok(_) => verification.status = VerificationStatus::Verified,
                Err(_) => {
                    verification.status = VerificationStatus::Failed;
                }
            }
            self.repo().sent_verification().update(verification).await?;
        }
        Ok(())
    }
    async fn get_all(&self) -> Outcome<Vec<grant::Model>> {
        self.repo()
            .sent_grant()
            .get_by_type(GrantKind::CredentialRequest)
            .await
    }

    async fn get_by_id(&self, id: String) -> Outcome<grant::Model> {
        self.repo().vc_req().get_by_id(&id).await
    }
}
