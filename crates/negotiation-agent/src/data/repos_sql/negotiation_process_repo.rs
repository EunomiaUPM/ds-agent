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

use crate::data::entities::negotiation_process;
use crate::data::entities::negotiation_process::{
    EditNegotiationProcessModel, Model, NewNegotiationProcessModel,
};
use crate::data::entities::negotiation_process_identifier;
use crate::data::repo_traits::negotiation_process_repo::{
    NegotiationProcessRepoErrors, NegotiationProcessRepoTrait,
};
use crate::entities::filters::NegotiationProcessFilter;
use common::paginated_spec::{Page, SelectCursorExt, Sort};
use common::query::FilterApplier;
use sea_orm::QueryTrait;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, JoinType,
    PaginatorTrait, QueryFilter, QuerySelect, RelationTrait, Select,
};
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

impl FilterApplier<Select<negotiation_process::Entity>> for NegotiationProcessFilter {
    fn apply_to(
        &self,
        mut q: Select<negotiation_process::Entity>,
    ) -> Select<negotiation_process::Entity> {
        if let Some(ref id) = self.id {
            q = q.filter(negotiation_process::Column::Id.eq(id));
        }
        if let Some(ref tenant_id) = self.tenant_id {
            q = q.filter(negotiation_process::Column::TenantId.eq(tenant_id));
        }
        if let Some(ref state) = self.state {
            let normalized = state.replace("dspace:", "");
            let dspace_state = format!("dspace:{normalized}");
            q = q.filter(
                negotiation_process::Column::State
                    .eq(state.as_str())
                    .or(negotiation_process::Column::State.eq(normalized.as_str()))
                    .or(negotiation_process::Column::State.eq(dspace_state.as_str())),
            );
        }
        if let Some(ref role) = self.role {
            q = q.filter(
                negotiation_process::Column::Role
                    .eq(role.as_str())
                    .or(negotiation_process::Column::Role.eq(role.to_lowercase())),
            );
        }
        if let Some(ref protocol) = self.protocol {
            q = q.filter(negotiation_process::Column::Protocol.eq(protocol));
        }
        if let Some(ref associated_agent_peer) = self.associated_agent_peer {
            q = q
                .filter(negotiation_process::Column::AssociatedAgentPeer.eq(associated_agent_peer));
        }
        if let Some(after) = self.created_after {
            q = q.filter(negotiation_process::Column::CreatedAt.gte(after));
        }
        if let Some(before) = self.created_before {
            q = q.filter(negotiation_process::Column::CreatedAt.lte(before));
        }
        q
    }
}

pub struct NegotiationProcessRepoForSql {
    db_connection: DatabaseConnection,
}

impl NegotiationProcessRepoForSql {
    pub fn new(db_connection: DatabaseConnection) -> Self {
        Self { db_connection }
    }
}

