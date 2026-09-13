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

//! Domain filters for User queries.

use chrono::{DateTime, Utc};
use common::query::{QueryFilter, validate_date_range};
use serde::{Deserialize, Serialize};
use ymir::errors::Outcome;

use crate::entities::role::RbacRole;

/// Filter criteria for querying users.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserFilter {
    pub tenant_id: Option<String>,
    pub role: Option<RbacRole>,
    pub email: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

impl QueryFilter for UserFilter {
    fn is_empty(&self) -> bool {
        self.tenant_id.is_none()
            && self.role.is_none()
            && self.email.is_none()
            && self.created_after.is_none()
            && self.created_before.is_none()
    }

    fn validate(&self) -> Outcome<()> {
        validate_date_range(self.created_after, self.created_before)
    }
}
