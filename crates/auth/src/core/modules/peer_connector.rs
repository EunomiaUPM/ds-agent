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

use crate::core::traits::{HasCallback, HasPeerConnector, HasRepo};
use crate::services::callback::CallbackTrait;
use crate::services::peer_connector::PeerConnectorTrait;
use crate::services::repo::repo_trait::AuthRepoTrait;
use crate::types::entities::ReachProvider;
use async_trait::async_trait;
use ymir::data::entities::sent::grant;
use ymir::errors::Outcome;
use ymir::modules::HasWallet;
use ymir::types::gnap::grant_request::GrantKind;
use ymir::types::gnap::ApprovedCallbackBody;

#[async_trait]
pub trait PeerConnectorModuleTrait:
    HasPeerConnector + HasRepo + HasCallback + HasWallet + Send + Sync + 'static
{
    async fn onboard_req(&self, payload: ReachProvider) -> Outcome<Option<String>> {
        let plan = self.peer_connector().build_peer_plan(&payload);
        let mut grant = self.repo().sent_grant().create(plan.grant).await?;
        let mut interaction = self
            .repo()
            .sent_interaction()
            .create(plan.interaction)
            .await?;
        let resource_req = self.repo().resource_req().create(plan.resource_req).await?;

        let grant_resp = self
            .peer_connector()
            .send_grant_req(&grant, &interaction, &resource_req)
            .await?;

        let result =
            self.peer_connector()
                .manage_grant_resp(grant_resp, &mut grant, &mut interaction);

        let grant = self.repo().sent_grant().update(grant).await?;
        let interaction = self.repo().sent_interaction().update(interaction).await?;
        let oid4vp_uri = result?;

        let plan = self
            .peer_connector()
            .build_verification_plan(&interaction.id, &oid4vp_uri)?;

        let verification = self.repo().sent_verification().create(plan).await?;

        if grant.auto {
            self.wallet().process_oid4vp(&verification.uri).await?;
            Ok(None)
        } else {
            Ok(Some(verification.uri))
        }
    }

    async fn continue_req(&self, id: &str, payload: ApprovedCallbackBody) -> Outcome<()> {
        let mut interaction = self.repo().interaction_req().get_by_id(id).await?;
        let mut grant = self.repo().sent_grant().get_by_id(id).await?;
        self.callback().apply_callback(&mut interaction, &payload);

        let result = self.callback().check_callback(&mut interaction, &grant);
        let interaction = self.repo().sent_interaction().update(interaction).await?;
        result?;

        let response = self.callback().send_continue_req(&interaction).await?;
        let result = self.peer_connector().manage_cont_resp(response, &mut grant);

        self.repo().sent_grant().update(grant).await?;
        result?;

        let mate = self.peer_connector().build_mate_plan(&grant);

        let mate = self.repo().participants().force_create(mate).await?;

        Ok(mate)
    }

    async fn manage_rejection(&self, id: String) -> Outcome<()> {
        let mut grant = self.repo().sent_grant().get_by_id(&id).await?;
        self.peer_connector().manage_rejection(&mut grant).await?;
        self.repo().sent_grant().update(grant).await?;
        Ok(())
    }

    async fn get_all(&self) -> Outcome<Vec<grant::Model>> {
        self.repo()
            .sent_grant()
            .get_by_type(GrantKind::AccessToken)
            .await
    }

    async fn get_by_id(&self, id: String) -> Outcome<grant::Model> {
        self.repo().sent_grant().get_by_id(&id).await
    }
}
