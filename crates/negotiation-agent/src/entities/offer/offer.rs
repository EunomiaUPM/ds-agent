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

use crate::data::entities::offer::NewOfferModel;
use crate::data::factory_trait::NegotiationAgentRepoTrait;
use crate::entities::filters::{NegotiationMessageFilter, OfferFilter};
use crate::entities::offer::{NegotiationAgentOffersTrait, NewOfferDto, OfferDto};
use common::paginated_spec::{Cursor, Page, Paginated, Sort};
use common::query::QueryFilter;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::Outcome;

pub struct NegotiationAgentOffersService {
    pub negotiation_repo: Arc<dyn NegotiationAgentRepoTrait>,
    pub event_bus: Option<events::EventBus>,
}

impl NegotiationAgentOffersService {
    pub fn new(negotiation_repo: Arc<dyn NegotiationAgentRepoTrait>) -> Self {
        Self {
            negotiation_repo,
            event_bus: None,
        }
    }

    pub fn with_event_bus(mut self, event_bus: Option<events::EventBus>) -> Self {
        self.event_bus = event_bus;
        self
    }
}

#[async_trait::async_trait]
impl NegotiationAgentOffersTrait for NegotiationAgentOffersService {
    async fn get_all_offers(
        &self,
        filters: &OfferFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<OfferDto>> {
        filters.validate()?;
        let page = page.clamped();

        let (offers, total) = self
            .negotiation_repo
            .get_offer_repo()
            .get_all_offers(filters, &page, sort)
            .await?;

        let dtos: Vec<OfferDto> = offers.into_iter().map(|m| OfferDto { inner: m }).collect();

        Ok(Paginated::from_page(dtos, &page, total, |d| {
            Cursor::encode_composite(&d.inner.created_at, &d.inner.id)
        }))
    }

    async fn get_batch_offers(&self, ids: &Vec<Urn>) -> Outcome<Vec<OfferDto>> {
        let mut dtos = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(dto) = self.get_offer_by_id(id).await? {
                dtos.push(dto);
            }
        }
        Ok(dtos)
    }

    async fn get_offers_by_negotiation_process(&self, id: &Urn) -> Outcome<Vec<OfferDto>> {
        let process_opt = self
            .negotiation_repo
            .get_negotiation_process_repo()
            .get_negotiation_process_by_key_value(None, id)
            .await?;
        let Some(process) = process_opt else {
            return Ok(vec![]);
        };

        let offers = self
            .negotiation_repo
            .get_offer_repo()
            .get_offers_by_negotiation_process(&process.tenant_id, id)
            .await?;

        Ok(offers.into_iter().map(|m| OfferDto { inner: m }).collect())
    }

    async fn get_last_offer_by_negotiation_process(&self, id: &Urn) -> Outcome<Option<OfferDto>> {
        let process_opt = self
            .negotiation_repo
            .get_negotiation_process_repo()
            .get_negotiation_process_by_key_value(None, id)
            .await?;
        let Some(process) = process_opt else {
            return Ok(None);
        };

        let offers = self
            .negotiation_repo
            .get_offer_repo()
            .get_last_offer_by_negotiation_process(&process.tenant_id, id)
            .await?;

        Ok(offers.map(|m| OfferDto { inner: m }))
    }

    async fn get_offer_by_id(&self, id: &Urn) -> Outcome<Option<OfferDto>> {
        let filter = OfferFilter {
            id: Some(id.to_string()),
            ..Default::default()
        };
        let page = Page {
            limit: 1,
            ..Default::default()
        };
        let (offers, _) = self
            .negotiation_repo
            .get_offer_repo()
            .get_all_offers(&filter, &page, &Sort::default())
            .await?;

        Ok(offers.into_iter().next().map(|m| OfferDto { inner: m }))
    }

    async fn get_offer_by_negotiation_message(&self, id: &Urn) -> Outcome<Option<OfferDto>> {
        let msg_opt = self
            .negotiation_repo
            .get_negotiation_message_repo()
            .get_all_negotiation_messages(
                &NegotiationMessageFilter {
                    id: Some(id.to_string()),
                    ..Default::default()
                },
                &Page {
                    limit: 1,
                    ..Default::default()
                },
                &Sort::default(),
            )
            .await?;
        let Some(msg) = msg_opt.0.into_iter().next() else {
            return Ok(None);
        };

        let offer = self
            .negotiation_repo
            .get_offer_repo()
            .get_offer_by_negotiation_message(&msg.tenant_id, id)
            .await?;

        Ok(offer.map(|m| OfferDto { inner: m }))
    }

    async fn get_offer_by_offer_id(&self, id: &Urn) -> Outcome<Option<OfferDto>> {
        let filter = OfferFilter {
            offer_id: Some(id.to_string()),
            ..Default::default()
        };
        let page = Page {
            limit: 1,
            ..Default::default()
        };
        let (offers, _) = self
            .negotiation_repo
            .get_offer_repo()
            .get_all_offers(&filter, &page, &Sort::default())
            .await?;

        Ok(offers.into_iter().next().map(|m| OfferDto { inner: m }))
    }

    async fn create_offer(&self, new_model_dto: &NewOfferDto) -> Outcome<OfferDto> {
        let new_model: NewOfferModel = new_model_dto.clone().into();

        let created = self
            .negotiation_repo
            .get_offer_repo()
            .create_offer(&new_model)
            .await?;

        let dto = OfferDto { inner: created };
        events::emit_action!(self.event_bus, crate::EVENT_PREFIX, "offer", "create", &dto);
        Ok(dto)
    }

    async fn delete_offer(&self, id: &Urn) -> Outcome<()> {
        if let Some(offer) = self.get_offer_by_id(id).await? {
            self.negotiation_repo
                .get_offer_repo()
                .delete_offer(&offer.inner.tenant_id, id)
                .await?;
        }
        events::emit_action!(
            self.event_bus,
            crate::EVENT_PREFIX,
            "offer",
            "delete",
            &events::EntityDeletedDto::new(id)
        );
        Ok(())
    }
}
