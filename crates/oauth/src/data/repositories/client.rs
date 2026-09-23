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

use crate::entities::client::Client;
use crate::entities::query::{ClientFilter, Page, Sort};

#[mockall::automock]
#[async_trait::async_trait]
pub trait ClientRepository: Send + Sync {
    async fn get_all(
        &self,
        filter: &ClientFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Vec<Client>>;
    async fn count(&self, filter: &ClientFilter) -> Outcome<u64>;
    async fn get_by_id(
        &self,
        tenant_id: Option<String>,
        client_id: &str,
    ) -> Outcome<Option<Client>>;
    async fn get_batch(&self, tenant_id: &str, client_ids: &[String]) -> Outcome<Vec<Client>>;
    async fn get_by_client_id(&self, client_id: &str) -> Outcome<Option<Client>>;
    async fn create(&self, client: &Client) -> Outcome<Client>;
    async fn delete(&self, tenant_id: Option<String>, client_id: &str) -> Outcome<()>;
}

#[derive(Debug, Error)]
pub enum ClientRepositoryError {
    #[error("client not found")]
    NotFound,
    #[error("client already exists")]
    AlreadyExists,
    #[error("database error: {0}")]
    Db(Box<dyn std::error::Error + Send + Sync>),
}

impl RepoIntoErrors for ClientRepositoryError {}
