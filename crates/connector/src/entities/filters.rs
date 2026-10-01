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

//! Domain filters for connector entities.

use chrono::{DateTime, Utc};
use common::query::{validate_date_range, QueryFilter};
use serde::{Deserialize, Serialize};
use urn::Urn;
use ymir::errors::Outcome;

/// Filter criteria for querying connector templates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ConnectorTemplateFilter {
    pub tenant_id: Option<String>,
    pub name: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

impl QueryFilter for ConnectorTemplateFilter {
    fn is_empty(&self) -> bool {
        self.tenant_id.is_none()
            && self.name.is_none()
            && self.author.is_none()
            && self.version.is_none()
            && self.created_after.is_none()
            && self.created_before.is_none()
    }

    fn validate(&self) -> Outcome<()> {
        validate_date_range(self.created_after, self.created_before)
    }
}

/// Filter criteria for querying connector instances.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ConnectorInstanceFilter {
    pub tenant_id: Option<String>,
    pub distribution_id: Option<Urn>,
    pub template_name: Option<String>,
    pub template_version: Option<String>,
    pub author: Option<String>,
    pub owner_id: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

impl QueryFilter for ConnectorInstanceFilter {
    fn is_empty(&self) -> bool {
        self.tenant_id.is_none()
            && self.distribution_id.is_none()
            && self.template_name.is_none()
            && self.template_version.is_none()
            && self.author.is_none()
            && self.owner_id.is_none()
            && self.created_after.is_none()
            && self.created_before.is_none()
    }

    fn validate(&self) -> Outcome<()> {
        validate_date_range(self.created_after, self.created_before)
    }
}
