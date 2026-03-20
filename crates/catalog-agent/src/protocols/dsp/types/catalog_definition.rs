/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

// use db::catalog::entities::catalog;
use crate::protocols::dsp::types::dataservice_definition::{DataService, DataServiceMinimized};
use crate::protocols::dsp::types::dataset_definition::{Dataset, DatasetMinimized};
use crate::protocols::dsp::types::CatalogDspTraitDefinition;
use common::dsp_common::context_field::ContextField;
use common::dsp_common::odrl::OdrlOffer;
use serde::{Deserialize, Serialize};
use urn::Urn;

#[derive(Debug, Serialize, Deserialize)]
pub struct Catalog {
    #[serde(rename = "@context")]
    pub context: ContextField,
    #[serde(rename = "@type")]
    pub _type: String,
    #[serde(rename = "@id")]
    pub id: Urn,
    #[serde(flatten)]
    pub foaf: CatalogFoafDeclaration,
    #[serde(flatten)]
    pub dcat: CatalogDcatDeclaration,
    #[serde(flatten)]
    pub dct: CatalogDctDeclaration,
    #[serde(flatten)]
    pub dspace: CatalogDSpaceDeclaration,
    #[serde(rename = "hasPolicy")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub odrl_offer: Option<Vec<OdrlOffer>>,
    #[serde(rename = "extraFields")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_fields: Option<serde_json::Value>,
    #[serde(rename = "catalog")]
    pub catalogs: CatalogCatalogTypes,
    #[serde(rename = "dataset")]
    pub datasets: CatalogDatasetTypes,
    #[serde(rename = "service")]
    pub data_services: CatalogServiceTypes,
}

impl CatalogDspTraitDefinition for Catalog {}

#[derive(Debug, Serialize, Deserialize)]
pub struct CatalogMinimized {
    #[serde(rename = "@type")]
    pub _type: String,
    #[serde(rename = "@id")]
    pub id: Urn,
    #[serde(flatten)]
    pub foaf: CatalogFoafDeclaration,
    #[serde(flatten)]
    pub dcat: CatalogDcatDeclaration,
    #[serde(flatten)]
    pub dct: CatalogDctDeclaration,
    #[serde(flatten)]
    pub dspace: CatalogDSpaceDeclaration,
    #[serde(rename = "hasPolicy")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub odrl_offer: Option<Vec<OdrlOffer>>,
    #[serde(rename = "extraFields")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_fields: Option<serde_json::Value>,
    #[serde(rename = "dataset")]
    pub datasets: CatalogDatasetTypes,
    #[serde(rename = "service")]
    pub data_services: CatalogServiceTypes,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CatalogFoafDeclaration {
    #[serde(rename = "homepage")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CatalogDcatDeclaration {
    #[serde(rename = "theme")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(rename = "keyword")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyword: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CatalogDctDeclaration {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct CatalogDSpaceDeclaration {
    #[serde(rename = "participantId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participant_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CatalogCatalogTypes {
    CatalogMultipleMinimized(Vec<CatalogMinimized>),
    CatalogMultipleOriginal(Vec<Catalog>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum CatalogServiceTypes {
    ServiceOnly(DataService),
    ServiceMinimized(DataServiceMinimized),
    ServiceMultiple(Vec<DataService>),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CatalogDatasetTypes {
    DatasetMultipleMinimized(Vec<DatasetMinimized>),
    DatasetMultipleOriginal(Vec<Dataset>),
}

impl From<Catalog> for CatalogMinimized {
    fn from(dc: Catalog) -> Self {
        Self {
            _type: dc._type,
            id: dc.id,
            foaf: dc.foaf,
            dcat: dc.dcat,
            dct: dc.dct,
            dspace: dc.dspace,
            odrl_offer: dc.odrl_offer,
            extra_fields: dc.extra_fields,
            datasets: dc.datasets,
            data_services: dc.data_services,
        }
    }
}

impl From<&Catalog> for CatalogMinimized {
    fn from(dc: &Catalog) -> Self {
        dc.into()
    }
}
