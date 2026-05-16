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
use uuid::Uuid;
use ymir::errors::{Outcome, RepoIntoErrors};

use crate::entities::refresh_token::RefreshToken;

#[mockall::automock]
#[async_trait::async_trait]
pub(crate) trait RefreshTokenRepository: Send + Sync {
    async fn create(&self, token: &RefreshToken) -> Outcome<RefreshToken>;
    async fn get_by_jti(&self, jti: &str) -> Outcome<Option<RefreshToken>>;
    async fn revoke(&self, id: Uuid) -> Outcome<()>;
    async fn revoke_all_for_tenant(&self, tenant_id: &str) -> Outcome<()>;
}

#[derive(Debug, Error)]
pub(crate) enum RefreshTokenRepositoryError {
    #[error("token not found")]
    NotFound,
    #[error("database error: {0}")]
    Db(Box<dyn std::error::Error + Send + Sync>),
}

impl RepoIntoErrors for RefreshTokenRepositoryError {}
