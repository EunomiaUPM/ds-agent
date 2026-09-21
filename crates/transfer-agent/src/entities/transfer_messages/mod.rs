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

use crate::data::entities::transfer_message::{
    self as transfer_message_model, NewTransferMessageModel,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use urn::Urn;
use ymir::errors::Outcome;

pub(crate) mod transfer_messages;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferMessageDto {
    #[serde(flatten)]
    pub inner: transfer_message_model::Model,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NewTransferMessageDto {
    pub id: Option<Urn>,
    pub tenant_id: Option<String>,
    pub transfer_agent_process_id: Urn,
    pub direction: String,
    pub protocol: String,
    pub message_type: String,
    pub state_transition_from: String,
    pub state_transition_to: String,
    pub payload: Option<Json>,
}

impl NewTransferMessageDto {
    pub fn into_model(self, tenant_id: String) -> NewTransferMessageModel {
        NewTransferMessageModel {
            id: self.id,
            tenant_id,
            transfer_agent_process_id: self.transfer_agent_process_id,
            direction: self.direction,
            protocol: self.protocol,
            message_type: self.message_type,
            state_transition_from: self.state_transition_from,
            state_transition_to: self.state_transition_to,
            payload: self.payload,
        }
    }
}

impl From<NewTransferMessageDto> for NewTransferMessageModel {
    fn from(dto: NewTransferMessageDto) -> Self {
        let tenant_id = dto.tenant_id.clone().unwrap_or_default();
        dto.into_model(tenant_id)
    }
}

use crate::entities::filters::TransferMessageFilter;
use common::paginated_spec::{Page, Paginated, Sort};

#[mockall::automock]
#[async_trait::async_trait]
pub trait TransferAgentMessagesTrait: Send + Sync + 'static {
    async fn get_all_transfer_messages(
        &self,
        filters: &TransferMessageFilter,
        page: &Page,
        sort: Sort,
    ) -> Outcome<Paginated<TransferMessageDto>>;

    async fn get_messages_by_process_id(
        &self,
        process_id: &Urn,
    ) -> Outcome<Vec<TransferMessageDto>>;

    async fn get_transfer_message_by_id(&self, id: &Urn) -> Outcome<TransferMessageDto>;

    async fn create_transfer_message(
        &self,
        new_model: &NewTransferMessageDto,
    ) -> Outcome<TransferMessageDto>;

    async fn delete_transfer_message(&self, id: &Urn) -> Outcome<()>;
}
