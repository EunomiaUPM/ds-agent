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

//! Offer repository.

use crate::data::entities::offer;
use common::oauth::{Owner, OwnerScope};
use crate::data::entities::offer::NewOfferModel;
use crate::entities::filters::OfferFilter;
use common::paginated_spec::{Page, Sort};
use thiserror::Error;
use urn::Urn;
use ymir::errors::Outcome;
use ymir::errors::RepoIntoErrors;

/// Persistence of offers, within the owner scope each call gives.
#[mockall::automock]
#[async_trait::async_trait]
pub trait OfferRepoTrait: Send + Sync {
    /// Page of offers matching the filters.
    async fn get_all_offers(
        &self,
        scope: &OwnerScope,
        filters: &OfferFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<(Vec<offer::Model>, Option<u64>)>;
    async fn get_batch_offers(
        &self,
        scope: &OwnerScope,
        ids: &[Urn],
    ) -> Outcome<Vec<offer::Model>>;
    /// Every offer made in the process.
    async fn get_offers_by_negotiation_process(
        &self,
        scope: &OwnerScope,
        id: &Urn,
    ) -> Outcome<Vec<offer::Model>>;
    /// Most recent offer of the process.
    async fn get_last_offer_by_negotiation_process(
        &self,
        scope: &OwnerScope,
        id: &Urn,
    ) -> Outcome<Option<offer::Model>>;
    async fn get_offer_by_id(
        &self,
        scope: &OwnerScope,
        id: &Urn,
    ) -> Outcome<Option<offer::Model>>;
    /// Offer carried by the message.
    async fn get_offer_by_negotiation_message(
        &self,
        scope: &OwnerScope,
        id: &Urn,
    ) -> Outcome<Option<offer::Model>>;
    /// Offer by its ODRL `@id`, not the row id.
    async fn get_offer_by_offer_id(
        &self,
        scope: &OwnerScope,
        id: &Urn,
    ) -> Outcome<Option<offer::Model>>;
    async fn create_offer(&self, new_model: &NewOfferModel) -> Outcome<offer::Model>;
    /// Returns the owner of the removed record.
    async fn delete_offer(&self, scope: &OwnerScope, id: &Urn) -> Outcome<Owner>;
}

/// Failures of the offer repository, mapped onto `Errors`.
#[derive(Debug, Error)]
pub enum OfferRepoErrors {
    #[error("Offer not found")]
    OfferNotFound,
    #[error("Error fetching offer. {0}")]
    ErrorFetchingOffer(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error creating offer. {0}")]
    ErrorCreatingOffer(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error deleting offer. {0}")]
    ErrorDeletingOffer(Box<dyn std::error::Error + Send + Sync>),
}

impl RepoIntoErrors for OfferRepoErrors {}
