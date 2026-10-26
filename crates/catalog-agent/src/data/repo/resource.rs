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

//! Persistence of catalog resources and the links between them.

use common::oauth::OwnerScope;
use common::paginated_spec::{Page, Sort};
use thiserror::Error;
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

use crate::entities::filters::ResourceFilter;
use crate::entities::resource::CatalogResource;
use crate::entities::resource_kind::ResourceKind;

/// Resources within the owner scope each call gives; the links of a resource are derived from
/// its description on every write.
#[mockall::automock]
#[async_trait::async_trait]
pub trait ResourceRepoTrait: Send + Sync {
    /// One page of the resources matching the filter.
    async fn get_page(
        &self,
        scope: &OwnerScope,
        filter: &ResourceFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Vec<CatalogResource>>;
    async fn count(&self, scope: &OwnerScope, filter: &ResourceFilter) -> Outcome<u64>;
    /// Every resource matching the filter, unpaged: for the closures a projection needs whole.
    async fn get_all(&self, scope: &OwnerScope, filter: &ResourceFilter) -> Outcome<Vec<CatalogResource>>;
    async fn get_batch(&self, scope: &OwnerScope, ids: &[Urn]) -> Outcome<Vec<CatalogResource>>;
    async fn get_by_id(&self, scope: &OwnerScope, id: &Urn) -> Outcome<Option<CatalogResource>>;
    /// The connector's main resource of `kind`, whoever asks.
    async fn get_main(&self, kind: ResourceKind) -> Outcome<Option<CatalogResource>>;
    /// The resources hanging from any of `parents` (catalogs or datasets), in one query.
    async fn get_children(
        &self,
        scope: &OwnerScope,
        kind: Option<ResourceKind>,
        parents: &[Urn],
    ) -> Outcome<Vec<CatalogResource>>;
    /// The dataset's distribution whose `dct:format` is `format`.
    async fn get_distribution_by_format(
        &self,
        scope: &OwnerScope,
        dataset_id: &Urn,
        format: &str,
    ) -> Outcome<Option<CatalogResource>>;
    async fn insert(&self, resource: &CatalogResource) -> Outcome<CatalogResource>;
    /// Stores `resource` over the stored version `expected`, with its links, atomically.
    async fn replace(
        &self,
        scope: &OwnerScope,
        resource: &CatalogResource,
        expected: i64,
    ) -> Outcome<CatalogResource>;
    /// Removes the resource (and what hangs from it) and returns it.
    async fn delete(&self, scope: &OwnerScope, id: &Urn) -> Outcome<CatalogResource>;
    /// The `(subject, predicate)` pairs linking to `id`.
    async fn referrers(&self, id: &Urn) -> Outcome<Vec<(String, String)>>;
}

#[derive(Debug, Error)]
pub enum ResourceRepoErrors {
    #[error("Resource not found")]
    NotFound,
    #[error("Invalid pagination cursor")]
    InvalidCursor,
    #[error("Error fetching resources. {0}")]
    Fetching(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error creating resource. {0}")]
    Creating(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error updating resource. {0}")]
    Updating(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error deleting resource. {0}")]
    Deleting(Box<dyn std::error::Error + Send + Sync>),
    #[error("Stored description is not valid. {0}")]
    Decoding(String),
}

impl RepoIntoErrors for ResourceRepoErrors {}
