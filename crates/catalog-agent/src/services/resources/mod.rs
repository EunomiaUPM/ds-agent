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

//! Management of catalog resources of every kind, described as RDF.

pub mod service;

use serde_json::Value;
use urn::Urn;
use ymir::errors::Outcome;
use ymir::paginated_spec::{Page, Paginated, Sort};
use ymir::types::oauth::UserInfo;
use ymir::types::participants::Visibility;

use crate::entities::commands::{NewResourceCommand, ReplaceResourceCommand};
use crate::entities::filters::ResourceFilter;
use crate::entities::resource::CatalogResource;
use crate::entities::resource_kind::ResourceKind;

/// Management of resources; a lookup for the wrong kind is a missing resource.
#[mockall::automock]
#[async_trait::async_trait]
pub trait ResourceServiceTrait: Send + Sync {
    /// Page of the resources the caller sees, of the filter's kind.
    async fn list(
        &self,
        user: &UserInfo,
        filter: &ResourceFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<CatalogResource>>;
    /// The resources of `kind` found among `ids`.
    async fn batch(
        &self,
        user: &UserInfo,
        kind: ResourceKind,
        ids: &[Urn],
    ) -> Outcome<Vec<CatalogResource>>;
    /// 404 when it is not visible to the caller or not of `kind`.
    async fn get(&self, user: &UserInfo, kind: ResourceKind, id: &Urn) -> Outcome<CatalogResource>;
    /// The connector's main resource of `kind`, if any (whoever asks).
    async fn main(&self, kind: ResourceKind) -> Outcome<Option<CatalogResource>>;
    /// The resources of `kind` hanging from `parent`.
    async fn children(
        &self,
        user: &UserInfo,
        kind: ResourceKind,
        parent: &Urn,
    ) -> Outcome<Vec<CatalogResource>>;
    /// The distributions of any of the datasets, in one query.
    async fn distributions_of(
        &self,
        user: &UserInfo,
        datasets: &[Urn],
    ) -> Outcome<Vec<CatalogResource>>;
    /// The dataset's distribution in `format` (`dct:format`).
    async fn distribution_by_format(
        &self,
        user: &UserInfo,
        dataset: &Urn,
        format: &str,
    ) -> Outcome<CatalogResource>;
    async fn create(
        &self,
        user: &UserInfo,
        command: &NewResourceCommand,
    ) -> Outcome<CatalogResource>;
    /// Creates the connector's main resource of the command's kind (the root only), unless there
    /// is one already.
    async fn create_main(
        &self,
        user: &UserInfo,
        command: &NewResourceCommand,
    ) -> Outcome<CatalogResource>;
    async fn replace(
        &self,
        user: &UserInfo,
        kind: ResourceKind,
        id: &Urn,
        command: &ReplaceResourceCommand,
    ) -> Outcome<CatalogResource>;
    /// Creates the resource a management API document describes; a document with no `@type` is
    /// taken as one of `kind`, and a main data service with no catalog goes to the main one.
    async fn create_document(
        &self,
        user: &UserInfo,
        kind: ResourceKind,
        document: Value,
        visibility: Option<Visibility>,
        main: bool,
    ) -> Outcome<CatalogResource>;
    /// Replaces the description of `id` with the one a document gives.
    async fn replace_document(
        &self,
        user: &UserInfo,
        kind: ResourceKind,
        id: &Urn,
        document: Value,
        visibility: Option<Visibility>,
        expected: Option<i64>,
    ) -> Outcome<CatalogResource>;
    /// Publishes it (`draft` false) or keeps it out of every profile (`draft` true); datasets
    /// and distributions only. Emits `publish` or `draft`, for crawlers to be told later.
    async fn set_publication(
        &self,
        user: &UserInfo,
        kind: ResourceKind,
        id: &Urn,
        draft: bool,
        expected: Option<i64>,
    ) -> Outcome<CatalogResource>;
    /// Removes it and what hangs from it; `expected` is the version the caller read, if any.
    async fn delete(
        &self,
        user: &UserInfo,
        kind: ResourceKind,
        id: &Urn,
        expected: Option<i64>,
    ) -> Outcome<()>;
}
