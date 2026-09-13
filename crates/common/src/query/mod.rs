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

//! Query specification combining domain filters, pagination, and sorting.

pub mod filter;
pub mod spec;

pub use filter::{DateRange, FilterApplier, QueryFilter, validate_date_range};
pub use spec::QuerySpec;

// Re-exports from paginated_spec for unified access and backwards compatibility.
pub use crate::paginated_spec::{
    Cursor, DEFAULT_PAGE_LIMIT, MAX_BATCH_IDS, MAX_PAGE_LIMIT, Page, Paginated,
    PaginationParams, Sort, clamp_page_limit, default_limit,
};
