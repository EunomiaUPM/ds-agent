/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::data::entities::transfer_process;
use crate::data::entities::transfer_process::{
    EditTransferProcessModel, Model, NewTransferProcessModel,
};
use crate::data::entities::transfer_process_identifier;
use crate::data::repo_traits::transfer_process_repo::{
    TransferProcessRepoErrors, TransferProcessRepoTrait,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, JoinType,
    QueryFilter, QuerySelect, RelationTrait,
};
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

pub struct TransferProcessRepoForSql {
    db_connection: DatabaseConnection,
}

impl TransferProcessRepoForSql {
    pub fn new(db_connection: DatabaseConnection) -> Self {
        Self { db_connection }
    }
}

#[async_trait::async_trait]
impl TransferProcessRepoTrait for TransferProcessRepoForSql {
    async fn get_all_transfer_processes(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<transfer_process::Model>> {
        let processes = transfer_process::Entity::find()
            .limit(limit.unwrap_or(100))
            .offset(page.map(|p| p * limit.unwrap_or(100)).unwrap_or(0))
            .all(&self.db_connection)
            .await;
        match processes {
            Ok(processes) => Ok(processes),
            Err(e) => {
                Err(TransferProcessRepoErrors::ErrorFetchingTransferProcess(e.into()).into_errors())
            }
        }
    }

    async fn get_batch_transfer_processes(
        &self,
        ids: &Vec<Urn>,
    ) -> Outcome<Vec<transfer_process::Model>> {
        let transfer_ids = ids.iter().map(|t| t.to_string()).collect::<Vec<_>>();
        let transfer_process = transfer_process::Entity::find()
            .filter(transfer_process::Column::Id.is_in(transfer_ids))
            .all(&self.db_connection)
            .await;
        match transfer_process {
            Ok(transfer_process) => Ok(transfer_process),
            Err(e) => {
                Err(TransferProcessRepoErrors::ErrorFetchingTransferProcess(e.into()).into_errors())
            }
        }
    }

    async fn get_transfer_process_by_id(
        &self,
        id: &Urn,
    ) -> Outcome<Option<transfer_process::Model>> {
        let pid = id.to_string();
        let transfer_process =
            transfer_process::Entity::find_by_id(pid).one(&self.db_connection).await;
        match transfer_process {
            Ok(transfer_process) => Ok(transfer_process),
            Err(e) => {
                Err(TransferProcessRepoErrors::ErrorFetchingTransferProcess(e.into()).into_errors())
            }
        }
    }

    async fn get_transfer_process_by_key_id(
        &self,
        key_id: &str,
        id: &Urn,
    ) -> Outcome<Option<transfer_process::Model>> {
        let id = id.to_string();
        let transfer_process = transfer_process::Entity::find()
            .join(JoinType::InnerJoin, transfer_process::Relation::Identifiers.def())
            .filter(transfer_process_identifier::Column::IdKey.eq(key_id))
            .filter(transfer_process_identifier::Column::IdValue.eq(id))
            .one(&self.db_connection)
            .await;
        match transfer_process {
            Ok(transfer_process) => Ok(transfer_process),
            Err(e) => {
                Err(TransferProcessRepoErrors::ErrorFetchingTransferProcess(e.into()).into_errors())
            }
        }
    }

    async fn get_transfer_process_by_key_value(&self, id: &Urn) -> Outcome<Option<Model>> {
        let id = id.to_string();
        let transfer_process = transfer_process::Entity::find()
            .join(JoinType::InnerJoin, transfer_process::Relation::Identifiers.def())
            .filter(transfer_process_identifier::Column::IdValue.eq(id))
            .one(&self.db_connection)
            .await;
        match transfer_process {
            Ok(transfer_process) => Ok(transfer_process),
            Err(e) => {
                Err(TransferProcessRepoErrors::ErrorFetchingTransferProcess(e.into()).into_errors())
            }
        }
    }

    async fn create_transfer_process(
        &self,
        new_process: &NewTransferProcessModel,
    ) -> Outcome<transfer_process::Model> {
        let model: transfer_process::ActiveModel = new_process.clone().into();
        let transfer_proces =
            transfer_process::Entity::insert(model).exec_with_returning(&self.db_connection).await;
        match transfer_proces {
            Ok(transfer_process) => Ok(transfer_process),
            Err(e) => {
                return Err(
                    TransferProcessRepoErrors::ErrorCreatingTransferProcess(e.into()).into_errors()
                )
            }
        }
    }

    async fn put_transfer_process(
        &self,
        id: &Urn,
        edit_model: &EditTransferProcessModel,
    ) -> Outcome<transfer_process::Model> {
        let id = id.to_string();
        let old_model = transfer_process::Entity::find_by_id(id).one(&self.db_connection).await;
        let old_model = match old_model {
            Ok(old_model) => match old_model {
                Some(old_model) => old_model,
                None => {
                    return Err(TransferProcessRepoErrors::TransferProcessNotFound.into_errors())
                }
            },
            Err(e) => {
                return Err(
                    TransferProcessRepoErrors::ErrorFetchingTransferProcess(e.into()).into_errors()
                )
            }
        };
        let mut old_active_model: transfer_process::ActiveModel = old_model.into();
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
            Err(e) => {
                Err(TransferProcessRepoErrors::ErrorUpdatingTransferProcess(e.into()).into_errors())
            }
        }
    }

    async fn delete_transfer_process(&self, id: &Urn) -> Outcome<()> {
        let id = id.to_string();
        let transfer_process =
            transfer_process::Entity::delete_by_id(id).exec(&self.db_connection).await;
        match transfer_process {
            Ok(delete_result) => match delete_result.rows_affected {
                0 => Err(TransferProcessRepoErrors::TransferProcessNotFound.into_errors()),
                _ => Ok(()),
            },
            Err(e) => {
                Err(TransferProcessRepoErrors::ErrorDeletingTransferProcess(e.into()).into_errors())
            }
        }
    }
}
