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

use rainbow_common::auth::business::RainbowBusinessLoginRequest;
use ymir::data::entities::{business_mates, mates, recv_request, recv_verification};

use crate::ssi::types::business::BusinessResponse;

pub trait BusinessTrait: Send + Sync + 'static {
    fn start(
        &self,
        payload: &RainbowBusinessLoginRequest,
    ) -> (recv_request::NewModel, recv_verification::Model);
    fn get_token(
        &self,
        mate: &mates::Model,
        bus_model: &business_mates::Model,
    ) -> anyhow::Result<BusinessResponse>;
    fn end(&self, ver_model: &recv_verification::Model)
        -> anyhow::Result<business_mates::NewModel>;
}
