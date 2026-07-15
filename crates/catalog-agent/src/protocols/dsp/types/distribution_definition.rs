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

use crate::protocols::dsp::types::catalog_definition::CatalogServiceTypes;
use crate::protocols::dsp::types::dataservice_definition::DataService;
use common::dsp_common::context_field::ContextField;
use common::dsp_common::odrl::OdrlOffer;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Distribution {
    #[serde(rename = "@context")]
    pub context: ContextField,
    #[serde(rename = "@type")]
    pub _type: String,
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(flatten)]
    pub dcat: DistributionDcatDeclaration,
    #[serde(flatten)]
    pub dct: DistributionDctDeclaration,
    #[serde(rename = "hasPolicy")]
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub odrl_offer: Vec<OdrlOffer>,
    #[serde(rename = "extraFields")]
    #[serde(skip_serializing_if = "serde_json::Value::is_null")]
    pub extra_fields: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DistributionMinimized {
    #[serde(rename = "@type")]
    pub _type: String,
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(flatten)]
    pub dcat: DistributionDcatDeclaration,
    #[serde(flatten)]
    pub dct: DistributionDctDeclaration,
    #[serde(rename = "hasPolicy")]
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    #[serde(skip)]
    pub odrl_offer: Vec<OdrlOffer>,
    #[serde(rename = "extraFields")]
    #[serde(skip_serializing_if = "serde_json::Value::is_null")]
    #[serde(skip)]
    pub extra_fields: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DistributionDcatDeclaration {
    #[serde(rename = "accessService")]
    pub access_service: Option<CatalogServiceTypes>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DistributionDctDeclaration {
    #[serde(rename = "issued")]
    pub issued: chrono::NaiveDateTime,
    #[serde(rename = "modified")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<chrono::NaiveDateTime>,
    #[serde(rename = "title")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(rename = "description")]
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub description: Vec<String>,
    #[serde(rename = "formats")]
    pub formats: String,
}
