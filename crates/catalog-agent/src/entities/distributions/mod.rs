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

use crate::data::entities::distribution;
use crate::data::entities::distribution::{EditDistributionModel, Model, NewDistributionModel};
use serde::{Deserialize, Serialize};
use urn::Urn;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DistributionDto {
    #[serde(flatten)]
    pub inner: distribution::Model,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NewDistributionDto {
    pub id: Option<Urn>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    pub dct_title: Option<String>,
    pub dct_description: Option<String>,
    pub dct_formats: Option<String>,
    pub dcat_access_service: String,
    pub dataset_id: Urn,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct EditDistributionDto {
    pub dct_title: Option<String>,
    pub dct_description: Option<String>,
    pub dcat_access_service: Option<String>,
}

impl NewDistributionDto {
    pub fn into_model(self, tenant_id: String) -> NewDistributionModel {
        NewDistributionModel {
            id: self.id,
            tenant_id,
            dct_title: self.dct_title,
            dct_description: self.dct_description,
            dct_formats: self.dct_formats,
            dcat_access_service: self.dcat_access_service,
            dataset_id: self.dataset_id,
        }
    }
}

impl From<EditDistributionDto> for EditDistributionModel {
    fn from(dto: EditDistributionDto) -> Self {
        Self {
            dct_title: dto.dct_title,
            dct_description: dto.dct_description,
            dcat_access_service: dto.dcat_access_service,
        }
    }
}

impl From<distribution::Model> for DistributionDto {
    fn from(value: Model) -> Self {
        Self { inner: value }
    }
}
