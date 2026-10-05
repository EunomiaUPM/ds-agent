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

//! Catalog management use cases.

pub mod service;

use crate::entities::catalogs::{CatalogDto, EditCatalogDto, NewCatalogDto};
use crate::entities::filters::CatalogFilter;
use common::oauth::UserInfo;
use common::paginated_spec::{Page, Paginated, Sort};
use urn::Urn;
use ymir::errors::Outcome;

/// Management of catalogs.
#[mockall::automock]
#[async_trait::async_trait]
pub trait CatalogServiceTrait: Send + Sync {
    /// Page of catalogs visible to the caller.
    async fn get_all_catalogs(
        &self,
        user: &UserInfo,
        filters: &CatalogFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<CatalogDto>>;
    /// Catalogs found among `ids`.
    async fn get_batch_catalogs(
        &self,
        user: &UserInfo,
        ids: &[Urn],
    ) -> Outcome<Vec<CatalogDto>>;
    /// 404 when the catalog is not visible to the caller.
    async fn get_catalog_by_id(&self, user: &UserInfo, catalog_id: &Urn)
        -> Outcome<CatalogDto>;
    /// The connector's main catalog, if it has one (whoever asks).
    async fn get_main_catalog(&self, user: &UserInfo) -> Outcome<Option<CatalogDto>>;

    async fn put_catalog_by_id(
        &self,
        user: &UserInfo,
        catalog_id: &Urn,
        edit_catalog_model: &EditCatalogDto,
    ) -> Outcome<CatalogDto>;
    async fn create_catalog(
        &self,
        user: &UserInfo,
        new_catalog_model: &NewCatalogDto,
    ) -> Outcome<CatalogDto>;

    /// Creates the connector's main catalog (the root only), unless there is one already.
    async fn create_main_catalog(
        &self,
        user: &UserInfo,
        new_catalog_model: &NewCatalogDto,
    ) -> Outcome<CatalogDto>;

    async fn delete_catalog_by_id(&self, user: &UserInfo, catalog_id: &Urn) -> Outcome<()>;
}
