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

pub(crate) mod agreement;

use crate::data::entities::agreement as agreement_model;
use crate::data::entities::agreement::{EditAgreementModel, NewAgreementModel};
use crate::entities::filters::AgreementFilter;
use common::paginated_spec::{Page, Paginated, Sort};
use serde::{Deserialize, Serialize};
use urn::Urn;
use ymir::errors::Outcome;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AgreementDto {
    #[serde(flatten)]
    pub inner: agreement_model::Model,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NewAgreementDto {
    pub id: Option<Urn>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    pub negotiation_agent_process_id: Urn,
    pub negotiation_agent_message_id: Urn,
    pub consumer_participant_id: String,
    pub provider_participant_id: String,
    pub agreement_content: serde_json::Value,
    pub target: Urn,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct EditAgreementDto {
    pub state: Option<String>,
}

impl NewAgreementDto {
    pub fn into_model(self, tenant_id: String) -> NewAgreementModel {
        NewAgreementModel {
            id: self.id,
            tenant_id,
            negotiation_agent_process_id: self.negotiation_agent_process_id,
            negotiation_agent_message_id: self.negotiation_agent_message_id,
            consumer_participant_id: self.consumer_participant_id,
            provider_participant_id: self.provider_participant_id,
            agreement_content: self.agreement_content,
            target: self.target,
        }
    }
}

impl From<NewAgreementDto> for NewAgreementModel {
    fn from(dto: NewAgreementDto) -> Self {
        let tenant_id = dto.tenant_id.clone().unwrap_or_default();
        dto.into_model(tenant_id)
    }
}

impl From<EditAgreementDto> for EditAgreementModel {
    fn from(dto: EditAgreementDto) -> Self {
        Self { state: dto.state }
    }
}

#[mockall::automock]
#[async_trait::async_trait]
pub trait NegotiationAgentAgreementsTrait: Send + Sync + 'static {
    async fn get_all_agreements(
        &self,
        filters: &AgreementFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<AgreementDto>>;

    async fn get_batch_agreements(&self, ids: &Vec<Urn>) -> Outcome<Vec<AgreementDto>>;

    async fn get_agreement_by_id(&self, id: &Urn) -> Outcome<Option<AgreementDto>>;

    async fn get_agreement_by_negotiation_process(&self, id: &Urn)
    -> Outcome<Option<AgreementDto>>;

    async fn get_agreement_by_negotiation_message(&self, id: &Urn)
    -> Outcome<Option<AgreementDto>>;

    async fn get_agreements_by_assignee(&self, id: &String) -> Outcome<Vec<AgreementDto>>;

    async fn get_agreements_by_assigner(&self, id: &String) -> Outcome<Vec<AgreementDto>>;

    async fn create_agreement(&self, new_model: &NewAgreementDto) -> Outcome<AgreementDto>;

    async fn put_agreement(&self, id: &Urn, edit_model: &EditAgreementDto)
    -> Outcome<AgreementDto>;

    async fn delete_agreement(&self, id: &Urn) -> Outcome<()>;
}
