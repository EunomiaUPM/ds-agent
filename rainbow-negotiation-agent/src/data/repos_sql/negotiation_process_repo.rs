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

use crate::data::entities::negotiation_process;
use crate::data::entities::negotiation_process::{
    EditNegotiationProcessModel, Model, NewNegotiationProcessModel,
};
use crate::data::entities::negotiation_process_identifier;
use crate::data::repo_traits::negotiation_process_repo::{
    NegotiationProcessRepoErrors, NegotiationProcessRepoTrait,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, JoinType,
    QueryFilter, QuerySelect, RelationTrait,
};
use urn::Urn;

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
    async fn get_all_negotiation_processes(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> anyhow::Result<Vec<Model>, NegotiationProcessRepoErrors> {
        let processes = negotiation_process::Entity::find()
            .limit(limit.unwrap_or(100))
            .offset(page.map(|p| p * limit.unwrap_or(100)).unwrap_or(0))
            .all(&self.db_connection)
            .await;
        match processes {
            Ok(processes) => Ok(processes),
            Err(e) => Err(NegotiationProcessRepoErrors::ErrorFetchingNegotiationProcess(
                e.into(),
            )),
        }
    }

    async fn get_batch_negotiation_processes(
        &self,
        ids: &Vec<Urn>,
    ) -> anyhow::Result<Vec<Model>, NegotiationProcessRepoErrors> {
        let negotiation_ids = ids.iter().map(|t| t.to_string()).collect::<Vec<_>>();
        let negotiation_process = negotiation_process::Entity::find()
            .filter(negotiation_process::Column::Id.is_in(negotiation_ids))
            .all(&self.db_connection)
            .await;
        match negotiation_process {
            Ok(negotiation_process) => Ok(negotiation_process),
            Err(e) => Err(NegotiationProcessRepoErrors::ErrorFetchingNegotiationProcess(
                e.into(),
            )),
        }
    }

    async fn get_negotiation_process_by_id(
        &self,
        id: &Urn,
    ) -> anyhow::Result<Option<Model>, NegotiationProcessRepoErrors> {
        let pid = id.to_string();
        let negotiation_process =
            negotiation_process::Entity::find_by_id(pid).one(&self.db_connection).await;
        match negotiation_process {
            Ok(negotiation_process) => Ok(negotiation_process),
            Err(e) => Err(NegotiationProcessRepoErrors::ErrorFetchingNegotiationProcess(
                e.into(),
            )),
        }
    }

    async fn get_negotiation_process_by_key_id(
        &self,
        key_id: &str,
        id: &Urn,
    ) -> anyhow::Result<Option<Model>, NegotiationProcessRepoErrors> {
        let id = id.to_string();
        let negotiation_process = negotiation_process::Entity::find()
            .join(JoinType::InnerJoin, negotiation_process::Relation::Identifiers.def())
            .filter(negotiation_process_identifier::Column::IdKey.eq(key_id))
            .filter(negotiation_process_identifier::Column::IdValue.eq(id))
            .one(&self.db_connection)
            .await;
        match negotiation_process {
            Ok(negotiation_process) => Ok(negotiation_process),
            Err(e) => Err(NegotiationProcessRepoErrors::ErrorFetchingNegotiationProcess(
                e.into(),
            )),
        }
    }

    async fn get_negotiation_process_by_key_value(
        &self,
        id: &Urn,
    ) -> anyhow::Result<Option<Model>, NegotiationProcessRepoErrors> {
        let id = id.to_string();
        let negotiation_process = negotiation_process::Entity::find()
            .join(JoinType::InnerJoin, negotiation_process::Relation::Identifiers.def())
            .filter(negotiation_process_identifier::Column::IdValue.eq(id))
            .one(&self.db_connection)
            .await;
        match negotiation_process {
            Ok(negotiation_process) => Ok(negotiation_process),
            Err(e) => Err(NegotiationProcessRepoErrors::ErrorFetchingNegotiationProcess(
                e.into(),
            )),
        }
    }

    async fn create_negotiation_process(
        &self,
        new_model: &NewNegotiationProcessModel,
    ) -> anyhow::Result<Model, NegotiationProcessRepoErrors> {
        let model: negotiation_process::ActiveModel = new_model.clone().into();
        let negotiation_process = negotiation_process::Entity::insert(model)
            .exec_with_returning(&self.db_connection)
            .await;
        match negotiation_process {
            Ok(negotiation_process) => Ok(negotiation_process),
            Err(e) => Err(NegotiationProcessRepoErrors::ErrorCreatingNegotiationProcess(
                e.into(),
            )),
        }
    }

    async fn put_negotiation_process(
        &self,
        id: &Urn,
        edit_model: &EditNegotiationProcessModel,
    ) -> anyhow::Result<Model, NegotiationProcessRepoErrors> {
        let id = id.to_string();
        let old_model = negotiation_process::Entity::find_by_id(id).one(&self.db_connection).await;
        let old_model = match old_model {
            Ok(old_model) => match old_model {
                Some(old_model) => old_model,
                None => return Err(NegotiationProcessRepoErrors::NegotiationProcessNotFound),
            },
            Err(e) => {
                return Err(NegotiationProcessRepoErrors::ErrorFetchingNegotiationProcess(
                    e.into(),
                ));
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
            Err(e) => Err(NegotiationProcessRepoErrors::ErrorUpdatingNegotiationProcess(
                e.into(),
            )),
        }
    }

    async fn delete_negotiation_process(
        &self,
        id: &Urn,
    ) -> anyhow::Result<(), NegotiationProcessRepoErrors> {
        let id = id.to_string();
        let negotiation_process =
            negotiation_process::Entity::delete_by_id(id).exec(&self.db_connection).await;
        match negotiation_process {
            Ok(delete_result) => match delete_result.rows_affected {
                0 => Err(NegotiationProcessRepoErrors::NegotiationProcessNotFound),
                _ => Ok(()),
            },
            Err(e) => Err(NegotiationProcessRepoErrors::ErrorDeletingNegotiationProcess(
                e.into(),
            )),
        }
    }
}
