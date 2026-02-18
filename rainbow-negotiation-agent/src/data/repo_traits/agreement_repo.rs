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

use crate::data::entities::agreement;
use crate::data::entities::agreement::{EditAgreementModel, NewAgreementModel};
use anyhow::Error;
use thiserror::Error;
use urn::Urn;

#[async_trait::async_trait]
pub trait AgreementRepoTrait: Send + Sync {
    async fn get_all_agreements(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> anyhow::Result<Vec<agreement::Model>, AgreementRepoErrors>;
    async fn get_batch_agreements(
        &self,
        ids: &Vec<Urn>,
    ) -> anyhow::Result<Vec<agreement::Model>, AgreementRepoErrors>;
    async fn get_agreement_by_id(
        &self,
        id: &Urn,
    ) -> anyhow::Result<Option<agreement::Model>, AgreementRepoErrors>;
    async fn get_agreement_by_negotiation_process(
        &self,
        id: &Urn,
    ) -> anyhow::Result<Option<agreement::Model>, AgreementRepoErrors>;
    async fn get_agreements_by_assignee(
        &self,
        id: &String,
    ) -> anyhow::Result<Vec<agreement::Model>, AgreementRepoErrors>;

    async fn get_agreements_by_assigner(
        &self,
        id: &String,
    ) -> anyhow::Result<Vec<agreement::Model>, AgreementRepoErrors>;

    async fn get_agreement_by_negotiation_message(
        &self,
        id: &Urn,
    ) -> anyhow::Result<Option<agreement::Model>, AgreementRepoErrors>;
    async fn create_agreement(
        &self,
        new_model: &NewAgreementModel,
    ) -> anyhow::Result<agreement::Model, AgreementRepoErrors>;
    async fn put_agreement(
        &self,
        id: &Urn,
        edit_model: &EditAgreementModel,
    ) -> anyhow::Result<agreement::Model, AgreementRepoErrors>;
    async fn delete_agreement(&self, id: &Urn) -> anyhow::Result<(), AgreementRepoErrors>;
}

#[derive(Debug, Error)]
pub enum AgreementRepoErrors {
    #[error("Agreement not found")]
    AgreementNotFound,
    #[error("Error fetching agreement. {0}")]
    ErrorFetchingAgreement(Error),
    #[error("Error creating agreement. {0}")]
    ErrorCreatingAgreement(Error),
    #[error("Error updating agreement. {0}")]
    ErrorUpdatingAgreement(Error),
    #[error("Error deleting agreement. {0}")]
    ErrorDeletingAgreement(Error),
}
