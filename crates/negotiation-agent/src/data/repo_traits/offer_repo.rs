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

use crate::data::entities::offer;
use crate::data::entities::offer::NewOfferModel;
use thiserror::Error;
use urn::Urn;
use ymir::errors::Outcome;
use ymir::errors::RepoIntoErrors;

#[async_trait::async_trait]
pub trait OfferRepoTrait: Send + Sync {
    async fn get_all_offers(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<offer::Model>>;
    async fn get_batch_offers(&self, ids: &Vec<Urn>) -> Outcome<Vec<offer::Model>>;
    async fn get_offers_by_negotiation_process(&self, id: &Urn) -> Outcome<Vec<offer::Model>>;
    async fn get_last_offer_by_negotiation_process(
        &self,
        id: &Urn,
    ) -> Outcome<Option<offer::Model>>;
    async fn get_offer_by_id(&self, id: &Urn) -> Outcome<Option<offer::Model>>;
    async fn get_offer_by_negotiation_message(&self, id: &Urn) -> Outcome<Option<offer::Model>>;
    async fn get_offer_by_offer_id(&self, id: &Urn) -> Outcome<Option<offer::Model>>;
    async fn create_offer(&self, new_model: &NewOfferModel) -> Outcome<offer::Model>;
    async fn delete_offer(&self, id: &Urn) -> Outcome<()>;
}

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
