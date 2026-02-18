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
use ymir::data::entities::req_interaction;
use ymir::types::gnap::ApprovedCallbackBody;

#[async_trait]
pub trait CallbackTrait: Send + Sync + 'static {
    fn check_callback(
        &self,
        int_model: &mut req_interaction::Model,
        payload: &ApprovedCallbackBody,
    ) -> anyhow::Result<()>;
    async fn continue_req(&self, int_model: &req_interaction::Model) -> anyhow::Result<Response>;
}
