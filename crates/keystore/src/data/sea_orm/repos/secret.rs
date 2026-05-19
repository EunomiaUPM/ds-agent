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

use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use ymir::errors::{Outcome, RepoIntoErrors};

use crate::data::repo::secrets::{SecretRepoErrors, SecretRepoTrait};
use crate::data::sea_orm::orm::secret;
use crate::entities::commands::{EditSecretCommand, NewSecretCommand};
use crate::entities::entry::SecretEntry;
use crate::entities::key::Key;

pub struct SeaOrmSecretRepo {
    db: DatabaseConnection,
}

impl SeaOrmSecretRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl SecretRepoTrait for SeaOrmSecretRepo {
    async fn get_all_secrets(&self) -> Outcome<Vec<SecretEntry>> {
        let rows = secret::Entity::find()
            .filter(secret::Column::DeletedAt.is_null())
            .all(&self.db)
            .await
            .map_err(|e| SecretRepoErrors::ErrorFetchingSecret(e.into()).into_errors())?;

        rows.into_iter()
            .map(|m| {
                m.into_entry()
                    .map_err(|e| SecretRepoErrors::ErrorFetchingSecret(e.into()).into_errors())
            })
            .collect()
    }

    async fn count_secrets(&self) -> Outcome<u64> {
        use sea_orm::PaginatorTrait;
        secret::Entity::find()
            .filter(secret::Column::DeletedAt.is_null())
            .count(&self.db)
            .await
            .map_err(|e| SecretRepoErrors::ErrorFetchingSecret(e.into()).into_errors())
    }

    async fn get_batch_secrets(&self, keys: &[Key]) -> Outcome<Vec<SecretEntry>> {
        let key_strs: Vec<&str> = keys.iter().map(|k| k.as_str()).collect();
        let rows = secret::Entity::find()
            .filter(secret::Column::Key.is_in(key_strs))
            .filter(secret::Column::DeletedAt.is_null())
            .all(&self.db)
            .await
            .map_err(|e| SecretRepoErrors::ErrorFetchingSecret(e.into()).into_errors())?;

        rows.into_iter()
            .map(|m| {
                m.into_entry()
                    .map_err(|e| SecretRepoErrors::ErrorFetchingSecret(e.into()).into_errors())
            })
            .collect()
    }

    async fn get_secret_by_key(&self, key: &Key) -> Outcome<Option<SecretEntry>> {
        let row = secret::Entity::find_by_id(key.as_str())
            .filter(secret::Column::DeletedAt.is_null())
            .one(&self.db)
            .await
            .map_err(|e| SecretRepoErrors::ErrorFetchingSecret(e.into()).into_errors())?;

        row.map(|m| {
            m.into_entry()
                .map_err(|e| SecretRepoErrors::ErrorFetchingSecret(e.into()).into_errors())
        })
        .transpose()
    }

    async fn create_secret(&self, cmd: &NewSecretCommand) -> Outcome<SecretEntry> {
        let exists = secret::Entity::find_by_id(cmd.key.as_str())
            .filter(secret::Column::DeletedAt.is_null())
            .one(&self.db)
            .await
            .map_err(|e| SecretRepoErrors::ErrorCreatingSecret(e.into()).into_errors())?
            .is_some();

        if exists {
            return Err(SecretRepoErrors::SecretAlreadyExists.into_errors());
        }

        let active = secret::ActiveModel::from_new_cmd(cmd);
        let model = secret::Entity::insert(active)
            .exec_with_returning(&self.db)
            .await
            .map_err(|e| SecretRepoErrors::ErrorCreatingSecret(e.into()).into_errors())?;

        model
            .into_entry()
            .map_err(|e| SecretRepoErrors::ErrorCreatingSecret(e.into()).into_errors())
    }

    async fn put_secret(&self, key: &Key, cmd: &EditSecretCommand) -> Outcome<SecretEntry> {
        let current = secret::Entity::find_by_id(key.as_str())
            .filter(secret::Column::DeletedAt.is_null())
            .one(&self.db)
            .await
            .map_err(|e| SecretRepoErrors::ErrorFetchingSecret(e.into()).into_errors())?
            .ok_or_else(|| SecretRepoErrors::SecretNotFound.into_errors())?;

        let current_version = current.version;
        let expected = cmd.expected_version.value() as i64;

        if current_version != expected {
            use crate::entities::version::Version;
            return Err(SecretRepoErrors::VersionConflict {
                expected: cmd.expected_version,
                actual: Version::new(current_version as u64),
            }
            .into_errors());
        }

        let updated = secret::ActiveModel::from(current)
            .apply_edit_cmd(cmd, expected + 1)
            .update(&self.db)
            .await
            .map_err(|e| SecretRepoErrors::ErrorUpdatingSecret(e.into()).into_errors())?;

        updated
            .into_entry()
            .map_err(|e| SecretRepoErrors::ErrorUpdatingSecret(e.into()).into_errors())
    }

    async fn delete_secret(&self, key: &Key) -> Outcome<()> {
        let result = secret::Entity::delete_by_id(key.as_str())
            .exec(&self.db)
            .await
            .map_err(|e| SecretRepoErrors::ErrorDeletingSecret(e.into()).into_errors())?;

        if result.rows_affected == 0 {
            return Err(SecretRepoErrors::SecretNotFound.into_errors());
        }
        Ok(())
    }
}
