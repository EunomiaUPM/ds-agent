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

use async_trait::async_trait;
use reqwest::Response;
use ymir::data::entities::req_request;
use ymir::data::entities::{mates, req_interaction, req_verification, token_requirements};

use crate::ssi::types::entities::ReachProvider;

#[async_trait]
pub trait OnboarderTrait: Send + Sync + 'static {
    fn start(
        &self,
        payload: &ReachProvider
    ) -> (
        req_request::NewModel,
        req_interaction::NewModel,
        token_requirements::Model
    );
    async fn send_req(
        &self,
        req_model: &mut req_request::Model,
        int_model: &mut req_interaction::Model
    ) -> anyhow::Result<()>;
    fn save_verification(
        &self,
        int_model: &req_interaction::Model
    ) -> anyhow::Result<req_verification::NewModel>;
    async fn manage_res(
        &self,
        req_model: &mut req_request::Model,
        res: Response
    ) -> anyhow::Result<mates::NewModel>;
    async fn manage_rejection(&self, model: &mut req_request::Model) -> anyhow::Result<()>;
}
