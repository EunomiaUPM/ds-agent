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

//! Domain query filter trait and reusable date range validation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ymir::errors::{BadFormat, Errors, Outcome};

/// Domain filter validation trait.
pub trait QueryFilter: Send + Sync {
    /// Returns true if the filter contains no criteria.
    fn is_empty(&self) -> bool {
        true
    }

    /// Validates filter invariants before repository execution.
    fn validate(&self) -> Outcome<()> {
        Ok(())
    }
}

impl QueryFilter for () {}

/// Port for applying domain filter criteria to query builders or collections.
pub trait FilterApplier<Target> {
    /// Applies domain filter criteria to the target query builder or collection.
    fn apply_to(&self, target: Target) -> Target;
}

/// Encapsulates temporal boundaries with ordering validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DateRange {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_after: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_before: Option<DateTime<Utc>>,
}

impl DateRange {
    /// Creates a new date range filter.
    pub fn new(after: Option<DateTime<Utc>>, before: Option<DateTime<Utc>>) -> Self {
        Self {
            created_after: after,
            created_before: before,
        }
    }

    /// Validates that created_after is strictly before created_before.
    pub fn validate(&self) -> Outcome<()> {
        Self::validate_bounds(self.created_after, self.created_before)
    }

    /// Validates that an after timestamp precedes a before timestamp.
    pub fn validate_bounds(after: Option<DateTime<Utc>>, before: Option<DateTime<Utc>>) -> Outcome<()> {
        if let (Some(a), Some(b)) = (after, before) {
            if a >= b {
                return Err(Errors::format(
                    BadFormat::Received,
                    "createdAfter must be strictly before createdBefore",
                    None,
                ));
            }
        }
        Ok(())
    }
}

/// Backwards compatible function for validating date range boundaries.
pub fn validate_date_range(
    after: Option<DateTime<Utc>>,
    before: Option<DateTime<Utc>>,
) -> Outcome<()> {
    DateRange::validate_bounds(after, before)
}
