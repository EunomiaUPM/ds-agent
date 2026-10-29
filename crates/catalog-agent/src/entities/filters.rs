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

//! Query filters over resources, offers and templates; every one runs in the database.

use chrono::{DateTime, Utc};
use common::query::{DateRange, QueryFilter};
use serde::{Deserialize, Serialize};
use ymir::errors::Outcome;

use crate::entities::resource_kind::ResourceKind;

/// Filter criteria for querying resources of one kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ResourceFilter {
    /// Set by the route, never by the query string.
    #[serde(skip)]
    pub kind: Option<ResourceKind>,
    /// Only the records of this user, among those the caller sees.
    pub user_id: Option<String>,
    pub catalog_id: Option<String>,
    pub dataset_id: Option<String>,
    /// Part of the title, in any language.
    pub title: Option<String>,
    /// A `dcat:keyword`, exactly.
    pub keyword: Option<String>,
    /// A `dcat:theme` IRI.
    pub theme: Option<String>,
    /// A `dct:format`, literal or IRI.
    pub format: Option<String>,
    pub creator: Option<String>,
    pub publisher: Option<String>,
    pub conforms_to: Option<String>,
    /// The data schema URL (`schema`), own, not inherited.
    pub schema: Option<String>,
    pub participant_id: Option<String>,
    pub access_service: Option<String>,
    pub endpoint_url: Option<String>,
    /// `false` leaves out the connector's main resource; `true` keeps only it.
    pub main: Option<bool>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

impl ResourceFilter {
    /// Every resource of `kind`.
    pub fn of(kind: ResourceKind) -> Self {
        Self {
            kind: Some(kind),
            ..Self::default()
        }
    }
}

impl QueryFilter for ResourceFilter {
    fn is_empty(&self) -> bool {
        self.user_id.is_none()
            && self.catalog_id.is_none()
            && self.dataset_id.is_none()
            && self.title.is_none()
            && self.keyword.is_none()
            && self.theme.is_none()
            && self.format.is_none()
            && self.creator.is_none()
            && self.publisher.is_none()
            && self.conforms_to.is_none()
            && self.schema.is_none()
            && self.participant_id.is_none()
            && self.access_service.is_none()
            && self.endpoint_url.is_none()
            && self.main.is_none()
            && self.created_after.is_none()
            && self.created_before.is_none()
    }

    fn validate(&self) -> Outcome<()> {
        DateRange::validate_bounds(self.created_after, self.created_before)
    }
}

/// Filter criteria for querying ODRL offers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct OdrlPolicyFilter {
    /// Only the records of this user, among those the caller sees.
    pub user_id: Option<String>,
    pub entity: Option<String>,
    pub entity_type: Option<String>,
    pub source_template_id: Option<String>,
    pub source_template_version: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

impl QueryFilter for OdrlPolicyFilter {
    fn is_empty(&self) -> bool {
        self.user_id.is_none()
            && self.entity.is_none()
            && self.entity_type.is_none()
            && self.source_template_id.is_none()
            && self.source_template_version.is_none()
            && self.created_after.is_none()
            && self.created_before.is_none()
    }

    fn validate(&self) -> Outcome<()> {
        DateRange::validate_bounds(self.created_after, self.created_before)
    }
}

/// Filter criteria for querying policy templates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PolicyTemplateFilter {
    /// Only the records of this user, among those the caller sees.
    pub user_id: Option<String>,
    pub id: Option<String>,
    pub version: Option<String>,
    pub author: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

/// Filter criteria for querying schema versions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SchemaFilter {
    pub id: Option<String>,
    /// Case-insensitive part of the title.
    pub title: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

impl QueryFilter for SchemaFilter {
    fn is_empty(&self) -> bool {
        self.id.is_none()
            && self.title.is_none()
            && self.created_after.is_none()
            && self.created_before.is_none()
    }

    fn validate(&self) -> Outcome<()> {
        DateRange::validate_bounds(self.created_after, self.created_before)
    }
}

impl QueryFilter for PolicyTemplateFilter {
    fn is_empty(&self) -> bool {
        self.user_id.is_none()
            && self.id.is_none()
            && self.version.is_none()
            && self.author.is_none()
            && self.created_after.is_none()
            && self.created_before.is_none()
    }

    fn validate(&self) -> Outcome<()> {
        DateRange::validate_bounds(self.created_after, self.created_before)
    }
}
