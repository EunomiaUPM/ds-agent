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

//! Agreements.

use crate::data::entities::agreement::{EditAgreementModel, NewAgreementModel};
use common::oauth::{Owner, Visibility};
use serde::{Deserialize, Serialize};
use urn::Urn;

/// New agreement.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NewAgreementDto {
    pub id: Option<Urn>,
    /// Who else sees it; private by default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<Visibility>,
    /// Owner an in-process flow asks for; only honoured for the root.
    #[serde(skip)]
    pub owner: Option<Owner>,
    pub negotiation_agent_process_id: Urn,
    pub negotiation_agent_message_id: Urn,
    pub consumer_participant_id: String,
    pub provider_participant_id: String,
    pub agreement_content: serde_json::Value,
    pub target: Urn,
}

/// Agreement update.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct EditAgreementDto {
    pub state: Option<String>,
}

impl NewAgreementDto {
    /// Row owned by `owner`.
    pub fn into_model(self, owner: Owner) -> NewAgreementModel {
        NewAgreementModel {
            id: self.id,
            owner,
            negotiation_agent_process_id: self.negotiation_agent_process_id,
            negotiation_agent_message_id: self.negotiation_agent_message_id,
            consumer_participant_id: self.consumer_participant_id,
            provider_participant_id: self.provider_participant_id,
            agreement_content: self.agreement_content,
            target: self.target,
        }
    }
}

impl From<EditAgreementDto> for EditAgreementModel {
    fn from(dto: EditAgreementDto) -> Self {
        Self { state: dto.state }
    }
}
