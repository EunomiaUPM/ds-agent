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

use catalog_agent::OdrlPolicyDto;
use urn::Urn;
use ymir::errors::Outcome;

pub mod local;
pub mod remote;

/// The provider's catalog, as the negotiation agent needs it: checking that an offer
/// referenced by a contract request is really published there.
#[mockall::automock]
#[async_trait::async_trait]
pub trait CatalogFacadeTrait: Send + Sync {
    /// Fails with a missing-resource error unless `tenant_id`'s catalog holds the offer.
    async fn get_offer(&self, tenant_id: &str, offer_id: &Urn) -> Outcome<OdrlPolicyDto>;
}
