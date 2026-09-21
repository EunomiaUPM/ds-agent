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

use crate::data::entities::transfer_message as transfer_message_model;
use crate::data::entities::transfer_process as transfer_process_model;
use crate::data::entities::transfer_process::{EditTransferProcessModel, NewTransferProcessModel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use urn::Urn;
use ymir::errors::Outcome;

pub(crate) mod transfer_process;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransferProcessDto {
    #[serde(flatten)]
    pub inner: transfer_process_model::Model,
    pub identifiers: HashMap<String, String>,
    pub messages: Vec<transfer_message_model::Model>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NewTransferProcessDto {
    pub id: Option<Urn>,
    pub tenant_id: Option<String>,
    pub state: String,
    pub associated_agent_peer: String,
    pub protocol: String,
    pub connector_instance_id: String,
    pub transfer_direction: String,
    pub agreement_id: Urn,
    pub callback_address: Option<String>,
    pub role: String,
    pub state_attribute: Option<String>,
    pub properties: Option<serde_json::Value>,
    pub identifiers: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct EditTransferProcessDto {
    pub state: Option<String>,
    pub state_attribute: Option<String>,
    pub properties: Option<serde_json::Value>,
    pub error_details: Option<serde_json::Value>,
    pub identifiers: Option<HashMap<String, String>>,
}

impl NewTransferProcessDto {
    pub fn into_model(self, tenant_id: String) -> NewTransferProcessModel {
        NewTransferProcessModel {
            id: self.id,
            tenant_id,
            state: self.state,
            state_attribute: self.state_attribute,
            associated_agent_peer: self.associated_agent_peer,
            protocol: self.protocol,
            connector_instance_id: self.connector_instance_id,
            transfer_direction: self.transfer_direction,
            agreement_id: self.agreement_id,
            callback_address: self.callback_address,
            role: self.role,
            properties: self.properties.unwrap_or(serde_json::json!({})),
            error_details: None,
        }
    }
}

impl From<NewTransferProcessDto> for NewTransferProcessModel {
    fn from(dto: NewTransferProcessDto) -> Self {
        let tenant_id = dto.tenant_id.clone().unwrap_or_default();
        dto.into_model(tenant_id)
    }
}

impl From<EditTransferProcessDto> for EditTransferProcessModel {
    fn from(dto: EditTransferProcessDto) -> Self {
        Self {
            state: dto.state,
            state_attribute: dto.state_attribute,
            properties: dto.properties,
            error_details: dto.error_details,
        }
    }
}

use crate::entities::filters::TransferProcessFilter;
use common::paginated_spec::{Page, Paginated, Sort};

#[mockall::automock]
#[async_trait::async_trait]
pub trait TransferAgentProcessesTrait: Send + Sync + 'static {
    async fn get_all_transfer_processes(
        &self,
        filters: &TransferProcessFilter,
        page: &Page,
        sort: Sort,
    ) -> Outcome<Paginated<TransferProcessDto>>;
    async fn get_batch_transfer_processes(
        &self,
        ids: &Vec<Urn>,
    ) -> Outcome<Vec<TransferProcessDto>>;
    async fn get_transfer_process_by_id(&self, id: &Urn) -> Outcome<TransferProcessDto>;
    async fn get_transfer_process_by_key_id(
        &self,
        key_id: &str,
        id: &Urn,
    ) -> Outcome<TransferProcessDto>;
    async fn get_transfer_process_by_key_value(&self, id: &Urn) -> Outcome<TransferProcessDto>;

    async fn create_transfer_process(
        &self,
        new_model: &NewTransferProcessDto,
    ) -> Outcome<TransferProcessDto>;
    async fn put_transfer_process(
        &self,
        id: &Urn,
        edit_model: &EditTransferProcessDto,
    ) -> Outcome<TransferProcessDto>;
    async fn delete_transfer_process(&self, id: &Urn) -> Outcome<()>;
}
