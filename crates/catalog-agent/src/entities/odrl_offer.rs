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

//! ODRL offers attached to catalog resources. They stay ODRL JSON as they are today: modelling
//! ODRL as RDF is a plan of its own.

use std::fmt::Display;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use urn::Urn;
use ymir::oauth::{OwnedTrait, Owner};

use crate::entities::resource_kind::ResourceKind;

/// Kind of catalog resource an offer is attached to.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum CatalogEntityTypes {
    Distribution,
    DataService,
    Catalog,
    Dataset,
}

impl CatalogEntityTypes {
    /// The resource kind it names.
    pub fn kind(&self) -> ResourceKind {
        match self {
            Self::Distribution => ResourceKind::Distribution,
            Self::DataService => ResourceKind::DataService,
            Self::Catalog => ResourceKind::Catalog,
            Self::Dataset => ResourceKind::Dataset,
        }
    }
}

impl FromStr for CatalogEntityTypes {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Distribution" => Ok(Self::Distribution),
            "DataService" => Ok(Self::DataService),
            "Catalog" => Ok(Self::Catalog),
            "Dataset" => Ok(Self::Dataset),
            other => Err(format!("unknown entity type: {other}")),
        }
    }
}

impl Display for CatalogEntityTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Distribution => "Distribution",
            Self::DataService => "DataService",
            Self::Catalog => "Catalog",
            Self::Dataset => "Dataset",
        })
    }
}

/// An ODRL offer on a catalog resource, possibly instantiated from a template.
#[derive(Debug, Clone, PartialEq)]
pub struct OdrlOffer {
    pub id: Urn,
    pub owner: Owner,
    /// The ODRL rules (`permission`, `prohibition`, `obligation`) as JSON.
    pub odrl_offer: Value,
    pub entity: Urn,
    pub entity_type: String,
    pub created_at: DateTime<Utc>,
    pub source_template_id: Option<String>,
    pub source_template_version: Option<String>,
    pub instantiation_parameters: Option<Value>,
    pub description: Option<String>,
    /// The parts the offer was composed from, as asked; none for offers written as plain ODRL.
    pub composition: Option<Value>,
}

impl OwnedTrait for OdrlOffer {
    fn owner(&self) -> Owner {
        self.owner.clone()
    }
}
