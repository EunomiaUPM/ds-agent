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

use crate::data::entities::agreement as agreement_model;
use crate::data::entities::negotiation_message as negotiation_message_model;
use crate::data::entities::negotiation_process as negotiation_process_model;
use crate::data::entities::negotiation_process::{
    EditNegotiationProcessModel, NewNegotiationProcessModel,
};
use crate::data::entities::offer as offer_model;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use urn::Urn;
use ymir::errors::Outcome;

pub(crate) mod negotiation_process;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NegotiationProcessDto {
    #[serde(flatten)]
    pub inner: negotiation_process_model::Model,
    pub identifiers: HashMap<String, String>,
    pub messages: Vec<negotiation_message_model::Model>,
    pub offers: Vec<offer_model::Model>,
    pub agreement: Option<agreement_model::Model>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NewNegotiationProcessDto {
    pub id: Option<Urn>,
    pub state: String,
    pub state_attribute: Option<String>,
    pub associated_agent_peer: String,
    pub protocol: String,
    pub callback_address: Option<String>,
    pub role: String,
    pub properties: Option<serde_json::Value>,
    pub identifiers: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct EditNegotiationProcessDto {
    pub state: Option<String>,
    pub state_attribute: Option<String>,
    pub properties: Option<serde_json::Value>,
    pub error_details: Option<serde_json::Value>,
    pub identifiers: Option<HashMap<String, String>>,
}

impl From<NewNegotiationProcessDto> for NewNegotiationProcessModel {
    fn from(dto: NewNegotiationProcessDto) -> Self {
        Self {
            id: dto.id,
            state: dto.state,
            state_attribute: dto.state_attribute,
            associated_agent_peer: dto.associated_agent_peer,
            protocol: dto.protocol,
            callback_address: dto.callback_address,
            role: dto.role,
            properties: dto.properties.unwrap_or(serde_json::json!({})),
            error_details: None,
        }
    }
}

impl From<EditNegotiationProcessDto> for EditNegotiationProcessModel {
    fn from(dto: EditNegotiationProcessDto) -> Self {
        Self {
            state: dto.state,
            state_attribute: dto.state_attribute,
            properties: dto.properties,
            error_details: dto.error_details,
        }
    }
}

#[mockall::automock]
#[async_trait::async_trait]
pub trait NegotiationAgentProcessesTrait: Send + Sync + 'static {
    async fn get_all_negotiation_processes(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<NegotiationProcessDto>>;

    async fn get_batch_negotiation_processes(
        &self,
        ids: &Vec<Urn>,
    ) -> Outcome<Vec<NegotiationProcessDto>>;

    async fn get_negotiation_process_by_id(
        &self,
        id: &Urn,
    ) -> Outcome<Option<NegotiationProcessDto>>;

    async fn get_negotiation_process_by_key_id(
        &self,
        key_id: &str,
        id: &Urn,
    ) -> Outcome<Option<NegotiationProcessDto>>;

    async fn get_negotiation_process_by_key_value(
        &self,
        id: &Urn,
    ) -> Outcome<Option<NegotiationProcessDto>>;

    async fn create_negotiation_process(
        &self,
        new_model: &NewNegotiationProcessDto,
    ) -> Outcome<NegotiationProcessDto>;

    async fn put_negotiation_process(
        &self,
        id: &Urn,
        edit_model: &EditNegotiationProcessDto,
    ) -> Outcome<NegotiationProcessDto>;

    async fn delete_negotiation_process(&self, id: &Urn) -> Outcome<()>;
}
