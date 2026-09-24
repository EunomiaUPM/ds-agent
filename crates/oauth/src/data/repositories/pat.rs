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

use crate::entities::pat::PersonalAccessToken;
use crate::entities::query::{Page, PatFilter, Sort};

#[mockall::automock]
#[async_trait::async_trait]
pub trait PatRepository: Send + Sync {
    async fn get_all(
        &self,
        filter: &PatFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Vec<PersonalAccessToken>>;
    async fn count(&self, filter: &PatFilter) -> Outcome<u64>;
    async fn create(&self, pat: &PersonalAccessToken) -> Outcome<PersonalAccessToken>;
    async fn get_by_id(&self, tenant_id: &str, id: Uuid) -> Outcome<Option<PersonalAccessToken>>;
    async fn get_batch(&self, tenant_id: &str, ids: &[Uuid]) -> Outcome<Vec<PersonalAccessToken>>;
    async fn get_by_hash(&self, token_hash: &str) -> Outcome<Option<PersonalAccessToken>>;
    async fn list_by_tenant(&self, tenant_id: &str) -> Outcome<Vec<PersonalAccessToken>>;
    /// Returns the tenant of the revoked token.
    async fn revoke(&self, tenant_id: Option<String>, id: Uuid) -> Outcome<String>;
    async fn update_last_used(&self, id: Uuid) -> Outcome<()>;
}

#[derive(Debug, Error)]
pub enum PatRepositoryError {
    #[error("personal access token not found")]
    NotFound,
    #[error("database error: {0}")]
    Db(Box<dyn std::error::Error + Send + Sync>),
}

impl RepoIntoErrors for PatRepositoryError {}
