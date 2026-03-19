/*
 *
 * * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 * *
 * * This program is free software: you can redistribute it and/or modify
 * * it under the terms of the GNU General Public License as published by
 * * the Free Software Foundation, either version 3 of the License, or
 * * (at your option) any later version.
 * *
 * * This program is distributed in the hope that it will be useful,
 * * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * * GNU General Public License for more details.
 * *
 * * You should have received a copy of the GNU General Public License
 * * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::data::entities::offer;
use crate::data::entities::offer::NewOfferModel;
use anyhow::Error;
use thiserror::Error;
use urn::Urn;
use ymir::errors::RepoIntoErrors;

#[async_trait::async_trait]
pub trait OfferRepoTrait: Send + Sync {
    async fn get_all_offers(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> anyhow::Result<Vec<offer::Model>, OfferRepoErrors>;
    async fn get_batch_offers(
        &self,
        ids: &Vec<Urn>,
    ) -> anyhow::Result<Vec<offer::Model>, OfferRepoErrors>;
    async fn get_offers_by_negotiation_process(
        &self,
        id: &Urn,
    ) -> anyhow::Result<Vec<offer::Model>, OfferRepoErrors>;
    async fn get_last_offer_by_negotiation_process(
        &self,
        id: &Urn,
    ) -> anyhow::Result<Option<offer::Model>, OfferRepoErrors>;
    async fn get_offer_by_id(
        &self,
        id: &Urn,
    ) -> anyhow::Result<Option<offer::Model>, OfferRepoErrors>;
    async fn get_offer_by_negotiation_message(
        &self,
        id: &Urn,
    ) -> anyhow::Result<Option<offer::Model>, OfferRepoErrors>;
    async fn get_offer_by_offer_id(
        &self,
        id: &Urn,
    ) -> anyhow::Result<Option<offer::Model>, OfferRepoErrors>;
    async fn create_offer(
        &self,
        new_model: &NewOfferModel,
    ) -> anyhow::Result<offer::Model, OfferRepoErrors>;
    async fn delete_offer(&self, id: &Urn) -> anyhow::Result<(), OfferRepoErrors>;
}

#[derive(Debug, Error)]
pub enum OfferRepoErrors {
    #[error("Offer not found")]
    OfferNotFound,
    #[error("Error fetching offer. {0}")]
    ErrorFetchingOffer(Error),
    #[error("Error creating offer. {0}")]
    ErrorCreatingOffer(Error),
    #[error("Error deleting offer. {0}")]
    ErrorDeletingOffer(Error),
}

impl RepoIntoErrors for OfferRepoErrors {}
