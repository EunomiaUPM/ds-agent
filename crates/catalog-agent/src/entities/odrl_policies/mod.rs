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

use crate::data::entities::odrl_offer;
use crate::data::entities::odrl_offer::NewOdrlOfferModel;
use common::dsp_common::odrl::OdrlPolicyInfo;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use urn::Urn;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OdrlPolicyDto {
    #[serde(flatten)]
    pub inner: odrl_offer::Model,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum CatalogEntityTypes {
    Distribution,
    DataService,
    Catalog,
    Dataset,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NewOdrlPolicyDto {
    pub id: Option<Urn>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    pub odrl_offer: OdrlPolicyInfo,
    pub entity_id: Urn,
    pub entity_type: CatalogEntityTypes,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_template_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_template_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instantiation_parameters: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl std::str::FromStr for CatalogEntityTypes {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Distribution" => Ok(CatalogEntityTypes::Distribution),
            "DataService" => Ok(CatalogEntityTypes::DataService),
            "Catalog" => Ok(CatalogEntityTypes::Catalog),
            "Dataset" => Ok(CatalogEntityTypes::Dataset),
            other => Err(format!("unknown entity type: {other}")),
        }
    }
}

impl Display for CatalogEntityTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            CatalogEntityTypes::Distribution => "Distribution",
            CatalogEntityTypes::DataService => "DataService",
            CatalogEntityTypes::Catalog => "Catalog",
            CatalogEntityTypes::Dataset => "Dataset",
        }
        .to_string();
        write!(f, "{str}")
    }
}

impl NewOdrlPolicyDto {
    pub fn into_model(self, tenant_id: String) -> NewOdrlOfferModel {
        NewOdrlOfferModel {
            id: self.id,
            tenant_id,
            odrl_offer: self.odrl_offer,
            entity_id: self.entity_id,
            entity_type: self.entity_type,
            source_template_id: self.source_template_id,
            source_template_version: self.source_template_version,
            instantiation_parameters: self.instantiation_parameters,
            description: self.description,
        }
    }
}

impl From<odrl_offer::Model> for OdrlPolicyDto {
    fn from(value: odrl_offer::Model) -> Self {
        Self { inner: value }
    }
}
