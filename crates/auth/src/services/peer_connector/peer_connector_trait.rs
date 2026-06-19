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
use crate::types::entities::ReachProvider;
use crate::types::response::TokenWhatResponse;
use async_trait::async_trait;
use ymir::data::entities::sent::{grant, interaction, verification};
use ymir::data::entities::shared::{participant, resource_req};
use ymir::errors::Outcome;
use ymir::types::gnap::grant_request::interact::InteractAction;
use ymir::types::gnap::grant_response::GrantResponse;

#[async_trait]
pub trait PeerConnectorTrait: Send + Sync + 'static {
    fn build_grant_plan(&self, payload: ReachProvider) -> grant::Plan;
    fn build_interaction_plan(&self, id: &str) -> interaction::Plan;
    fn build_resource_req_plan(
        &self,
        id: &str,
        actions: Vec<InteractAction>,
    ) -> resource_req::Model;
    fn build_verification_plan(&self, id: &str, uri: &str) -> Outcome<verification::Plan>;
    fn build_mate_plan(&self, grant: &grant::Model) -> participant::Plan;
    async fn send_grant_req(
        &self,
        grant: &grant::Model,
        interaction: &interaction::Model,
        resource_req: &resource_req::Model,
    ) -> Outcome<GrantResponse>;
    fn manage_grant_resp(
        &self,
        response: GrantResponse,
        grant: &mut grant::Model,
        interaction: &mut interaction::Model,
    ) -> Outcome<TokenWhatResponse>;
}
