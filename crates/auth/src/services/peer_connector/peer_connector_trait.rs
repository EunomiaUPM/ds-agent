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

use crate::types::entities::ReachProvider;
use crate::types::response::{RotationOutcome, TokenWhatResponse};
use async_trait::async_trait;
use ymir::data::entities::sent::{grant, interaction, verification};
use ymir::data::entities::shared::{participant, participant_relation, resource_req};
use ymir::errors::Outcome;
use ymir::types::gnap::grant_request::access::ResourceAccess;
use ymir::types::gnap::grant_response::GrantResponse;
use ymir::types::oauth::UserInfo;

/// Client side of GNAP towards a peer: building, sending and reading grant requests.
#[mockall::automock]
#[async_trait]
pub trait PeerConnectorTrait: Send + Sync + 'static {
    fn build_grant_plan(&self, user_info: &UserInfo, payload: ReachProvider) -> grant::Plan;
    fn build_interaction_plan(&self, id: &str) -> interaction::Plan;
    /// Resource request asking for `access`.
    fn build_resource_req_plan(&self, id: &str, access: Vec<ResourceAccess>) -> resource_req::Model;
    /// Presentation the peer asks for at `uri`.
    fn build_verification_plan(&self, uri: &str, id: &str) -> Outcome<verification::Plan>;
    /// Participant record of the peer once the grant completes.
    fn build_mate_plan(&self, grant: &grant::Model) -> participant::Plan;
    /// Relation of the user who sent the grant with the peer, under the role and visibility the
    /// grant was sent with.
    fn build_mate_relation(&self, grant: &grant::Model) -> participant_relation::Model;
    async fn send_grant_req(
        &self,
        grant: &grant::Model,
        interaction: &interaction::Model,
        resource_req: &resource_req::Model,
    ) -> Outcome<GrantResponse>;
    /// Asks for the access of `resource_req` (`PATCH`), declaring the interaction `round`.
    async fn send_modify_req(
        &self,
        previous: &interaction::Model,
        round: &interaction::Model,
        resource_req: &resource_req::Model,
    ) -> Outcome<GrantResponse>;
    /// DSP base of the peer whose gate is `grant_endpoint`, from its `.well-known/dspace-version`.
    async fn discover_dsp_base(&self, grant_endpoint: &str) -> String;
    /// Updates grant and interaction from the peer's answer and says what to do next.
    fn manage_grant_resp(
        &self,
        response: GrantResponse,
        grant: &mut grant::Model,
        interaction: &mut interaction::Model,
    ) -> Outcome<TokenWhatResponse>;
    async fn discover_gate(&self, base_url: &str) -> Outcome<String>;
    async fn send_rotation_req(&self, grant: &grant::Model) -> Outcome<GrantResponse>;
    async fn send_revocation_req(&self, grant: &grant::Model) -> Outcome<()>;
    fn apply_rotation_resp(
        &self,
        response: GrantResponse,
        grant: &mut grant::Model,
    ) -> Outcome<RotationOutcome>;
}
