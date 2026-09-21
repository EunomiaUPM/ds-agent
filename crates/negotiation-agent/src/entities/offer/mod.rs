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

pub(crate) mod offer;

use crate::data::entities::offer as offer_model;
use crate::data::entities::offer::NewOfferModel;
use crate::entities::filters::OfferFilter;
use common::paginated_spec::{Page, Paginated, Sort};
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    pub negotiation_agent_process_id: Urn,
    pub negotiation_agent_message_id: Urn,
    pub offer_id: String,
    pub offer_content: serde_json::Value,
}

impl NewOfferDto {
    pub fn into_model(self, tenant_id: String) -> NewOfferModel {
        NewOfferModel {
            id: self.id,
            tenant_id,
            negotiation_agent_process_id: self.negotiation_agent_process_id,
            negotiation_agent_message_id: self.negotiation_agent_message_id,
            offer_id: self.offer_id,
            offer_content: self.offer_content,
        }
    }
}

impl From<NewOfferDto> for NewOfferModel {
    fn from(dto: NewOfferDto) -> Self {
        let tenant_id = dto.tenant_id.clone().unwrap_or_default();
        dto.into_model(tenant_id)
    }
}

#[mockall::automock]
#[async_trait::async_trait]
pub trait NegotiationAgentOffersTrait: Send + Sync + 'static {
    async fn get_all_offers(
        &self,
        filters: &OfferFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<OfferDto>>;

    async fn get_batch_offers(&self, ids: &Vec<Urn>) -> Outcome<Vec<OfferDto>>;

    async fn get_offers_by_negotiation_process(&self, id: &Urn) -> Outcome<Vec<OfferDto>>;
    async fn get_last_offer_by_negotiation_process(&self, id: &Urn) -> Outcome<Option<OfferDto>>;

    async fn get_offer_by_id(&self, id: &Urn) -> Outcome<Option<OfferDto>>;

    async fn get_offer_by_negotiation_message(&self, id: &Urn) -> Outcome<Option<OfferDto>>;

    async fn get_offer_by_offer_id(&self, id: &Urn) -> Outcome<Option<OfferDto>>;

    async fn create_offer(&self, new_model: &NewOfferDto) -> Outcome<OfferDto>;

    async fn delete_offer(&self, id: &Urn) -> Outcome<()>;
}
