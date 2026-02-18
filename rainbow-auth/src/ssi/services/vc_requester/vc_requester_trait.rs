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
use ymir::data::entities::{mates, req_interaction, req_vc, req_verification};
use ymir::types::gnap::grant_request::InteractStart;

use crate::ssi::types::entities::ReachAuthority;

#[async_trait]
pub trait VcRequesterTrait: Send + Sync + 'static {
    fn start(
        &self,
        payload: ReachAuthority,
        reach_method: InteractStart,
    ) -> (req_vc::NewModel, req_interaction::NewModel);
    async fn send_req(
        &self,
        vc_model: &mut req_vc::Model,
        int_model: &mut req_interaction::Model,
    ) -> anyhow::Result<Option<String>>;
    fn save_ver_data(&self, uri: &str, id: &str) -> anyhow::Result<req_verification::NewModel>;
    async fn manage_res(
        &self,
        vc_req_model: &mut req_vc::Model,
        res: Response,
    ) -> anyhow::Result<mates::NewModel>;
    async fn manage_rejection(&self, vc_req_model: &mut req_vc::Model) -> anyhow::Result<()>;
}
