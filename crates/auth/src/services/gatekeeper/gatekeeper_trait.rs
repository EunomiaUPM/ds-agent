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

use axum::body::Bytes;
use axum::http::HeaderMap;
use ymir::data::entities::received::{grant, interaction};
use ymir::data::entities::shared::{participant, resource_req};
use ymir::errors::Outcome;
use ymir::types::gnap::grant_request::GrantRequest;
use ymir::types::gnap::grant_response::GrantResponse;

pub trait GateKeeperTrait: Send + Sync + 'static {
    fn validate_grant(&self, payload: &Bytes, headers: &HeaderMap) -> Outcome<GrantRequest>;
    fn build_grant_plan(
        &self,
        payload: GrantRequest,
    ) -> Outcome<(grant::Plan, interaction::Plan, resource_req::Model)>;

    fn respond_grant_pending(&self, interaction: &interaction::Model, uri: &str) -> GrantResponse;
    fn validate_cont_req(
        &self,
        model: &interaction::Model,
        payload: &Bytes,
        headers: &HeaderMap,
    ) -> Outcome<()>;
    async fn finish_interaction(&self, model: &interaction::Model) -> Outcome<Option<String>>;
    fn end_req(
        &self,
        grant: &mut grant::Model,
        resource_req: &resource_req::Model,
        token: &str,
    ) -> GrantResponse;
    fn build_mate_plan(
        &self,
        holder: &str,
        nick: &str,
        base_url: &str,
        token: &str,
    ) -> participant::Plan;
}