#[async_trait::async_trait]
impl NegotiationProcessRepoTrait for NegotiationProcessRepoForSql {
    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_all_negotiation_processes(
        &self,
        filters: &NegotiationProcessFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<(Vec<Model>, Option<u64>)> {
        let mut q = negotiation_process::Entity::find();
        q = filters.apply_to(q);

        let total = q.clone().count(&self.db_connection).await.map_err(|e| {
            NegotiationProcessRepoErrors::ErrorFetchingNegotiationProcess(e.into()).into_errors()
        })?;

        let items = q
            .apply_cursor_pagination_with_tie_break(
                page,
                sort,
                negotiation_process::Column::CreatedAt,
                negotiation_process::Column::Id,
            )
            .all(&self.db_connection)
            .await
            .map_err(|e| {
                NegotiationProcessRepoErrors::ErrorFetchingNegotiationProcess(e.into())
                    .into_errors()
            })?;

        Ok((items, Some(total)))
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_batch_negotiation_processes(
        &self,
        tenant_id: Option<String>,
        ids: &[Urn],
    ) -> Outcome<Vec<Model>> {
        let negotiation_ids = ids.iter().map(|t| t.to_string()).collect::<Vec<_>>();
        let negotiation_process = negotiation_process::Entity::find()
            .apply_if(tenant_id, |q, t| {
                q.filter(negotiation_process::Column::TenantId.eq(t))
            })
            .filter(negotiation_process::Column::Id.is_in(negotiation_ids))
            .all(&self.db_connection)
            .await;
        match negotiation_process {
            Ok(negotiation_process) => Ok(negotiation_process),
            Err(e) => Err(
                NegotiationProcessRepoErrors::ErrorFetchingNegotiationProcess(e.into())
                    .into_errors(),
            ),
        }
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_negotiation_process_by_id(
        &self,
        tenant_id: Option<String>,
        id: &Urn,
    ) -> Outcome<Option<Model>> {
        let pid = id.to_string();
        let negotiation_process = negotiation_process::Entity::find_by_id(pid)
            .apply_if(tenant_id, |q, t| {
                q.filter(negotiation_process::Column::TenantId.eq(t))
            })
            .one(&self.db_connection)
            .await;
        match negotiation_process {
            Ok(negotiation_process) => Ok(negotiation_process),
            Err(e) => Err(
                NegotiationProcessRepoErrors::ErrorFetchingNegotiationProcess(e.into())
                    .into_errors(),
            ),
        }
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_negotiation_process_by_key_id(
        &self,
        tenant_id: Option<String>,
        key_id: &str,
        id: &Urn,
    ) -> Outcome<Option<Model>> {
        let id_str = id.to_string();
        let mut q = negotiation_process::Entity::find()
            .join(
                JoinType::InnerJoin,
                negotiation_process::Relation::Identifiers.def(),
            )
            .filter(negotiation_process_identifier::Column::IdKey.eq(key_id))
            .filter(negotiation_process_identifier::Column::IdValue.eq(id_str));
        if let Some(ref t) = tenant_id {
            q = q.filter(negotiation_process::Column::TenantId.eq(t));
        }
        let negotiation_process = q.one(&self.db_connection).await;
        match negotiation_process {
            Ok(negotiation_process) => Ok(negotiation_process),
            Err(e) => Err(
                NegotiationProcessRepoErrors::ErrorFetchingNegotiationProcess(e.into())
                    .into_errors(),
            ),
        }
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_negotiation_process_by_key_value(
        &self,
        tenant_id: Option<String>,
        id: &Urn,
    ) -> Outcome<Option<Model>> {
        let id_str = id.to_string();
        let mut q = negotiation_process::Entity::find()
            .join(
                JoinType::LeftJoin,
                negotiation_process::Relation::Identifiers.def(),
            )
            .filter(
                negotiation_process::Column::Id
                    .eq(&id_str)
                    .or(negotiation_process_identifier::Column::IdValue.eq(&id_str)),
            );
        if let Some(ref t) = tenant_id {
            q = q.filter(negotiation_process::Column::TenantId.eq(t));
        }
        let negotiation_process = q.one(&self.db_connection).await;
        match negotiation_process {
            Ok(negotiation_process) => Ok(negotiation_process),
            Err(e) => Err(
                NegotiationProcessRepoErrors::ErrorFetchingNegotiationProcess(e.into())
                    .into_errors(),
            ),
        }
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn create_negotiation_process(
        &self,
        new_model: &NewNegotiationProcessModel,
    ) -> Outcome<Model> {
        let model: negotiation_process::ActiveModel = new_model.clone().into();
        let negotiation_process = negotiation_process::Entity::insert(model)
            .exec_with_returning(&self.db_connection)
            .await;
        match negotiation_process {
            Ok(negotiation_process) => Ok(negotiation_process),
            Err(e) => Err(
                NegotiationProcessRepoErrors::ErrorCreatingNegotiationProcess(e.into())
                    .into_errors(),
            ),
        }
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn put_negotiation_process(
        &self,
        tenant_id: Option<String>,
        id: &Urn,
        edit_model: &EditNegotiationProcessModel,
    ) -> Outcome<Model> {
        let id_str = id.to_string();
        let old_model = negotiation_process::Entity::find_by_id(&id_str)
            .apply_if(tenant_id, |q, t| {
                q.filter(negotiation_process::Column::TenantId.eq(t))
            })
            .one(&self.db_connection)
            .await;
        let old_model = match old_model {
            Ok(Some(old_model)) => old_model,
            Ok(None) => {
                return Err(NegotiationProcessRepoErrors::NegotiationProcessNotFound.into_errors());
            }
            Err(e) => {
                return Err(
                    NegotiationProcessRepoErrors::ErrorFetchingNegotiationProcess(e.into())
                        .into_errors(),
                );
            }
        };
        let mut old_active_model: negotiation_process::ActiveModel = old_model.into();
        if let Some(state) = &edit_model.state {
            old_active_model.state = ActiveValue::Set(state.clone());
        }
        if let Some(state_attribute) = &edit_model.state_attribute {
            old_active_model.state_attribute = ActiveValue::Set(Some(state_attribute.clone()));
        }
        if let Some(properties) = &edit_model.properties {
            old_active_model.properties = ActiveValue::Set(properties.clone());
        }
        if let Some(error_details) = &edit_model.error_details {
            old_active_model.error_details = ActiveValue::Set(Some(error_details.clone()));
        }
        old_active_model.updated_at = ActiveValue::Set(Some(chrono::Utc::now().into()));
        let model = old_active_model.update(&self.db_connection).await;
        match model {
            Ok(model) => Ok(model),
            Err(e) => Err(
                NegotiationProcessRepoErrors::ErrorUpdatingNegotiationProcess(e.into())
                    .into_errors(),
            ),
        }
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn delete_negotiation_process(
        &self,
        tenant_id: Option<String>,
        id: &Urn,
    ) -> Outcome<String> {
        let id_str = id.to_string();
        let delete_result = negotiation_process::Entity::delete_many()
            .filter(negotiation_process::Column::Id.eq(&id_str))
            .apply_if(tenant_id, |q, t| {
                q.filter(negotiation_process::Column::TenantId.eq(t))
            })
            .exec_with_returning(&self.db_connection)
            .await;
        match delete_result {
            Ok(rows) => rows
                .into_iter()
                .next()
                .map(|row| row.tenant_id)
                .ok_or_else(|| {
                    NegotiationProcessRepoErrors::NegotiationProcessNotFound.into_errors()
                }),
            Err(e) => Err(
                NegotiationProcessRepoErrors::ErrorDeletingNegotiationProcess(e.into())
                    .into_errors(),
            ),
        }
    }
}
