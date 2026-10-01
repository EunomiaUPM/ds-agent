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

use async_trait::async_trait;
use ymir::data::entities::sent::{grant, interaction};
use ymir::errors::Outcome;
use ymir::types::gnap::grant_response::GrantResponse;
use ymir::types::gnap::ApprovedCallbackBody;

#[async_trait]
pub trait CallbackTrait: Send + Sync + 'static {
    fn apply_callback(&self, interaction: &mut interaction::Model, payload: &ApprovedCallbackBody);
    fn check_callback(&self, interaction: &interaction::Model, grant: &grant::Model)
        -> Outcome<()>;
    async fn send_continue_req(&self, int_model: &interaction::Model) -> Outcome<GrantResponse>;
}
