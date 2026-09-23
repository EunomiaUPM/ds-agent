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

use crate::data::entities::dataservice;
use crate::data::entities::dataservice::{EditDataServiceModel, Model, NewDataServiceModel};
use serde::{Deserialize, Serialize};
use urn::Urn;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataServiceDto {
    #[serde(flatten)]
    pub inner: dataservice::Model,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NewDataServiceDto {
    pub id: Option<Urn>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    pub dcat_endpoint_description: Option<String>,
    pub dcat_endpoint_url: String,
    pub dct_conforms_to: Option<String>,
    pub dct_creator: Option<String>,
    pub dct_title: Option<String>,
    pub dct_description: Option<String>,
    pub catalog_id: Urn,
}

impl Default for NewDataServiceDto {
    fn default() -> Self {
        Self {
            id: None,
            tenant_id: None,
            dcat_endpoint_description: None,
            dcat_endpoint_url: "".to_string(),
            dct_conforms_to: None,
            dct_creator: None,
            dct_title: None,
            dct_description: None,
            catalog_id: std::str::FromStr::from_str("urn:fake-urn:000").unwrap(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct EditDataServiceDto {
    pub dcat_endpoint_description: Option<String>,
    pub dcat_endpoint_url: Option<String>,
    pub dct_conforms_to: Option<String>,
    pub dct_creator: Option<String>,
    pub dct_title: Option<String>,
    pub dct_description: Option<String>,
}

impl NewDataServiceDto {
    pub fn into_model(self, tenant_id: String) -> NewDataServiceModel {
        NewDataServiceModel {
            id: self.id,
            tenant_id,
            dcat_endpoint_description: self.dcat_endpoint_description,
            dcat_endpoint_url: self.dcat_endpoint_url,
            dct_conforms_to: self.dct_conforms_to,
            dct_creator: self.dct_creator,
            dct_title: self.dct_title,
            dct_description: self.dct_description,
            catalog_id: self.catalog_id,
            dspace_main_data_service: false,
        }
    }
}

impl From<EditDataServiceDto> for EditDataServiceModel {
    fn from(dto: EditDataServiceDto) -> Self {
        Self {
            dcat_endpoint_description: dto.dcat_endpoint_description,
            dcat_endpoint_url: dto.dcat_endpoint_url,
            dct_conforms_to: dto.dct_conforms_to,
            dct_creator: dto.dct_creator,
            dct_title: dto.dct_title,
            dct_description: dto.dct_description,
        }
    }
}

impl From<dataservice::Model> for DataServiceDto {
    fn from(value: Model) -> Self {
        Self { inner: value }
    }
}
