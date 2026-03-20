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
use crate::data::entities::catalog::{EditCatalogModel, Model, NewCatalogModel};
use serde::{Deserialize, Serialize};
use urn::Urn;
use ymir::errors::Outcome;

pub(crate) mod catalogs;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CatalogDto {
    #[serde(flatten)]
    pub inner: catalog::Model,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NewCatalogDto {
    pub id: Option<Urn>,
    pub foaf_home_page: Option<String>,
    pub dct_conforms_to: Option<String>,
    pub dct_creator: Option<String>,
    pub dct_title: Option<String>,
    pub dspace_participant_id: Option<String>,
}

impl Default for NewCatalogDto {
    fn default() -> Self {
        Self {
            id: None,
            foaf_home_page: None,
            dct_conforms_to: None,
            dct_creator: None,
            dct_title: None,
            dspace_participant_id: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct EditCatalogDto {
    pub foaf_home_page: Option<String>,
    pub dct_conforms_to: Option<String>,
    pub dct_creator: Option<String>,
    pub dct_title: Option<String>,
}

impl From<NewCatalogDto> for NewCatalogModel {
    fn from(dto: NewCatalogDto) -> Self {
        Self {
            id: dto.id,
            foaf_home_page: dto.foaf_home_page,
            dct_conforms_to: dto.dct_conforms_to,
            dct_creator: dto.dct_creator,
            dct_title: dto.dct_title,
            dspace_participant_id: dto.dspace_participant_id,
        }
    }
}

impl From<EditCatalogDto> for EditCatalogModel {
    fn from(dto: EditCatalogDto) -> Self {
        Self {
            foaf_home_page: dto.foaf_home_page,
            dct_conforms_to: dto.dct_conforms_to,
            dct_creator: dto.dct_creator,
            dct_title: dto.dct_title,
        }
    }
}

impl From<catalog::Model> for CatalogDto {
    fn from(value: Model) -> Self {
        Self { inner: value }
    }
}

#[mockall::automock]
#[async_trait::async_trait]
pub trait CatalogEntityTrait: Send + Sync {
    async fn get_all_catalogs(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
        with_main_catalog: bool,
    ) -> Outcome<Vec<CatalogDto>>;
    async fn get_batch_catalogs(&self, ids: &Vec<Urn>) -> Outcome<Vec<CatalogDto>>;
    async fn get_catalog_by_id(&self, catalog_id: &Urn) -> Outcome<Option<CatalogDto>>;
    async fn get_main_catalog(&self) -> Outcome<Option<CatalogDto>>;

    async fn put_catalog_by_id(
        &self,
        catalog_id: &Urn,
        edit_catalog_model: &EditCatalogDto,
    ) -> Outcome<CatalogDto>;
    async fn create_catalog(&self, new_catalog_model: &NewCatalogDto)
        -> Outcome<CatalogDto>;

    async fn create_main_catalog(
        &self,
        new_catalog_model: &NewCatalogDto,
    ) -> Outcome<CatalogDto>;

    async fn delete_catalog_by_id(&self, catalog_id: &Urn) -> Outcome<()>;
}
