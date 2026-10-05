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

//! Datasets.

use crate::data::entities::dataset;
use common::oauth::{Owner, Visibility};
use crate::data::entities::dataset::{EditDatasetModel, Model, NewDatasetModel};
use serde::{Deserialize, Serialize};
use urn::Urn;

/// Dataset as returned by the API.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DatasetDto {
    #[serde(flatten)]
    pub inner: dataset::Model,
}

/// New dataset.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NewDatasetDto {
    pub id: Option<Urn>,
    /// Who else sees it; private by default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<Visibility>,
    /// Owner an in-process flow asks for; only honoured for the root.
    #[serde(skip)]
    pub owner: Option<Owner>,
    pub dct_conforms_to: Option<String>,
    pub dct_creator: Option<String>,
    pub dct_title: Option<String>,
    pub dct_description: Option<String>,
    pub catalog_id: Urn,
}

/// Partial dataset update.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct EditDatasetDto {
    pub dct_conforms_to: Option<String>,
    pub dct_creator: Option<String>,
    pub dct_title: Option<String>,
    pub dct_description: Option<String>,
}

impl NewDatasetDto {
    /// Row owned by `owner`.
    pub fn into_model(self, owner: Owner) -> NewDatasetModel {
        NewDatasetModel {
            id: self.id,
            owner,
            dct_conforms_to: self.dct_conforms_to,
            dct_creator: self.dct_creator,
            dct_title: self.dct_title,
            dct_description: self.dct_description,
            catalog_id: self.catalog_id,
        }
    }
}

impl From<EditDatasetDto> for EditDatasetModel {
    fn from(dto: EditDatasetDto) -> Self {
        Self {
            dct_conforms_to: dto.dct_conforms_to,
            dct_creator: dto.dct_creator,
            dct_title: dto.dct_title,
            dct_description: dto.dct_description,
        }
    }
}

impl From<dataset::Model> for DatasetDto {
    fn from(value: Model) -> Self {
        Self { inner: value }
    }
}
