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

//! The DCAT classes the catalog keeps as resources of their own.

use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use urn::{Urn, UrnBuilder};
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::entities::vocab::Vocab;

/// What a catalog resource is; each one maps to one RDF class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceKind {
    Catalog,
    Dataset,
    DatasetSeries,
    Distribution,
    DataService,
    Agent,
    CatalogRecord,
}

/// Where a resource hangs from: its catalog, its dataset, or nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parent {
    Catalog,
    Dataset,
    None,
}

impl ResourceKind {
    pub const ALL: [ResourceKind; 7] = [
        Self::Catalog,
        Self::Dataset,
        Self::DatasetSeries,
        Self::Distribution,
        Self::DataService,
        Self::Agent,
        Self::CatalogRecord,
    ];

    /// The RDF class of the resource.
    pub fn class_iri(&self) -> &'static str {
        match self {
            Self::Catalog => Vocab::DCAT_CATALOG,
            Self::Dataset => Vocab::DCAT_DATASET,
            Self::DatasetSeries => Vocab::DCAT_DATASET_SERIES,
            Self::Distribution => Vocab::DCAT_DISTRIBUTION,
            Self::DataService => Vocab::DCAT_DATA_SERVICE,
            Self::Agent => Vocab::FOAF_AGENT,
            Self::CatalogRecord => Vocab::DCAT_CATALOG_RECORD,
        }
    }

    /// The class's local name, the `@type` term every context here uses.
    pub fn term(&self) -> &'static str {
        match self {
            Self::Catalog => "Catalog",
            Self::Dataset => "Dataset",
            Self::DatasetSeries => "DatasetSeries",
            Self::Distribution => "Distribution",
            Self::DataService => "DataService",
            Self::Agent => "Agent",
            Self::CatalogRecord => "CatalogRecord",
        }
    }

    /// Namespace of the URNs minted for it, e.g. `urn:dataset:<uuid>`.
    pub fn urn_nid(&self) -> &'static str {
        match self {
            Self::Catalog => "catalog",
            Self::Dataset => "dataset",
            Self::DatasetSeries => "dataset-series",
            Self::Distribution => "distribution",
            Self::DataService => "data-service",
            Self::Agent => "agent",
            Self::CatalogRecord => "catalog-record",
        }
    }

    /// Its value in the `kind` column.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Catalog => "catalog",
            Self::Dataset => "dataset",
            Self::DatasetSeries => "dataset_series",
            Self::Distribution => "distribution",
            Self::DataService => "data_service",
            Self::Agent => "agent",
            Self::CatalogRecord => "catalog_record",
        }
    }

    /// What the resource belongs to.
    pub fn parent(&self) -> Parent {
        match self {
            Self::Dataset | Self::DatasetSeries | Self::DataService | Self::CatalogRecord => {
                Parent::Catalog
            }
            Self::Distribution => Parent::Dataset,
            Self::Catalog | Self::Agent => Parent::None,
        }
    }

    /// The link from the parent to it (`dcat:dataset`, `dcat:distribution`…), if it has one.
    pub fn membership(&self) -> Option<&'static str> {
        match self {
            Self::Dataset | Self::DatasetSeries => Some(Vocab::DCAT_DATASET_P),
            Self::DataService => Some(Vocab::DCAT_SERVICE),
            Self::CatalogRecord => Some(Vocab::DCAT_RECORD),
            Self::Distribution => Some(Vocab::DCAT_DISTRIBUTION_P),
            Self::Catalog => Some(Vocab::DCAT_CATALOG_P),
            Self::Agent => None,
        }
    }

    /// Whether the connector has one of its own (the main catalog and data service).
    pub fn has_main(&self) -> bool {
        matches!(self, Self::Catalog | Self::DataService)
    }

    /// The kind whose class `iri` is; Organization and Person are agents too.
    pub fn from_class(iri: &str) -> Option<Self> {
        if iri == Vocab::FOAF_ORGANIZATION || iri == Vocab::FOAF_PERSON {
            return Some(Self::Agent);
        }
        Self::ALL.into_iter().find(|kind| kind.class_iri() == iri)
    }

    /// A fresh URN in the kind's namespace.
    pub fn mint(&self) -> Urn {
        UrnBuilder::new(self.urn_nid(), &uuid::Uuid::new_v4().to_string())
            .build()
            .expect("a UUID is a valid URN NSS")
    }
}

impl Display for ResourceKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ResourceKind {
    type Err = Errors;

    fn from_str(s: &str) -> Outcome<Self> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.as_str() == s || kind.term() == s)
            .ok_or_else(|| {
                Errors::format(BadFormat::Received, format!("unknown resource kind {s}"), None)
            })
    }
}
