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

use crate::data::entities::agreement::{EditAgreementModel, NewAgreementModel};
use crate::data::factory_trait::NegotiationAgentRepoTrait;
use crate::data::repo_traits::agreement_repo::AgreementRepoErrors;
use crate::entities::agreement::{
    AgreementDto, EditAgreementDto, NegotiationAgentAgreementsTrait, NewAgreementDto,
};
use rainbow_common::errors::{CommonErrors, ErrorLog};
use std::sync::Arc;
use tracing::error;
use urn::Urn;

pub struct NegotiationAgentAgreementsService {
    pub negotiation_repo: Arc<dyn NegotiationAgentRepoTrait>,
}

impl NegotiationAgentAgreementsService {
    pub fn new(negotiation_repo: Arc<dyn NegotiationAgentRepoTrait>) -> Self {
        Self { negotiation_repo }
    }
}

#[async_trait::async_trait]
impl NegotiationAgentAgreementsTrait for NegotiationAgentAgreementsService {
    async fn get_all_agreements(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> anyhow::Result<Vec<AgreementDto>> {
        let agreements = self
            .negotiation_repo
            .get_agreement_repo()
            .get_all_agreements(limit, page)
            .await
            .map_err(|e| {
                let err = CommonErrors::database_new(&e.to_string());
                error!("{}", err.log());
                err
            })?;

        Ok(agreements.into_iter().map(|m| AgreementDto { inner: m }).collect())
    }

    async fn get_batch_agreements(&self, ids: &Vec<Urn>) -> anyhow::Result<Vec<AgreementDto>> {
        let agreements =
            self.negotiation_repo.get_agreement_repo().get_batch_agreements(ids).await.map_err(
                |e| {
                    let err = CommonErrors::database_new(&e.to_string());
                    error!("{}", err.log());
                    err
                },
            )?;

        Ok(agreements.into_iter().map(|m| AgreementDto { inner: m }).collect())
    }

    async fn get_agreement_by_id(&self, id: &Urn) -> anyhow::Result<Option<AgreementDto>> {
        let agreement =
            self.negotiation_repo.get_agreement_repo().get_agreement_by_id(id).await.map_err(
                |e| {
                    let err = CommonErrors::database_new(&e.to_string());
                    error!("{}", err.log());
                    err
                },
            )?;

        Ok(agreement.map(|m| AgreementDto { inner: m }))
    }

    async fn get_agreement_by_negotiation_process(
        &self,
        id: &Urn,
    ) -> anyhow::Result<Option<AgreementDto>> {
        let agreement = self
            .negotiation_repo
            .get_agreement_repo()
            .get_agreement_by_negotiation_process(id)
            .await
            .map_err(|e| {
                let err = CommonErrors::database_new(&e.to_string());
                error!("{}", err.log());
                err
            })?;

        Ok(agreement.map(|m| AgreementDto { inner: m }))
    }

    async fn get_agreement_by_negotiation_message(
        &self,
        id: &Urn,
    ) -> anyhow::Result<Option<AgreementDto>> {
        let agreement = self
            .negotiation_repo
            .get_agreement_repo()
            .get_agreement_by_negotiation_message(id)
            .await
            .map_err(|e| {
                let err = CommonErrors::database_new(&e.to_string());
                error!("{}", err.log());
                err
            })?;

        Ok(agreement.map(|m| AgreementDto { inner: m }))
    }

    async fn get_agreements_by_assignee(&self, id: &String) -> anyhow::Result<Vec<AgreementDto>> {
        let agreements = self
            .negotiation_repo
            .get_agreement_repo()
            .get_agreements_by_assignee(id)
            .await
            .map_err(|e| {
                let err = CommonErrors::database_new(&e.to_string());
                error!("{}", err.log());
                err
            })?;

        Ok(agreements.into_iter().map(|m| AgreementDto { inner: m }).collect())
    }

    async fn get_agreements_by_assigner(&self, id: &String) -> anyhow::Result<Vec<AgreementDto>> {
        let agreements = self
            .negotiation_repo
            .get_agreement_repo()
            .get_agreements_by_assigner(id)
            .await
            .map_err(|e| {
                let err = CommonErrors::database_new(&e.to_string());
                error!("{}", err.log());
                err
            })?;

        Ok(agreements.into_iter().map(|m| AgreementDto { inner: m }).collect())
    }

    async fn create_agreement(
        &self,
        new_model_dto: &NewAgreementDto,
    ) -> anyhow::Result<AgreementDto> {
        let new_model: NewAgreementModel = new_model_dto.clone().into();

        let created =
            self.negotiation_repo.get_agreement_repo().create_agreement(&new_model).await.map_err(
                |e| {
                    let err = CommonErrors::database_new(&e.to_string());
                    error!("{}", err.log());
                    err
                },
            )?;

        Ok(AgreementDto { inner: created })
    }

    async fn put_agreement(
        &self,
        id: &Urn,
        edit_model_dto: &EditAgreementDto,
    ) -> anyhow::Result<AgreementDto> {
        let edit_model: EditAgreementModel = edit_model_dto.clone().into();

        let updated = self
            .negotiation_repo
            .get_agreement_repo()
            .put_agreement(id, &edit_model)
            .await
            .map_err(|e| match e {
                AgreementRepoErrors::AgreementNotFound => {
                    let err = CommonErrors::missing_resource_new(
                        &id.to_string(),
                        "Agreement not found for update",
                    );
                    error!("{}", err.log());
                    err
                }
                _ => {
                    let err = CommonErrors::database_new(&e.to_string());
                    error!("{}", err.log());
                    err
                }
            })?;

        Ok(AgreementDto { inner: updated })
    }

    async fn delete_agreement(&self, id: &Urn) -> anyhow::Result<()> {
        self.negotiation_repo.get_agreement_repo().delete_agreement(id).await.map_err(
            |e| match e {
                AgreementRepoErrors::AgreementNotFound => {
                    let err = CommonErrors::missing_resource_new(
                        &id.to_string(),
                        "Agreement not found for deletion",
                    );
                    error!("{}", err.log());
                    err
                }
                _ => {
                    let err = CommonErrors::database_new(&e.to_string());
                    error!("{}", err.log());
                    err
                }
            },
        )?;
        Ok(())
    }
}
