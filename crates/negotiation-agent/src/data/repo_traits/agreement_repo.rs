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

use crate::data::entities::agreement;
use crate::data::entities::agreement::{EditAgreementModel, NewAgreementModel};
use crate::entities::filters::AgreementFilter;
use common::paginated_spec::{Page, Sort};
use thiserror::Error;
use urn::Urn;
use ymir::errors::Outcome;
use ymir::errors::RepoIntoErrors;

#[mockall::automock]
#[async_trait::async_trait]
pub trait AgreementRepoTrait: Send + Sync {
    async fn get_all_agreements(
        &self,
        filters: &AgreementFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<(Vec<agreement::Model>, Option<u64>)>;
    async fn get_batch_agreements(
        &self,
        tenant_id: &str,
        ids: &[Urn],
    ) -> Outcome<Vec<agreement::Model>>;
    async fn get_agreement_by_id(
        &self,
        tenant_id: &str,
        id: &Urn,
    ) -> Outcome<Option<agreement::Model>>;
    async fn get_agreement_by_negotiation_process(
        &self,
        tenant_id: &str,
        id: &Urn,
    ) -> Outcome<Option<agreement::Model>>;
    async fn get_agreements_by_assignee(
        &self,
        tenant_id: &str,
        id: &str,
    ) -> Outcome<Vec<agreement::Model>>;

    async fn get_agreements_by_assigner(
        &self,
        tenant_id: &str,
        id: &str,
    ) -> Outcome<Vec<agreement::Model>>;

    async fn get_agreement_by_negotiation_message(
        &self,
        tenant_id: &str,
        id: &Urn,
    ) -> Outcome<Option<agreement::Model>>;
    async fn create_agreement(&self, new_model: &NewAgreementModel) -> Outcome<agreement::Model>;
    async fn put_agreement(
        &self,
        tenant_id: &str,
        id: &Urn,
        edit_model: &EditAgreementModel,
    ) -> Outcome<agreement::Model>;
    async fn delete_agreement(&self, tenant_id: &str, id: &Urn) -> Outcome<()>;
}

#[derive(Debug, Error)]
pub enum AgreementRepoErrors {
    #[error("Agreement not found")]
    AgreementNotFound,
    #[error("Error fetching agreement. {0}")]
    ErrorFetchingAgreement(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error creating agreement. {0}")]
    ErrorCreatingAgreement(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error updating agreement. {0}")]
    ErrorUpdatingAgreement(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error deleting agreement. {0}")]
    ErrorDeletingAgreement(Box<dyn std::error::Error + Send + Sync>),
}

impl RepoIntoErrors for AgreementRepoErrors {}
