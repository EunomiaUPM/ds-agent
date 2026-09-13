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

//! Domain filters for catalog agent entities.

use chrono::{DateTime, Utc};
use common::query::{QueryFilter, validate_date_range};
use serde::{Deserialize, Serialize};
use ymir::errors::Outcome;

/// Filter criteria for querying catalogs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CatalogFilter {
    pub title: Option<String>,
    pub creator: Option<String>,
    pub participant_id: Option<String>,
    pub with_main_catalog: Option<bool>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

impl QueryFilter for CatalogFilter {
    fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.creator.is_none()
            && self.participant_id.is_none()
            && self.with_main_catalog.is_none()
            && self.created_after.is_none()
            && self.created_before.is_none()
    }

    fn validate(&self) -> Outcome<()> {
        validate_date_range(self.created_after, self.created_before)
    }
}

/// Filter criteria for querying datasets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DatasetFilter {
    pub catalog_id: Option<String>,
    pub title: Option<String>,
    pub creator: Option<String>,
    pub conforms_to: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

impl QueryFilter for DatasetFilter {
    fn is_empty(&self) -> bool {
        self.catalog_id.is_none()
            && self.title.is_none()
            && self.creator.is_none()
            && self.conforms_to.is_none()
            && self.created_after.is_none()
            && self.created_before.is_none()
    }

    fn validate(&self) -> Outcome<()> {
        validate_date_range(self.created_after, self.created_before)
    }
}

/// Filter criteria for querying distributions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DistributionFilter {
    pub dataset_id: Option<String>,
    pub access_service: Option<String>,
    pub format: Option<String>,
    pub title: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

impl QueryFilter for DistributionFilter {
    fn is_empty(&self) -> bool {
        self.dataset_id.is_none()
            && self.access_service.is_none()
            && self.format.is_none()
            && self.title.is_none()
            && self.created_after.is_none()
            && self.created_before.is_none()
    }

    fn validate(&self) -> Outcome<()> {
        validate_date_range(self.created_after, self.created_before)
    }
}

/// Filter criteria for querying data services.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DataServiceFilter {
    pub catalog_id: Option<String>,
    pub endpoint_url: Option<String>,
    pub title: Option<String>,
    pub creator: Option<String>,
    pub main_data_service: Option<bool>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

impl QueryFilter for DataServiceFilter {
    fn is_empty(&self) -> bool {
        self.catalog_id.is_none()
            && self.endpoint_url.is_none()
            && self.title.is_none()
            && self.creator.is_none()
            && self.main_data_service.is_none()
            && self.created_after.is_none()
            && self.created_before.is_none()
    }

    fn validate(&self) -> Outcome<()> {
        validate_date_range(self.created_after, self.created_before)
    }
}

/// Filter criteria for querying ODRL policies and offers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct OdrlPolicyFilter {
    pub entity: Option<String>,
    pub entity_type: Option<String>,
    pub source_template_id: Option<String>,
    pub source_template_version: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

impl QueryFilter for OdrlPolicyFilter {
    fn is_empty(&self) -> bool {
        self.entity.is_none()
            && self.entity_type.is_none()
            && self.source_template_id.is_none()
            && self.source_template_version.is_none()
            && self.created_after.is_none()
            && self.created_before.is_none()
    }

    fn validate(&self) -> Outcome<()> {
        validate_date_range(self.created_after, self.created_before)
    }
}

/// Filter criteria for querying policy templates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PolicyTemplateFilter {
    pub id: Option<String>,
    pub version: Option<String>,
    pub author: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

impl QueryFilter for PolicyTemplateFilter {
    fn is_empty(&self) -> bool {
        self.id.is_none()
            && self.version.is_none()
            && self.author.is_none()
            && self.created_after.is_none()
            && self.created_before.is_none()
    }

    fn validate(&self) -> Outcome<()> {
        validate_date_range(self.created_after, self.created_before)
    }
}
