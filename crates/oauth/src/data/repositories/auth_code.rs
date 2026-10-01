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

use crate::entities::auth_code::AuthCode;

#[mockall::automock]
#[async_trait::async_trait]
pub(crate) trait AuthCodeRepository: Send + Sync {
    async fn save(&self, auth_code: &AuthCode) -> Outcome<AuthCode>;
    async fn get_by_code(&self, code: &str) -> Outcome<Option<AuthCode>>;
    async fn mark_used(&self, code: &str) -> Outcome<()>;
}

#[derive(Debug, Error)]
pub(crate) enum AuthCodeRepositoryError {
    #[error("code not found")]
    NotFound,
    #[error("database error: {0}")]
    Db(Box<dyn std::error::Error + Send + Sync>),
}

impl RepoIntoErrors for AuthCodeRepositoryError {}
