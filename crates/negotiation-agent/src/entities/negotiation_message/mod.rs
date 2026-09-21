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

pub(crate) mod negotiation_message;

use crate::data::entities::agreement as agreement_model;
use crate::data::entities::negotiation_message as negotiation_message_model;
use crate::data::entities::negotiation_message::NewNegotiationMessageModel;
use crate::data::entities::offer as offer_model;
use crate::entities::filters::NegotiationMessageFilter;
use common::paginated_spec::{Page, Paginated, Sort};
use serde::{Deserialize, Serialize};
use urn::Urn;
use ymir::errors::Outcome;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NegotiationMessageDto {
    #[serde(flatten)]
    pub inner: negotiation_message_model::Model,
    pub offer: Option<offer_model::Model>,
    pub agreement: Option<agreement_model::Model>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NewNegotiationMessageDto {
    pub id: Option<Urn>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    pub negotiation_agent_process_id: Urn,
    pub direction: String,
    pub protocol: String,
    pub message_type: String,
    pub state_transition_from: String,
    pub state_transition_to: String,
    pub payload: serde_json::Value,
}

impl NewNegotiationMessageDto {
    pub fn into_model(self, tenant_id: String) -> NewNegotiationMessageModel {
        NewNegotiationMessageModel {
            id: self.id,
            tenant_id,
            negotiation_agent_process_id: self.negotiation_agent_process_id,
            direction: self.direction,
            protocol: self.protocol,
            message_type: self.message_type,
            state_transition_from: self.state_transition_from,
            state_transition_to: self.state_transition_to,
            payload: self.payload,
        }
    }
}

impl From<NewNegotiationMessageDto> for NewNegotiationMessageModel {
    fn from(dto: NewNegotiationMessageDto) -> Self {
        let tenant_id = dto.tenant_id.clone().unwrap_or_default();
        dto.into_model(tenant_id)
    }
}

#[mockall::automock]
#[async_trait::async_trait]
pub trait NegotiationAgentMessagesTrait: Send + Sync + 'static {
    async fn get_all_negotiation_messages(
        &self,
        filters: &NegotiationMessageFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<NegotiationMessageDto>>;

    async fn get_messages_by_process_id(
        &self,
        process_id: &Urn,
    ) -> Outcome<Vec<NegotiationMessageDto>>;

    async fn get_negotiation_message_by_id(
        &self,
        id: &Urn,
    ) -> Outcome<Option<NegotiationMessageDto>>;

    async fn create_negotiation_message(
        &self,
        new_model_dto: &NewNegotiationMessageDto,
    ) -> Outcome<NegotiationMessageDto>;

    async fn delete_negotiation_message(&self, id: &Urn) -> Outcome<()>;
}
