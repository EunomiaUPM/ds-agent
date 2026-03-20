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

use axum::body::Bytes;
use axum::http::HeaderMap;
use ymir::data::entities::{
    mates, recv_interaction, recv_request, recv_verification, token_requirements,
};
use ymir::errors::Outcome;
use ymir::types::gnap::grant_request::GrantRequest;
use ymir::types::gnap::grant_response::GrantResponse;
use ymir::types::gnap::AccessToken;

pub trait GateKeeperTrait: Send + Sync + 'static {
    fn start(
        &self,
        payload: &Bytes,
        headers: &HeaderMap,
    ) -> Outcome<(
        recv_request::NewModel,
        recv_interaction::NewModel,
        token_requirements::Model,
    )>;
    fn validate_req(&self, payload: &Bytes, headers: &HeaderMap) -> Outcome<GrantRequest>;
    fn respond_req(&self, int_model: &recv_interaction::Model, uri: &str) -> GrantResponse;
    fn validate_cont_req(
        &self,
        model: &recv_interaction::Model,
        payload: &Bytes,
        headers: &HeaderMap,
    ) -> Outcome<()>;
    fn continue_req(
        &self,
        req_model: &mut recv_request::Model,
        int_model: &recv_interaction::Model,
        token_model: &token_requirements::Model,
        ver_model: &recv_verification::Model,
    ) -> (mates::NewModel, AccessToken);
}
