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
use crate::data::entities::agreement::{EditAgreementModel, Model, NewAgreementModel};
use crate::data::repo_traits::agreement_repo::{AgreementRepoErrors, AgreementRepoTrait};
use crate::entities::filters::AgreementFilter;
use common::paginated_spec::{Page, SelectCursorExt, Sort};
use common::query::FilterApplier;
use sea_orm::QueryTrait;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait,
    QueryFilter, Select,
};
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

impl FilterApplier<Select<agreement::Entity>> for AgreementFilter {
    fn apply_to(&self, mut q: Select<agreement::Entity>) -> Select<agreement::Entity> {
        if let Some(ref id) = self.id {
            q = q.filter(agreement::Column::Id.eq(id));
        }
        if let Some(ref tenant_id) = self.tenant_id {
            q = q.filter(agreement::Column::TenantId.eq(tenant_id));
        }
        if let Some(ref process_id) = self.process_id {
            q = q.filter(agreement::Column::NegotiationAgentProcessId.eq(process_id));
        }
        if let Some(ref consumer_id) = self.consumer_id {
            q = q.filter(agreement::Column::ConsumerParticipantId.eq(consumer_id));
        }
        if let Some(ref provider_id) = self.provider_id {
            q = q.filter(agreement::Column::ProviderParticipantId.eq(provider_id));
        }
        if let Some(ref target) = self.target {
            q = q.filter(agreement::Column::Target.eq(target));
        }
        if let Some(ref state) = self.state {
            q = q.filter(
                agreement::Column::State
                    .eq(state.as_str())
                    .or(agreement::Column::State.eq(state.to_uppercase()))
                    .or(agreement::Column::State.eq(state.to_lowercase())),
            );
        }
        if let Some(after) = self.created_after {
            q = q.filter(agreement::Column::CreatedAt.gte(after));
        }
        if let Some(before) = self.created_before {
            q = q.filter(agreement::Column::CreatedAt.lte(before));
        }
        q
    }
}

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
        filters: &AgreementFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<(Vec<Model>, Option<u64>)> {
        let mut q = agreement::Entity::find();
        q = filters.apply_to(q);

        let total = q
            .clone()
            .count(&self.db_connection)
            .await
            .map_err(|e| AgreementRepoErrors::ErrorFetchingAgreement(e.into()).into_errors())?;

        let items = q
            .apply_cursor_pagination_with_tie_break(
                page,
                sort,
                agreement::Column::CreatedAt,
                agreement::Column::Id,
            )
            .all(&self.db_connection)
            .await
            .map_err(|e| AgreementRepoErrors::ErrorFetchingAgreement(e.into()).into_errors())?;

        Ok((items, Some(total)))
    }

    async fn get_batch_agreements(
        &self,
        tenant_id: Option<String>,
        ids: &[Urn],
    ) -> Outcome<Vec<Model>> {
        let agreement_ids = ids.iter().map(|t| t.to_string()).collect::<Vec<_>>();
        let agreements = agreement::Entity::find()
            .apply_if(tenant_id, |q, t| {
                q.filter(agreement::Column::TenantId.eq(t))
            })
            .filter(agreement::Column::Id.is_in(agreement_ids))
            .all(&self.db_connection)
            .await;

        match agreements {
            Ok(agreements) => Ok(agreements),
            Err(e) => Err(AgreementRepoErrors::ErrorFetchingAgreement(e.into()).into_errors()),
        }
    }

    async fn get_agreement_by_id(
        &self,
        tenant_id: Option<String>,
        id: &Urn,
    ) -> Outcome<Option<Model>> {
        let aid = id.to_string();
        let agreement = agreement::Entity::find_by_id(aid)
            .apply_if(tenant_id, |q, t| {
                q.filter(agreement::Column::TenantId.eq(t))
            })
            .one(&self.db_connection)
            .await;

        match agreement {
            Ok(agreement) => Ok(agreement),
            Err(e) => Err(AgreementRepoErrors::ErrorFetchingAgreement(e.into()).into_errors()),
        }
    }

    async fn get_agreement_by_negotiation_process(
        &self,
        tenant_id: Option<String>,
        id: &Urn,
    ) -> Outcome<Option<Model>> {
        let pid = id.to_string();
        let agreement = agreement::Entity::find()
            .apply_if(tenant_id, |q, t| {
                q.filter(agreement::Column::TenantId.eq(t))
            })
            .filter(agreement::Column::NegotiationAgentProcessId.eq(pid))
            .one(&self.db_connection)
            .await;

        match agreement {
            Ok(agreement) => Ok(agreement),
            Err(e) => Err(AgreementRepoErrors::ErrorFetchingAgreement(e.into()).into_errors()),
        }
    }

    async fn get_agreements_by_assignee(
        &self,
        tenant_id: Option<String>,
        id: &str,
    ) -> Outcome<Vec<Model>> {
        let agreement = agreement::Entity::find()
            .apply_if(tenant_id, |q, t| {
                q.filter(agreement::Column::TenantId.eq(t))
            })
            .filter(agreement::Column::ConsumerParticipantId.eq(id))
            .all(&self.db_connection)
            .await;

        match agreement {
            Ok(agreement) => Ok(agreement),
            Err(e) => Err(AgreementRepoErrors::ErrorFetchingAgreement(e.into()).into_errors()),
        }
    }

    async fn get_agreements_by_assigner(
        &self,
        tenant_id: Option<String>,
        id: &str,
    ) -> Outcome<Vec<Model>> {
        let agreement = agreement::Entity::find()
            .apply_if(tenant_id, |q, t| {
                q.filter(agreement::Column::TenantId.eq(t))
            })
            .filter(agreement::Column::ProviderParticipantId.eq(id))
            .all(&self.db_connection)
            .await;

        match agreement {
            Ok(agreement) => Ok(agreement),
            Err(e) => Err(AgreementRepoErrors::ErrorFetchingAgreement(e.into()).into_errors()),
        }
    }

    async fn get_agreement_by_negotiation_message(
        &self,
        tenant_id: Option<String>,
        id: &Urn,
    ) -> Outcome<Option<Model>> {
        let mid = id.to_string();
        let agreement = agreement::Entity::find()
            .apply_if(tenant_id, |q, t| {
                q.filter(agreement::Column::TenantId.eq(t))
            })
            .filter(agreement::Column::NegotiationAgentMessageId.eq(mid))
            .one(&self.db_connection)
            .await;

        match agreement {
            Ok(agreement) => Ok(agreement),
            Err(e) => Err(AgreementRepoErrors::ErrorFetchingAgreement(e.into()).into_errors()),
        }
    }

    async fn create_agreement(&self, new_model: &NewAgreementModel) -> Outcome<Model> {
        let model: agreement::ActiveModel = new_model.clone().into();
        let result = agreement::Entity::insert(model)
            .exec_with_returning(&self.db_connection)
            .await;

        match result {
            Ok(agreement) => Ok(agreement),
            Err(e) => Err(AgreementRepoErrors::ErrorCreatingAgreement(e.into()).into_errors()),
        }
    }

    async fn put_agreement(
        &self,
        tenant_id: Option<String>,
        id: &Urn,
        edit_model: &EditAgreementModel,
    ) -> Outcome<Model> {
        let aid = id.to_string();
        let old_model = agreement::Entity::find_by_id(&aid)
            .apply_if(tenant_id, |q, t| {
                q.filter(agreement::Column::TenantId.eq(t))
            })
            .one(&self.db_connection)
            .await;
        let old_model = match old_model {
            Ok(Some(model)) => model,
            Ok(None) => return Err(AgreementRepoErrors::AgreementNotFound.into_errors()),
            Err(e) => {
                return Err(AgreementRepoErrors::ErrorFetchingAgreement(e.into()).into_errors());
            }
        };

        let mut active_model: agreement::ActiveModel = old_model.into();
        if let Some(state) = &edit_model.state {
            active_model.state = ActiveValue::Set(state.clone());
        }
        active_model.updated_at = ActiveValue::Set(Some(chrono::Utc::now().into()));

        let result = active_model.update(&self.db_connection).await;
        match result {
            Ok(updated_model) => Ok(updated_model),
            Err(e) => Err(AgreementRepoErrors::ErrorUpdatingAgreement(e.into()).into_errors()),
        }
    }

    async fn delete_agreement(&self, tenant_id: Option<String>, id: &Urn) -> Outcome<()> {
        let aid = id.to_string();
        let result = agreement::Entity::delete_many()
            .filter(agreement::Column::Id.eq(&aid))
            .apply_if(tenant_id, |q, t| {
                q.filter(agreement::Column::TenantId.eq(t))
            })
            .exec(&self.db_connection)
            .await;

        match result {
            Ok(delete_result) => match delete_result.rows_affected {
                0 => Err(AgreementRepoErrors::AgreementNotFound.into_errors()),
                _ => Ok(()),
            },
            Err(e) => Err(AgreementRepoErrors::ErrorDeletingAgreement(e.into()).into_errors()),
        }
    }
}
