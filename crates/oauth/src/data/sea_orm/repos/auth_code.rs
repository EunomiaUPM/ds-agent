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

use std::sync::Arc;

use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait};
use ymir::errors::{Outcome, RepoIntoErrors};

use crate::data::repositories::auth_code::{AuthCodeRepository, AuthCodeRepositoryError};
use crate::data::sea_orm::orm::auth_code as orm;
use crate::entities::auth_code::AuthCode;

pub(crate) struct SeaOrmAuthCodeRepository {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmAuthCodeRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl AuthCodeRepository for SeaOrmAuthCodeRepository {
    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn save(&self, auth_code: &AuthCode) -> Outcome<AuthCode> {
        orm::ActiveModel::from_domain(auth_code)
            .insert(self.db.as_ref())
            .await
            .map_err(|e| AuthCodeRepositoryError::Db(Box::new(e)).into_errors())
            .and_then(orm::Model::into_domain)
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_by_code(&self, code: &str) -> Outcome<Option<AuthCode>> {
        orm::Entity::find_by_id(code)
            .one(self.db.as_ref())
            .await
            .map_err(|e| AuthCodeRepositoryError::Db(Box::new(e)).into_errors())?
            .map(orm::Model::into_domain)
            .transpose()
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn mark_used(&self, code: &str) -> Outcome<()> {
        let existing = orm::Entity::find_by_id(code)
            .one(self.db.as_ref())
            .await
            .map_err(|e| AuthCodeRepositoryError::Db(Box::new(e)).into_errors())?
            .ok_or_else(|| AuthCodeRepositoryError::NotFound.into_errors())?;

        let mut active: orm::ActiveModel = existing.into();
        active.used = Set(true);
        active
            .update(self.db.as_ref())
            .await
            .map_err(|e| AuthCodeRepositoryError::Db(Box::new(e)).into_errors())?;
        Ok(())
    }
}
