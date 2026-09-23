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

use crate::data::entities::negotiation_process::{
    EditNegotiationProcessModel, NewNegotiationProcessModel,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use urn::Urn;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NewNegotiationProcessDto {
    pub id: Option<Urn>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
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

impl NewNegotiationProcessDto {
    pub fn into_model(self, tenant_id: String) -> NewNegotiationProcessModel {
        NewNegotiationProcessModel {
            id: self.id,
            tenant_id,
            state: self.state,
            state_attribute: self.state_attribute,
            associated_agent_peer: self.associated_agent_peer,
            protocol: self.protocol,
            callback_address: self.callback_address,
            role: self.role,
            properties: self.properties.unwrap_or(serde_json::json!({})),
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
