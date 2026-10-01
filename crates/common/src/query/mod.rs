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
//!
//! A list endpoint receives one query string with three kinds of parameters: the resource's
//! own filters, the page and the sort. [`QuerySpec`] deserializes all of it at once and splits
//! it for the service. A filter is a plain struct that implements [`QueryFilter`] to validate
//! itself, and [`FilterApplier`] to turn itself into conditions on a SeaORM select. Page,
//! sort and cursor types come from [`crate::paginated_spec`] and are re-exported here.
//!
//! ## 1. Defining a filter
//!
//! Fields are optional and camelCase on the wire. `validate` runs before the repository is
//! called; [`DateRange`] covers the usual `createdAfter`/`createdBefore` pair.
//!
//! ```rust,ignore
//! use common::query::{validate_date_range, QueryFilter, QuerySpec};
//!
//! #[derive(Debug, Clone, Default, Serialize, Deserialize)]
//! #[serde(rename_all = "camelCase")]
//! pub struct ClientFilter {
//!     pub tenant_id: Option<String>,
//!     pub search: Option<String>,
//!     pub created_after: Option<DateTime<Utc>>,
//!     pub created_before: Option<DateTime<Utc>>,
//! }
//!
//! impl QueryFilter for ClientFilter {
//!     fn validate(&self) -> Outcome<()> {
//!         validate_date_range(self.created_after, self.created_before)
//!     }
//! }
//!
//! pub type ClientQuery = QuerySpec<ClientFilter, Sort>;
//! ```
//!
//! ## 2. Receiving it in a handler
//!
//! The filter and the page are flattened, so they sit side by side in the query string:
//! `?search=bob&createdAfter=2026-01-01T00:00:00Z&limit=50&sort=created_at_asc`.
//!
//! ```rust,ignore
//! async fn handle_list(
//!     State(s): State<Self>,
//!     scope: AccessScope,
//!     Query(q): Query<ClientQuery>,
//! ) -> AppResult<Json<Paginated<ClientView>>> {
//!     let (filter, page, sort) = q.into_domain();
//!     Ok(Json(s.client_svc.list_clients(&scope, &filter, &page, &sort).await?))
//! }
//! ```
//!
//! `validated()` clamps the page and runs both checks in one go when the handler wants to
//! reject a bad query before calling the service.
//!
//! ## 3. Applying it in the repository
//!
//! [`FilterApplier`] keeps the translation to SQL in the data layer, next to the entity.
//!
//! ```rust,ignore
//! use common::query::FilterApplier;
//!
//! impl FilterApplier<Select<dataset::Entity>> for DatasetFilter {
//!     fn apply_to(&self, mut q: Select<dataset::Entity>) -> Select<dataset::Entity> {
//!         if let Some(tenant_id) = &self.tenant_id {
//!             q = q.filter(dataset::Column::TenantId.eq(tenant_id));
//!         }
//!         if let Some(title) = &self.title {
//!             q = q.filter(dataset::Column::DctTitle.contains(title));
//!         }
//!         q
//!     }
//! }
//!
//! let q = filters.apply_to(dataset::Entity::find());
//! ```
//!
//! From there, paging continues as described in [`crate::paginated_spec`].

pub mod filter;
pub mod spec;

pub use filter::{validate_date_range, DateRange, FilterApplier, QueryFilter};
pub use spec::QuerySpec;

// Re-exports from paginated_spec for unified access and backwards compatibility.
pub use crate::paginated_spec::{
    clamp_page_limit, default_limit, Cursor, Page, Paginated, PaginationParams, Sort,
    DEFAULT_PAGE_LIMIT, MAX_BATCH_IDS, MAX_PAGE_LIMIT,
};
