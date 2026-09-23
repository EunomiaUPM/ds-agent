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

use crate::data::entities::catalog;
use crate::data::entities::catalog::{EditCatalogModel, Model, NewCatalogModel};
use serde::{Deserialize, Serialize};
use urn::Urn;

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
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
            tenant_id: None,
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

impl NewCatalogDto {
    pub fn into_model(self, tenant_id: String) -> NewCatalogModel {
        NewCatalogModel {
            id: self.id,
            tenant_id,
            foaf_home_page: self.foaf_home_page,
            dct_conforms_to: self.dct_conforms_to,
            dct_creator: self.dct_creator,
            dct_title: self.dct_title,
            dspace_participant_id: self.dspace_participant_id,
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
