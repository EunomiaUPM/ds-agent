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

use crate::types::entities::ReachAuthority;
use crate::types::response::VcWhatResponse;
use async_trait::async_trait;
use ymir::data::entities::sent::{grant, interaction, verification};
use ymir::data::entities::shared::{participant, participant_relation};
use ymir::errors::Outcome;
use ymir::types::gnap::grant_request::interact::InteractStart;
use ymir::types::gnap::grant_response::GrantResponse;
use ymir::types::oauth::UserInfo;

/// Client side of GNAP towards an authority: building, sending and reading credential requests.
#[mockall::automock]
#[async_trait]
pub trait VcRequesterTrait: Send + Sync + 'static {
    fn build_grant_plan(&self, user_info: &UserInfo, payload: ReachAuthority) -> grant::Plan;
    /// Interaction started the way the request asks, such as a redirect.
    fn build_interaction_plan(&self, id: &str, start: InteractStart) -> interaction::Plan;
    /// Presentation the authority asks for at `uri`.
    fn build_verification_plan(&self, uri: &str, id: &str) -> Outcome<verification::Plan>;
    /// Participant record of the authority once the credential is issued.
    fn build_authority_plan(&self, grant: &grant::Model) -> participant::Plan;
    fn build_auth_relation(
        &self,
        user_info: &UserInfo,
        holder: &str,
    ) -> participant_relation::Model;
    async fn send_grant_req(
        &self,
        grant: &grant::Model,
        interaction: &interaction::Model,
    ) -> Outcome<GrantResponse>;
    /// Updates grant and interaction from the authority's answer and says what to do next.
    fn manage_grant_resp(
        &self,
        response: GrantResponse,
        grant: &mut grant::Model,
        interaction: &mut interaction::Model,
    ) -> Outcome<VcWhatResponse>;
}
