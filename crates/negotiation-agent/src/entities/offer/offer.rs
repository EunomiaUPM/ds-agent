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

use crate::data::entities::offer::NewOfferModel;
use crate::data::factory_trait::NegotiationAgentRepoTrait;
use crate::entities::offer::{NegotiationAgentOffersTrait, NewOfferDto, OfferDto};
use std::sync::Arc;
use urn::Urn;
use ymir::errors::Outcome;

pub struct NegotiationAgentOffersService {
    pub negotiation_repo: Arc<dyn NegotiationAgentRepoTrait>,
}

impl NegotiationAgentOffersService {
    pub fn new(negotiation_repo: Arc<dyn NegotiationAgentRepoTrait>) -> Self {
        Self { negotiation_repo }
    }
}

#[async_trait::async_trait]
impl NegotiationAgentOffersTrait for NegotiationAgentOffersService {
    async fn get_all_offers(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<OfferDto>> {
        let offers = self
            .negotiation_repo
            .get_offer_repo()
            .get_all_offers(limit, page)
            .await?;

        Ok(offers.into_iter().map(|m| OfferDto { inner: m }).collect())
    }

    async fn get_batch_offers(&self, ids: &Vec<Urn>) -> Outcome<Vec<OfferDto>> {
        let offers = self
            .negotiation_repo
            .get_offer_repo()
            .get_batch_offers(ids)
            .await?;

        Ok(offers.into_iter().map(|m| OfferDto { inner: m }).collect())
    }

    async fn get_offers_by_negotiation_process(&self, id: &Urn) -> Outcome<Vec<OfferDto>> {
        let offers = self
            .negotiation_repo
            .get_offer_repo()
            .get_offers_by_negotiation_process(id)
            .await?;

        Ok(offers.into_iter().map(|m| OfferDto { inner: m }).collect())
    }

    async fn get_last_offer_by_negotiation_process(&self, id: &Urn) -> Outcome<Option<OfferDto>> {
        let offers = self
            .negotiation_repo
            .get_offer_repo()
            .get_last_offer_by_negotiation_process(id)
            .await?;

        Ok(offers.map(|m| OfferDto { inner: m }))
    }

    async fn get_offer_by_id(&self, id: &Urn) -> Outcome<Option<OfferDto>> {
        let offer = self
            .negotiation_repo
            .get_offer_repo()
            .get_offer_by_id(id)
            .await?;

        Ok(offer.map(|m| OfferDto { inner: m }))
    }

    async fn get_offer_by_negotiation_message(&self, id: &Urn) -> Outcome<Option<OfferDto>> {
        let offer = self
            .negotiation_repo
            .get_offer_repo()
            .get_offer_by_negotiation_message(id)
            .await?;

        Ok(offer.map(|m| OfferDto { inner: m }))
    }

    async fn get_offer_by_offer_id(&self, id: &Urn) -> Outcome<Option<OfferDto>> {
        let offer = self
            .negotiation_repo
            .get_offer_repo()
            .get_offer_by_offer_id(id)
            .await?;

        Ok(offer.map(|m| OfferDto { inner: m }))
    }

    async fn create_offer(&self, new_model_dto: &NewOfferDto) -> Outcome<OfferDto> {
        let new_model: NewOfferModel = new_model_dto.clone().into();

        let created = self
            .negotiation_repo
            .get_offer_repo()
            .create_offer(&new_model)
            .await?;

        Ok(OfferDto { inner: created })
    }

    async fn delete_offer(&self, id: &Urn) -> Outcome<()> {
        self.negotiation_repo
            .get_offer_repo()
            .delete_offer(id)
            .await?;
        Ok(())
    }
}
