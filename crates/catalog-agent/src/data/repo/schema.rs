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

//! Persistence of hosted schemas, one row per version; rows are never updated.

use thiserror::Error;
use ymir::errors::{Outcome, RepoIntoErrors};
use ymir::oauth::OwnerScope;
use ymir::paginated_spec::{Page, Sort};

use crate::entities::commands::NewSchemaVersionCommand;
use crate::entities::filters::SchemaFilter;
use crate::entities::schemas::DataSchema;

/// Schema versions within the owner scope each call gives.
#[mockall::automock]
#[async_trait::async_trait]
pub trait SchemaRepoTrait: Send + Sync {
    async fn get_page(
        &self,
        scope: &OwnerScope,
        filter: &SchemaFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Vec<DataSchema>>;
    async fn count(&self, scope: &OwnerScope, filter: &SchemaFilter) -> Outcome<u64>;
    /// Every version of the schema, newest first.
    async fn get_versions(&self, scope: &OwnerScope, id: &str) -> Outcome<Vec<DataSchema>>;
    async fn get_version(
        &self,
        scope: &OwnerScope,
        id: &str,
        version: i32,
    ) -> Outcome<Option<DataSchema>>;
    async fn insert(&self, command: &NewSchemaVersionCommand) -> Outcome<DataSchema>;
    /// Deletes one version, or every version when `version` is `None`.
    async fn delete(&self, scope: &OwnerScope, id: &str, version: Option<i32>) -> Outcome<()>;
}

#[derive(Debug, Error)]
pub enum SchemaRepoErrors {
    #[error("Schema not found")]
    NotFound,
    #[error("Error fetching schemas. {0}")]
    Fetching(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error creating schema. {0}")]
    Creating(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error deleting schema. {0}")]
    Deleting(Box<dyn std::error::Error + Send + Sync>),
}

impl RepoIntoErrors for SchemaRepoErrors {}
