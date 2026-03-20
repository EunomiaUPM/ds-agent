/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use crate::data::entities::catalog;
use crate::data::entities::catalog::{EditCatalogModel, NewCatalogModel};
use crate::data::repo_traits::catalog_db_errors::CatalogAgentRepoErrors;
use urn::Urn;
use ymir::errors::Outcome;

#[async_trait::async_trait]
pub trait CatalogRepositoryTrait: Send + Sync {
    async fn get_all_catalogs(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
        with_main_catalog: bool,
    ) -> Outcome<Vec<catalog::Model>>;
    async fn get_batch_catalogs(
        &self,
        ids: &Vec<Urn>,
    ) -> Outcome<Vec<catalog::Model>>;
    async fn get_catalog_by_id(
        &self,
        catalog_id: &Urn,
    ) -> Outcome<Option<catalog::Model>>;
    async fn get_main_catalog(
        &self,
    ) -> Outcome<Option<catalog::Model>>;

    async fn put_catalog_by_id(
        &self,
        catalog_id: &Urn,
        edit_catalog_model: &EditCatalogModel,
    ) -> Outcome<catalog::Model>;
    async fn create_catalog(
        &self,
        new_catalog_model: &NewCatalogModel,
    ) -> Outcome<catalog::Model>;

    async fn create_main_catalog(
        &self,
        new_catalog_model: &NewCatalogModel,
    ) -> Outcome<catalog::Model>;

    async fn delete_catalog_by_id(
        &self,
        catalog_id: &Urn,
    ) -> Outcome<()>;
}
