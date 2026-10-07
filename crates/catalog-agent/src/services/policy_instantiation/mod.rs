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

//! Instantiates ODRL policies from policy templates.

pub mod service;

use crate::entities::policy_instantiation::NewPolicyInstantiationDto;
use crate::OdrlPolicyDto;
use common::oauth::UserInfo;
use ymir::errors::Outcome;

/// Turning a policy template into a concrete ODRL offer.
#[async_trait::async_trait]
pub trait PolicyInstantiationServiceTrait: Send + Sync {
    /// Fills the template's parameters with the given values and stores the offer.
    async fn instantiate_policy(
        &self,
        user: &UserInfo,
        instantiation_request: &NewPolicyInstantiationDto,
    ) -> Outcome<OdrlPolicyDto>;
}
