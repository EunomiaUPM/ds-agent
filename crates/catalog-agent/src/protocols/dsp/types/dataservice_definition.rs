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

use crate::protocols::dsp::types::EntityTypes;
use common::dsp_common::context_field::ContextField;
use common::dsp_common::odrl::OdrlOffer;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataService {
    #[serde(rename = "@context")]
    pub context: ContextField,
    #[serde(rename = "@type")]
    pub _type: String,
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(flatten)]
    pub dcat: DataServiceDcatDeclaration,
    #[serde(flatten)]
    pub dct: DataServiceDctDeclaration,
    #[serde(rename = "hasPolicy")]
    pub odrl_offer: Vec<OdrlOffer>,
    #[serde(rename = "extraFields")]
    #[serde(skip_serializing_if = "serde_json::Value::is_null")]
    pub extra_fields: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataServiceMinimized {
    #[serde(rename = "@type")]
    pub _type: String,
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(flatten)]
    pub dcat: DataServiceDcatDeclaration,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataServiceDcatDeclaration {
    #[serde(rename = "theme")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(rename = "keyword")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyword: Option<String>,
    #[serde(rename = "endpointDescription")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint_description: Option<String>,
    #[serde(rename = "endpointURL")]
    pub endpoint_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataServiceDctDeclaration {
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

impl Default for DataService {
    fn default() -> Self {
        DataService {
            context: ContextField::default(),
            _type: EntityTypes::DataService.to_string(),
            id: "".to_string(),
            dcat: DataServiceDcatDeclaration {
                theme: None,
                keyword: None,
                endpoint_description: None,
                endpoint_url: "".to_string(),
            },
            dct: DataServiceDctDeclaration {
                conforms_to: None,
                creator: None,
                identifier: "".to_string(),
                issued: chrono::Utc::now().naive_utc(),
                modified: None,
                title: None,
                description: vec![],
            },
            odrl_offer: vec![],
            extra_fields: Value::default(),
        }
    }
}

impl From<DataService> for DataServiceMinimized {
    fn from(dc: DataService) -> Self {
        Self {
            _type: dc._type,
            id: dc.id,
            dcat: dc.dcat,
        }
    }
}

impl From<&DataService> for DataServiceMinimized {
    fn from(dc: &DataService) -> Self {
        dc.clone().into()
    }
}
