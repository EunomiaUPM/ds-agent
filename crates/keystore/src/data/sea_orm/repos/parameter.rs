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

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use ymir::errors::{Outcome, RepoIntoErrors};

use crate::data::repo::parameters::{ParameterRepoErrors, ParameterRepoTrait};
use crate::data::sea_orm::orm::parameter;
use crate::entities::commands::{EditParameterCommand, NewParameterCommand};
use crate::entities::entry::Entry;
use crate::entities::key::Key;

pub struct SeaOrmParameterRepo {
    db: DatabaseConnection,
}

impl SeaOrmParameterRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl ParameterRepoTrait for SeaOrmParameterRepo {
    type Value = serde_json::Value;

    async fn get_all_parameters(&self) -> Outcome<Vec<Entry<Self::Value>>> {
        let rows = parameter::Entity::find()
            .filter(parameter::Column::DeletedAt.is_null())
            .all(&self.db)
            .await
            .map_err(|e| ParameterRepoErrors::ErrorFetchingParameter(e.into()).into_errors())?;

        rows.into_iter()
            .map(|m| {
                m.into_entry().map_err(|e| {
                    ParameterRepoErrors::ErrorFetchingParameter(e.into()).into_errors()
                })
            })
            .collect()
    }

    async fn count_parameters(&self) -> Outcome<u64> {
        use sea_orm::PaginatorTrait;
        parameter::Entity::find()
            .filter(parameter::Column::DeletedAt.is_null())
            .count(&self.db)
            .await
            .map_err(|e| ParameterRepoErrors::ErrorFetchingParameter(e.into()).into_errors())
    }

    async fn get_batch_parameters(&self, keys: &[Key]) -> Outcome<Vec<Entry<Self::Value>>> {
        let key_strs: Vec<&str> = keys.iter().map(|k| k.as_str()).collect();
        let rows = parameter::Entity::find()
            .filter(parameter::Column::Key.is_in(key_strs))
            .filter(parameter::Column::DeletedAt.is_null())
            .all(&self.db)
            .await
            .map_err(|e| ParameterRepoErrors::ErrorFetchingParameter(e.into()).into_errors())?;

        rows.into_iter()
            .map(|m| {
                m.into_entry().map_err(|e| {
                    ParameterRepoErrors::ErrorFetchingParameter(e.into()).into_errors()
                })
            })
            .collect()
    }

    async fn get_parameter_by_key(&self, key: &Key) -> Outcome<Option<Entry<Self::Value>>> {
        let row = parameter::Entity::find_by_id(key.as_str())
            .filter(parameter::Column::DeletedAt.is_null())
            .one(&self.db)
            .await
            .map_err(|e| ParameterRepoErrors::ErrorFetchingParameter(e.into()).into_errors())?;

        row.map(|m| {
            m.into_entry()
                .map_err(|e| ParameterRepoErrors::ErrorFetchingParameter(e.into()).into_errors())
        })
        .transpose()
    }

    async fn create_parameter(
        &self,
        cmd: &NewParameterCommand<Self::Value>,
    ) -> Outcome<Entry<Self::Value>> {
        // Reject if an active (non-deleted) entry already exists.
        let exists = parameter::Entity::find_by_id(cmd.key.as_str())
            .filter(parameter::Column::DeletedAt.is_null())
            .one(&self.db)
            .await
            .map_err(|e| ParameterRepoErrors::ErrorCreatingParameter(e.into()).into_errors())?
            .is_some();

        if exists {
            return Err(ParameterRepoErrors::ParameterAlreadyExists.into_errors());
        }

        let active = parameter::ActiveModel::from_new_cmd(cmd);
        let model = parameter::Entity::insert(active)
            .exec_with_returning(&self.db)
            .await
            .map_err(|e| ParameterRepoErrors::ErrorCreatingParameter(e.into()).into_errors())?;

        model
            .into_entry()
            .map_err(|e| ParameterRepoErrors::ErrorCreatingParameter(e.into()).into_errors())
    }

    async fn put_parameter(
        &self,
        key: &Key,
        cmd: &EditParameterCommand<Self::Value>,
    ) -> Outcome<Entry<Self::Value>> {
        let current = parameter::Entity::find_by_id(key.as_str())
            .filter(parameter::Column::DeletedAt.is_null())
            .one(&self.db)
            .await
            .map_err(|e| ParameterRepoErrors::ErrorFetchingParameter(e.into()).into_errors())?
            .ok_or_else(|| ParameterRepoErrors::ParameterNotFound.into_errors())?;

        let current_version = current.version;
        let expected = cmd.expected_version.value() as i64;

        if current_version != expected {
            use crate::entities::version::Version;
            return Err(ParameterRepoErrors::VersionConflict {
                expected: cmd.expected_version,
                actual: Version::new(current_version as u64),
            }
            .into_errors());
        }

        let updated = parameter::ActiveModel::from(current)
            .apply_edit_cmd(cmd, expected + 1)
            .update(&self.db)
            .await
            .map_err(|e| ParameterRepoErrors::ErrorUpdatingParameter(e.into()).into_errors())?;

        updated
            .into_entry()
            .map_err(|e| ParameterRepoErrors::ErrorUpdatingParameter(e.into()).into_errors())
    }

    async fn delete_parameter(&self, key: &Key) -> Outcome<()> {
        let current = parameter::Entity::find_by_id(key.as_str())
            .filter(parameter::Column::DeletedAt.is_null())
            .one(&self.db)
            .await
            .map_err(|e| ParameterRepoErrors::ErrorDeletingParameter(e.into()).into_errors())?
            .ok_or_else(|| ParameterRepoErrors::ParameterNotFound.into_errors())?;

        let mut active: parameter::ActiveModel = current.into();
        active.deleted_at = ActiveValue::Set(Some(Utc::now().into()));

        active
            .update(&self.db)
            .await
            .map(|_| ())
            .map_err(|e| ParameterRepoErrors::ErrorDeletingParameter(e.into()).into_errors())
    }
}
