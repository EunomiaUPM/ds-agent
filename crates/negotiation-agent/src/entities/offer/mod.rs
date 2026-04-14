/*
 *
 *  * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

pub(crate) mod offer;

use crate::data::entities::offer as offer_model;
use crate::data::entities::offer::NewOfferModel;
use serde::{Deserialize, Serialize};
use urn::Urn;
use ymir::errors::Outcome;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OfferDto {
    #[serde(flatten)]
    pub inner: offer_model::Model,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NewOfferDto {
    pub id: Option<Urn>,
    pub negotiation_agent_process_id: Urn,
    pub negotiation_agent_message_id: Urn,
    pub offer_id: String,
    pub offer_content: serde_json::Value,
}

impl From<NewOfferDto> for NewOfferModel {
    fn from(dto: NewOfferDto) -> Self {
        Self {
            id: dto.id,
            negotiation_agent_process_id: dto.negotiation_agent_process_id,
            negotiation_agent_message_id: dto.negotiation_agent_message_id,
            offer_id: dto.offer_id,
            offer_content: dto.offer_content,
        }
    }
}

#[mockall::automock]
#[async_trait::async_trait]
pub trait NegotiationAgentOffersTrait: Send + Sync + 'static {
    async fn get_all_offers(&self, limit: Option<u64>, page: Option<u64>)
    -> Outcome<Vec<OfferDto>>;

    async fn get_batch_offers(&self, ids: &Vec<Urn>) -> Outcome<Vec<OfferDto>>;

    async fn get_offers_by_negotiation_process(&self, id: &Urn) -> Outcome<Vec<OfferDto>>;
    async fn get_last_offer_by_negotiation_process(&self, id: &Urn) -> Outcome<Option<OfferDto>>;

    async fn get_offer_by_id(&self, id: &Urn) -> Outcome<Option<OfferDto>>;

    async fn get_offer_by_negotiation_message(&self, id: &Urn) -> Outcome<Option<OfferDto>>;

    async fn get_offer_by_offer_id(&self, id: &Urn) -> Outcome<Option<OfferDto>>;

    async fn create_offer(&self, new_model: &NewOfferDto) -> Outcome<OfferDto>;

    async fn delete_offer(&self, id: &Urn) -> Outcome<()>;
}
