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
use crate::data::entities::agreement::{EditAgreementModel, Model, NewAgreementModel};
use crate::data::repo_traits::agreement_repo::{AgreementRepoErrors, AgreementRepoTrait};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QuerySelect,
};
use urn::Urn;

pub struct AgreementRepoForSql {
    db_connection: DatabaseConnection,
}

impl AgreementRepoForSql {
    pub fn new(db_connection: DatabaseConnection) -> Self {
        Self { db_connection }
    }
}

#[async_trait::async_trait]
impl AgreementRepoTrait for AgreementRepoForSql {
    async fn get_all_agreements(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> anyhow::Result<Vec<Model>, AgreementRepoErrors> {
        let agreements = agreement::Entity::find()
            .limit(limit.unwrap_or(20))
            .offset(page.map(|p| p * limit.unwrap_or(20)).unwrap_or(0))
            .all(&self.db_connection)
            .await;

        match agreements {
            Ok(agreements) => Ok(agreements),
            Err(e) => Err(AgreementRepoErrors::ErrorFetchingAgreement(e.into())),
        }
    }

    async fn get_batch_agreements(
        &self,
        ids: &Vec<Urn>,
    ) -> anyhow::Result<Vec<Model>, AgreementRepoErrors> {
        let agreement_ids = ids.iter().map(|t| t.to_string()).collect::<Vec<_>>();
        let agreements = agreement::Entity::find()
            .filter(agreement::Column::Id.is_in(agreement_ids))
            .all(&self.db_connection)
            .await;

        match agreements {
            Ok(agreements) => Ok(agreements),
            Err(e) => Err(AgreementRepoErrors::ErrorFetchingAgreement(e.into())),
        }
    }

    async fn get_agreement_by_id(
        &self,
        id: &Urn,
    ) -> anyhow::Result<Option<Model>, AgreementRepoErrors> {
        let aid = id.to_string();
        let agreement = agreement::Entity::find_by_id(aid).one(&self.db_connection).await;

        match agreement {
            Ok(agreement) => Ok(agreement),
            Err(e) => Err(AgreementRepoErrors::ErrorFetchingAgreement(e.into())),
        }
    }

    async fn get_agreement_by_negotiation_process(
        &self,
        id: &Urn,
    ) -> anyhow::Result<Option<Model>, AgreementRepoErrors> {
        let pid = id.to_string();
        let agreement = agreement::Entity::find()
            .filter(agreement::Column::NegotiationAgentProcessId.eq(pid))
            .one(&self.db_connection)
            .await;

        match agreement {
            Ok(agreement) => Ok(agreement),
            Err(e) => Err(AgreementRepoErrors::ErrorFetchingAgreement(e.into())),
        }
    }

    async fn get_agreements_by_assignee(
        &self,
        id: &String,
    ) -> anyhow::Result<Vec<Model>, AgreementRepoErrors> {
        let agreement = agreement::Entity::find()
            .filter(agreement::Column::ConsumerParticipantId.eq(id))
            .all(&self.db_connection)
            .await;

        match agreement {
            Ok(agreement) => Ok(agreement),
            Err(e) => Err(AgreementRepoErrors::ErrorFetchingAgreement(e.into())),
        }
    }

    async fn get_agreements_by_assigner(
        &self,
        id: &String,
    ) -> anyhow::Result<Vec<Model>, AgreementRepoErrors> {
        let agreement = agreement::Entity::find()
            .filter(agreement::Column::ProviderParticipantId.eq(id))
            .all(&self.db_connection)
            .await;

        match agreement {
            Ok(agreement) => Ok(agreement),
            Err(e) => Err(AgreementRepoErrors::ErrorFetchingAgreement(e.into())),
        }
    }

    async fn get_agreement_by_negotiation_message(
        &self,
        id: &Urn,
    ) -> anyhow::Result<Option<Model>, AgreementRepoErrors> {
        let mid = id.to_string();
        let agreement = agreement::Entity::find()
            .filter(agreement::Column::NegotiationAgentProcessId.eq(mid))
            .one(&self.db_connection)
            .await;

        match agreement {
            Ok(agreement) => Ok(agreement),
            Err(e) => Err(AgreementRepoErrors::ErrorFetchingAgreement(e.into())),
        }
    }

    async fn create_agreement(
        &self,
        new_model: &NewAgreementModel,
    ) -> anyhow::Result<Model, AgreementRepoErrors> {
        let model: agreement::ActiveModel = new_model.clone().into();
        let result =
            agreement::Entity::insert(model).exec_with_returning(&self.db_connection).await;

        match result {
            Ok(agreement) => Ok(agreement),
            Err(e) => Err(AgreementRepoErrors::ErrorCreatingAgreement(e.into())),
        }
    }

    async fn put_agreement(
        &self,
        id: &Urn,
        edit_model: &EditAgreementModel,
    ) -> anyhow::Result<Model, AgreementRepoErrors> {
        let aid = id.to_string();
        let old_model = agreement::Entity::find_by_id(aid).one(&self.db_connection).await;
        let old_model = match old_model {
            Ok(Some(model)) => model,
            Ok(None) => return Err(AgreementRepoErrors::AgreementNotFound),
            Err(e) => return Err(AgreementRepoErrors::ErrorFetchingAgreement(e.into())),
        };

        let mut active_model: agreement::ActiveModel = old_model.into();
        if let Some(state) = &edit_model.state {
            active_model.state = ActiveValue::Set(state.clone());
        }
        active_model.updated_at = ActiveValue::Set(Some(chrono::Utc::now().into()));

        let result = active_model.update(&self.db_connection).await;
        match result {
            Ok(updated_model) => Ok(updated_model),
            Err(e) => Err(AgreementRepoErrors::ErrorUpdatingAgreement(e.into())),
        }
    }

    async fn delete_agreement(&self, id: &Urn) -> anyhow::Result<(), AgreementRepoErrors> {
        let aid = id.to_string();
        let result = agreement::Entity::delete_by_id(aid).exec(&self.db_connection).await;

        match result {
            Ok(delete_result) => match delete_result.rows_affected {
                0 => Err(AgreementRepoErrors::AgreementNotFound),
                _ => Ok(()),
            },
            Err(e) => Err(AgreementRepoErrors::ErrorDeletingAgreement(e.into())),
        }
    }
}
