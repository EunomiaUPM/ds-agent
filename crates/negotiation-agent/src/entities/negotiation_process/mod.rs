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

//! Negotiation processes.

use crate::data::sea_orm::orm::negotiation_process::{
    EditNegotiationProcessModel, NewNegotiationProcessModel,
};
use serde::{Deserialize, Serialize};
use ymir::oauth::Owner;
use ymir::types::participants::Visibility;
use std::collections::HashMap;
use urn::Urn;

/// How a negotiation is matched with its peer: identifiers, pids, peer and callback.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct NegotiationCorrelation {
    pub identifiers: HashMap<String, String>,
    pub consumer_pid: Option<String>,
    pub provider_pid: Option<String>,
    pub callback_address: Option<String>,
    pub peer_participant_id: String,
}

impl NegotiationCorrelation {
    /// Identifier keys the pids are stored under.
    pub const CONSUMER_PID: &'static str = "consumerPid";
    pub const PROVIDER_PID: &'static str = "providerPid";

    /// Correlation with the pids lifted out of `identifiers`.
    pub fn new(
        identifiers: HashMap<String, String>,
        callback_address: Option<String>,
        peer_participant_id: String,
    ) -> Self {
        Self {
            consumer_pid: identifiers.get(Self::CONSUMER_PID).cloned(),
            provider_pid: identifiers.get(Self::PROVIDER_PID).cloned(),
            identifiers,
            callback_address,
            peer_participant_id,
        }
    }
}

/// New negotiation process with its identifiers.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NewNegotiationProcessDto {
    pub id: Option<Urn>,
    /// Who else sees it; private by default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<Visibility>,
    /// Owner an in-process flow asks for; only honoured for the root.
    #[serde(skip)]
    pub owner: Option<Owner>,
    pub state: String,
    pub state_attribute: Option<String>,
    pub associated_agent_peer: String,
    pub protocol: String,
    pub callback_address: Option<String>,
    pub role: String,
    pub properties: Option<serde_json::Value>,
    pub identifiers: Option<HashMap<String, String>>,
}

/// Partial process update.
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
    /// Row owned by `owner`.
    pub fn into_model(self, owner: Owner) -> NewNegotiationProcessModel {
        NewNegotiationProcessModel {
            id: self.id,
            owner,
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
