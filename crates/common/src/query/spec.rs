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

//! Unified query specification binding domain filters, pagination, and sorting.

use serde::{Deserialize, Serialize};
use ymir::errors::Outcome;

use crate::paginated_spec::{Page, Sort};
use crate::query::filter::QueryFilter;

/// Unified query specification combining filters, pagination, and sort order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuerySpec<F = (), S = Sort> {
    #[serde(flatten)]
    pub filter: F,
    #[serde(flatten)]
    pub page: Page,
    #[serde(default)]
    pub sort: S,
}

impl<F: Default, S: Default> Default for QuerySpec<F, S> {
    fn default() -> Self {
        Self {
            filter: F::default(),
            page: Page::default(),
            sort: S::default(),
        }
    }
}

impl<F, S> QuerySpec<F, S> {
    /// Creates a new query specification.
    pub fn new(filter: F, page: Page, sort: S) -> Self {
        Self { filter, page, sort }
    }

    /// Decomposes the specification into constituent filter, page, and sort.
    pub fn into_parts(self) -> (F, Page, S) {
        (self.filter, self.page, self.sort)
    }

    /// Alias for into_parts matching domain conversion conventions.
    pub fn into_domain(self) -> (F, Page, S) {
        self.into_parts()
    }

    /// Replaces the filter with another filter instance.
    pub fn with_filter<NewF>(self, filter: NewF) -> QuerySpec<NewF, S> {
        QuerySpec {
            filter,
            page: self.page,
            sort: self.sort,
        }
    }

    /// Replaces the pagination configuration.
    pub fn with_page(mut self, page: Page) -> Self {
        self.page = page;
        self
    }

    /// Replaces the sorting order.
    pub fn with_sort(mut self, sort: S) -> Self {
        self.sort = sort;
        self
    }
}

impl<F: QueryFilter, S> QuerySpec<F, S> {
    /// Returns true if the inner filter has no criteria set.
    pub fn is_empty(&self) -> bool {
        self.filter.is_empty()
    }

    /// Validates pagination limits and filter invariants.
    pub fn validate(&self) -> Outcome<()> {
        self.page.validate()?;
        self.filter.validate()?;
        Ok(())
    }

    /// Clamps page limit to bounds and validates invariants.
    pub fn validated(mut self) -> Outcome<Self> {
        self.page = self.page.clamped();
        self.validate()?;
        Ok(self)
    }
}
