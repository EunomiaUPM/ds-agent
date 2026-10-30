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

//! The catalog agent's published contract: what other agents and the connector read from it.

pub mod remote;
pub mod views;

use async_trait::async_trait;
use urn::Urn;
use ymir::errors::Outcome;
use ymir::types::oauth::UserInfo;

pub use views::{CatalogDataset, CatalogDistribution, CatalogOffer};

/// Datasets, distributions and offers of the catalog. Lookups without a user read whoever owns
/// the record: an agreement or an offer shown to a peer already fixed it.
#[mockall::automock]
#[async_trait]
pub trait CatalogFacadeTrait: Send + Sync {
    async fn get_dataset(&self, dataset_id: &Urn) -> Outcome<CatalogDataset>;

    /// The dataset's distribution in `format` (`dct:format`).
    async fn get_distribution_by_format(
        &self,
        dataset_id: &Urn,
        format: &str,
    ) -> Outcome<CatalogDistribution>;

    /// A distribution `user` sees; a missing-resource error otherwise.
    async fn get_distribution(
        &self,
        user: &UserInfo,
        distribution_id: &Urn,
    ) -> Outcome<CatalogDistribution>;

    /// A published offer; a missing-resource error unless the catalog holds it and its dataset
    /// is not a draft, so a draft is never negotiated.
    async fn get_offer(&self, offer_id: &Urn) -> Outcome<CatalogOffer>;
}

#[cfg(test)]
mod tests;
