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

//! Catalog repository.

use crate::data::entities::catalog;
use common::oauth::OwnerScope;
use crate::data::entities::catalog::{EditCatalogModel, NewCatalogModel};
use crate::entities::filters::CatalogFilter;
use common::paginated_spec::{Page, Sort};
use urn::Urn;
use ymir::errors::Outcome;

/// Persistence of catalogs, within the owner scope each call gives.
#[mockall::automock]
#[async_trait::async_trait]
pub trait CatalogRepositoryTrait: Send + Sync {
    /// Page of catalogs matching the filters, with the total.
    async fn get_all_catalogs(
        &self,
        scope: &OwnerScope,
        filters: &CatalogFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<(Vec<catalog::Model>, Option<u64>)>;
    async fn get_batch_catalogs(
        &self,
        scope: &OwnerScope,
        ids: &[Urn],
    ) -> Outcome<Vec<catalog::Model>>;
    async fn get_catalog_by_id(
        &self,
        scope: &OwnerScope,
        catalog_id: &Urn,
    ) -> Outcome<Option<catalog::Model>>;
    /// The connector's main catalog, the one served over DSP.
    async fn get_main_catalog(&self) -> Outcome<Option<catalog::Model>>;

    async fn put_catalog_by_id(
        &self,
        scope: &OwnerScope,
        catalog_id: &Urn,
        edit_catalog_model: &EditCatalogModel,
    ) -> Outcome<catalog::Model>;
    async fn create_catalog(&self, new_catalog_model: &NewCatalogModel) -> Outcome<catalog::Model>;

    /// Stores the catalog as the connector's main one, unless there is one already.
    async fn create_main_catalog(
        &self,
        new_catalog_model: &NewCatalogModel,
    ) -> Outcome<catalog::Model>;

    /// Deletes and returns the removed row.
    async fn delete_catalog_by_id(
        &self,
        scope: &OwnerScope,
        catalog_id: &Urn,
    ) -> Outcome<catalog::Model>;
}
