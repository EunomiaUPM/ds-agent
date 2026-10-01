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

use crate::protocols::dsp::types::dataservice_definition::{DataService, DataServiceMinimized};
use crate::protocols::dsp::types::distribution_definition::{Distribution, DistributionMinimized};
use crate::protocols::dsp::types::CatalogDspTraitDefinition;
use common::dsp_common::context_field::ContextField;
use common::dsp_common::odrl::OdrlOffer;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dataset {
    #[serde(rename = "@context")]
    pub context: ContextField,
    #[serde(rename = "@type")]
    pub _type: String,
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(flatten)]
    pub dcat: DatasetDcatDeclaration,
    #[serde(flatten)]
    pub dct: DatasetDctDeclaration,
    #[serde(rename = "hasPolicy")]
    pub odrl_offer: Vec<OdrlOffer>,
    #[serde(rename = "extraFields")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_fields: Option<serde_json::Value>,
    #[serde(rename = "distribution")]
    pub distribution: DatasetDistributionTypes,
}

impl CatalogDspTraitDefinition for Dataset {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatasetMinimized {
    #[serde(rename = "@type")]
    pub _type: String,
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(flatten)]
    pub dcat: DatasetDcatDeclaration,
    #[serde(flatten)]
    pub dct: DatasetDctDeclaration,
    #[serde(rename = "hasPolicy")]
    pub odrl_offer: Vec<OdrlOffer>,
    #[serde(rename = "extraFields")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_fields: Option<serde_json::Value>,
    #[serde(rename = "distribution")]
    pub distribution: DatasetDistributionTypes,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatasetDcatDeclaration {
    #[serde(rename = "theme")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(rename = "keyword")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyword: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatasetDctDeclaration {
    #[serde(rename = "conformsTo")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conforms_to: Option<String>,
    #[serde(rename = "creator")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator: Option<String>,
    #[serde(rename = "identifier")]
    pub identifier: String,
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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum DatasetDistributionTypes {
    DistributionMultipleMinimized(Vec<DistributionMinimized>),
    DistributionMultipleOriginal(Vec<Distribution>),
}

impl From<Distribution> for DistributionMinimized {
    fn from(dc: Distribution) -> Self {
        Self {
            _type: dc._type,
            id: dc.id,
            dcat: dc.dcat,
            dct: dc.dct,
            odrl_offer: dc.odrl_offer,
            extra_fields: dc.extra_fields,
        }
    }
}

impl From<&Distribution> for DistributionMinimized {
    fn from(dc: &Distribution) -> Self {
        dc.clone().into()
    }
}

impl From<Dataset> for DatasetMinimized {
    fn from(dc: Dataset) -> Self {
        Self {
            _type: dc._type,
            id: dc.id,
            dcat: dc.dcat,
            dct: dc.dct,
            odrl_offer: dc.odrl_offer,
            extra_fields: dc.extra_fields,
            distribution: dc.distribution,
        }
    }
}

impl From<&Dataset> for DatasetMinimized {
    fn from(dc: &Dataset) -> Self {
        dc.clone().into()
    }
}
