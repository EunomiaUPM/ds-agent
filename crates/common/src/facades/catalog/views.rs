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

//! The catalog records as they cross agents: only what consumers read.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A dataset (DCAT) of the catalog.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogDataset {
    pub id: String,
    pub catalog_id: String,
    pub dct_title: Option<String>,
}

/// A distribution of a dataset, with the data service that gives access to it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogDistribution {
    pub id: String,
    pub dataset_id: String,
    pub dct_format: Option<String>,
    pub dcat_access_service: String,
}

/// An ODRL offer attached to a catalog entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogOffer {
    pub id: String,
    pub entity: String,
    pub entity_type: String,
    pub odrl_offer: Value,
}
