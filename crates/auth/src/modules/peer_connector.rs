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
use crate::services::peer_connector::PeerConnectorTrait;
use crate::services::repo::repo_trait::AuthRepoTrait;
use crate::services::{HasCallback, HasPeerConnector, HasRepo};
use crate::types::entities::ReachProvider;
use crate::types::response::TokenWhatResponse;
use async_trait::async_trait;
use chrono::Utc;
use ymir::data::entities::sent::grant;
use ymir::errors::Outcome;
use ymir::services::HasWallet;
use ymir::types::gnap::grant_request::GrantKind;
use ymir::types::gnap::{ApprovedCallbackBody, CallbackBody, GrantStatus};
use ymir::types::verifying::VerificationStatus;

#[async_trait]
pub trait PeerConnectorModule:
    HasPeerConnector + HasRepo + HasCallback + HasWallet + Send + Sync + 'static
{
    async fn req_peer_connection(&self, payload: ReachProvider) -> Outcome<()> {
        let actions = payload.actions.clone();
        let grant = self.peer_connector().build_grant_plan(payload);
        let interaction = self.peer_connector().build_interaction_plan(&grant.id);
        let resource_req = self
            .peer_connector()
            .build_resource_req_plan(&grant.id, actions);

        let mut grant = self.repo().sent_grant().create(grant).await?;
        let mut interaction = self.repo().sent_interaction().create(interaction).await?;
        let resource_req = self.repo().resource_req().create(resource_req).await?;

        let grant_resp = self
            .peer_connector()
            .send_grant_req(&grant, &interaction, &resource_req)
            .await?;

        let what_response =
            self.peer_connector()
                    .manage_grant_resp(grant_resp, &mut grant, &mut interaction);

        let grant = self.repo().sent_grant().update(grant).await?;
        let _interaction = self.repo().sent_interaction().update(interaction).await?;

        self.manage_what_resp(grant, what_response).await
    }

    async fn manage_interaction_finish(&self, id: String, payload: CallbackBody) -> Outcome<()> {
        match payload {
            CallbackBody::Approved(payload) => self.req_peer_continuation(id, payload).await,
            CallbackBody::Rejected(_payload) => self.manage_rejection(id).await,
        }
    }

    // =================================== GETTERS FOR FRONTEND ====================================
    async fn get_all(&self) -> Outcome<Vec<grant::Model>> {
        self.repo()
            .sent_grant()
            .get_by_type(GrantKind::AccessToken)
            .await
    }

    async fn get_by_id(&self, id: String) -> Outcome<grant::Model> {
        self.repo().sent_grant().get_by_id(&id).await
    }

    // ========================================= INTERNALS =========================================
    async fn manage_what_resp(
        &self,
        grant: grant::Model,
        vc_what_response: Outcome<TokenWhatResponse>,
    ) -> Outcome<()> {
        match vc_what_response? {
            TokenWhatResponse::Completed => {
                let mate = self.peer_connector().build_mate_plan(&grant);
                self.repo().participant().force_update(mate).await?;
                Ok(())
            }
            TokenWhatResponse::Presentation(uri) => self.manage_oid4vp(&grant.id, grant.auto, &uri).await,
            TokenWhatResponse::Wait => Ok(()),
        }
    }

    async fn manage_oid4vp(&self, id: &str, auto: bool, uri: &str) -> Outcome<()> {
        let verification = self.peer_connector().build_verification_plan(&uri, id)?;
        let mut verification = self.repo().sent_verification().create(verification).await?;

        if auto {
            match self.wallet().process_oid4vp(&uri).await {
                Ok(_) => verification.status = VerificationStatus::Verified,
                Err(_) => {
                    verification.status = VerificationStatus::Failed;
                }
            }
            verification.ended_at = Some(Utc::now());
            self.repo().sent_verification().update(verification).await?;
        }
        Ok(())
    }

    async fn req_peer_continuation(
        &self,
        id: String,
        payload: ApprovedCallbackBody,
    ) -> Outcome<()> {
        let mut interaction = self.repo().sent_interaction().get_by_id(&id).await?;
        let mut grant = self.repo().sent_grant().get_by_id(&id).await?;
        self.callback().apply_callback(&mut interaction, &payload);

        let result = self.callback().check_callback(&interaction, &grant);
        let mut interaction = self.repo().sent_interaction().update(interaction).await?;
        result?;

        let grant_resp = self.callback().send_continue_req(&interaction).await?;

        let what_response =
            self.peer_connector()
                .manage_grant_resp(grant_resp, &mut grant, &mut interaction);

        let grant = self.repo().sent_grant().update(grant).await?;
        let _interaction = self.repo().sent_interaction().update(interaction).await?;

        self.manage_what_resp(grant, what_response).await
    }

    async fn manage_rejection(&self, id: String) -> Outcome<()> {
        let mut grant = self.repo().sent_grant().get_by_id(&id).await?;
        grant.status = GrantStatus::Rejected;
        grant.ended_at = Some(chrono::Utc::now());
        self.repo().sent_grant().update(grant).await?;
        Ok(())
    }


}
