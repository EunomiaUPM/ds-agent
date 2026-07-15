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

use thiserror::Error;
use ymir::errors::{Outcome, RepoIntoErrors};

use crate::entities::query::{Page, Sort, UserFilter};
use crate::entities::role::RbacRole;
use crate::entities::user::User;

#[mockall::automock]
#[async_trait::async_trait]
pub(crate) trait UserRepository: Send + Sync {
    async fn get_all(&self, filter: &UserFilter, page: &Page, sort: &Sort) -> Outcome<Vec<User>>;
    async fn get_by_tenant_id(&self, tenant_id: &str) -> Outcome<Option<User>>;
    async fn get_by_email(&self, email: &str) -> Outcome<Option<User>>;
    async fn create(&self, user: &User) -> Outcome<User>;
    async fn patch(
        &self,
        tenant_id: &str,
        email: Option<String>,
        role: Option<RbacRole>,
        extra_fields: Option<serde_json::Value>,
    ) -> Outcome<User>;
    async fn delete(&self, tenant_id: &str) -> Outcome<()>;
}

#[derive(Debug, Error)]
pub(crate) enum UserRepositoryError {
    #[error("user not found")]
    NotFound,
    #[error("user already exists")]
    AlreadyExists,
    #[error("database error: {0}")]
    Db(Box<dyn std::error::Error + Send + Sync>),
}

impl RepoIntoErrors for UserRepositoryError {}
