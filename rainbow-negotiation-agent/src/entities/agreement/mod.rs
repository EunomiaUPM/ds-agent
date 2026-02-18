/*
 *
 * * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

pub(crate) mod agreement;

use crate::data::entities::agreement as agreement_model;
use crate::data::entities::agreement::{EditAgreementModel, NewAgreementModel};
use serde::{Deserialize, Serialize};
use urn::Urn;

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

impl From<NewAgreementDto> for NewAgreementModel {
    fn from(dto: NewAgreementDto) -> Self {
        Self {
            id: dto.id,
            negotiation_agent_process_id: dto.negotiation_agent_process_id,
            negotiation_agent_message_id: dto.negotiation_agent_message_id,
            consumer_participant_id: dto.consumer_participant_id,
            provider_participant_id: dto.provider_participant_id,
            agreement_content: dto.agreement_content,
            target: dto.target,
        }
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
        limit: Option<u64>,
        page: Option<u64>,
    ) -> anyhow::Result<Vec<AgreementDto>>;

    async fn get_batch_agreements(&self, ids: &Vec<Urn>) -> anyhow::Result<Vec<AgreementDto>>;

    async fn get_agreement_by_id(&self, id: &Urn) -> anyhow::Result<Option<AgreementDto>>;

    async fn get_agreement_by_negotiation_process(
        &self,
        id: &Urn,
    ) -> anyhow::Result<Option<AgreementDto>>;

    async fn get_agreement_by_negotiation_message(
        &self,
        id: &Urn,
    ) -> anyhow::Result<Option<AgreementDto>>;

    async fn get_agreements_by_assignee(&self, id: &String) -> anyhow::Result<Vec<AgreementDto>>;

    async fn get_agreements_by_assigner(&self, id: &String) -> anyhow::Result<Vec<AgreementDto>>;

    async fn create_agreement(&self, new_model: &NewAgreementDto) -> anyhow::Result<AgreementDto>;

    async fn put_agreement(
        &self,
        id: &Urn,
        edit_model: &EditAgreementDto,
    ) -> anyhow::Result<AgreementDto>;

    async fn delete_agreement(&self, id: &Urn) -> anyhow::Result<()>;
}
