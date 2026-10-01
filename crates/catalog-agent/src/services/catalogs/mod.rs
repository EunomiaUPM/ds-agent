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
use common::auth::AccessScope;
use common::paginated_spec::{Page, Paginated, Sort};
use urn::Urn;
use ymir::errors::Outcome;

#[mockall::automock]
#[async_trait::async_trait]
pub trait CatalogServiceTrait: Send + Sync {
    async fn get_all_catalogs(
        &self,
        scope: &AccessScope,
        filters: &CatalogFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<CatalogDto>>;
    async fn get_batch_catalogs(
        &self,
        scope: &AccessScope,
        ids: &[Urn],
    ) -> Outcome<Vec<CatalogDto>>;
    async fn get_catalog_by_id(&self, scope: &AccessScope, catalog_id: &Urn)
        -> Outcome<CatalogDto>;
    async fn get_main_catalog(&self, scope: &AccessScope) -> Outcome<Option<CatalogDto>>;

    async fn put_catalog_by_id(
        &self,
        scope: &AccessScope,
        catalog_id: &Urn,
        edit_catalog_model: &EditCatalogDto,
    ) -> Outcome<CatalogDto>;
    async fn create_catalog(
        &self,
        scope: &AccessScope,
        new_catalog_model: &NewCatalogDto,
    ) -> Outcome<CatalogDto>;

    async fn create_main_catalog(
        &self,
        scope: &AccessScope,
        new_catalog_model: &NewCatalogDto,
    ) -> Outcome<CatalogDto>;

    async fn delete_catalog_by_id(&self, scope: &AccessScope, catalog_id: &Urn) -> Outcome<()>;
}
