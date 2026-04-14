/*
 *
 * * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
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

pub(crate) mod negotiation_message;

use crate::data::entities::agreement as agreement_model;
use crate::data::entities::negotiation_message as negotiation_message_model;
use crate::data::entities::negotiation_message::NewNegotiationMessageModel;
use crate::data::entities::offer as offer_model;
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
    pub negotiation_agent_process_id: Urn,
    pub direction: String,
    pub protocol: String,
    pub message_type: String,
    pub state_transition_from: String,
    pub state_transition_to: String,
    pub payload: serde_json::Value,
}

impl From<NewNegotiationMessageDto> for NewNegotiationMessageModel {
    fn from(dto: NewNegotiationMessageDto) -> Self {
        Self {
            id: dto.id,
            negotiation_agent_process_id: dto.negotiation_agent_process_id,
            direction: dto.direction,
            protocol: dto.protocol,
            message_type: dto.message_type,
            state_transition_from: dto.state_transition_from,
            state_transition_to: dto.state_transition_to,
            payload: dto.payload,
        }
    }
}

#[mockall::automock]
#[async_trait::async_trait]
pub trait NegotiationAgentMessagesTrait: Send + Sync + 'static {
    async fn get_all_negotiation_messages(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<NegotiationMessageDto>>;

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
