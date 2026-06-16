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
use crate::types::entities::ReachAuthority;
use crate::types::response::WhatResponse;
use async_trait::async_trait;
use reqwest::Response;
use std::unreachable;
use ymir::data::entities::sent::{grant, interaction, verification};
use ymir::data::entities::shared::participant;
use ymir::errors::{BadFormat, Errors, Outcome};
use ymir::types::gnap::grant_response::{GrantResponse, GrantResponseKind};
use ymir::types::gnap::GrantStatus;
use ymir::types::participants::ParticipantType;
use ymir::utils::trim_4_base;

#[async_trait]
pub trait VcRequesterTrait: Send + Sync + 'static {
    fn build_vc_plan(&self, payload: &ReachAuthority) -> Outcome<(grant::Plan, interaction::Plan)>;
    async fn send_grant_req(
        &self,
        grant: &grant::Model,
        interaction: &interaction::Model,
    ) -> Outcome<GrantResponse>;
    fn manage_grant_resp(
        &self,
        response: GrantResponse,
        grant: &mut grant::Model,
        interaction: &mut interaction::Model,
    ) -> Outcome<Option<WhatResponse>>;
    fn build_verification_plan(&self, uri: &str, id: &str) -> Outcome<verification::Plan>;
    async fn manage_res(
        &self,
        vc_req_model: &mut req_vc::Model,
        res: Response,
    ) -> Outcome<mates::NewModel>;
    fn build_authority_plan(&self, grant: &grant::Model) -> participant::Plan;
    async fn manage_rejection(&self, vc_req_model: &mut req_vc::Model) -> Outcome<()>;
}
