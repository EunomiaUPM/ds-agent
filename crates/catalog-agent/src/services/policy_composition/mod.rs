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

//! Offers composed from templates, other offers and hand-written ODRL.

pub mod service;

use serde_json::Value;
use ymir::errors::Outcome;
use ymir::types::oauth::UserInfo;

use crate::entities::odrl_offer::OdrlOffer;
use crate::entities::policy_composition::NewPolicyCompositionDto;
use crate::entities::policy_instantiation::NewPolicyInstantiationDto;
use crate::services::policy_templates::views::PolicyPreviewView;

/// Turning parts into one plain ODRL offer.
#[mockall::automock]
#[async_trait::async_trait]
pub trait PolicyCompositionServiceTrait: Send + Sync {
    /// Composes the parts and stores the offer with what it was composed from.
    async fn compose(&self, user: &UserInfo, request: &NewPolicyCompositionDto) -> Outcome<OdrlOffer>;
    /// The composed ODRL, without storing it.
    async fn preview(&self, user: &UserInfo, parts: &[Value]) -> Outcome<PolicyPreviewView>;
    /// One template filled with values, stored as an offer: a composition of one part.
    async fn instantiate(&self, user: &UserInfo, request: &NewPolicyInstantiationDto) -> Outcome<OdrlOffer>;
}
