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

//! Standard pagination, sorting, and cursor management.
//!
//! Every list endpoint takes a [`Page`] and a [`Sort`] and returns a [`Paginated`]. Paging is
//! keyset-based: the response carries an opaque `nextCursor` built from the last item's
//! timestamp and id, and the next request resumes right after it. Paging, filtering and
//! sorting always happen in the database query, never on a list already in memory.
//!
//! ## 1. The request
//!
//! [`Page`] has a `limit` (default 20, clamped to 1..=100), an optional `cursor` and an optional
//! 1-based `page` index for offset paging. [`Sort`] orders by creation or update time,
//! newest first by default. Numbers are accepted both as JSON integers and as query strings.
//!
//! ```text
//! GET /api/v1/catalog-agent/datasets?limit=50&sort=updated_at_desc&cursor=MjAyNi0wOS0...
//! ```
//!
//! HTTP adapters usually take the whole query as a `QuerySpec<Filter, Sort>` from
//! [`crate::query`], which splits into filter, page and sort.
//!
//! ## 2. Paging a SeaORM select
//!
//! [`SelectCursorExt`] adds the cursor condition, the order and the limit to a `Select`. Use the
//! tie-break variant whenever two rows can share a timestamp; it orders by the id column too.
//!
//! ```rust,ignore
//! use common::paginated_spec::{Page, SelectCursorExt, Sort};
//!
//! async fn get_all_datasets(&self, filters: &DatasetFilter, page: &Page, sort: &Sort)
//!     -> Outcome<(Vec<dataset::Model>, Option<u64>)>
//! {
//!     let q = filters.apply_to(dataset::Entity::find());
//!     let total = q.clone().count(&self.db_connection).await.map_err(/* repo error */)?;
//!     let rows = q
//!         .apply_cursor_pagination_with_tie_break(
//!             page,
//!             sort,
//!             dataset::Column::DctIssued,
//!             dataset::Column::Id,
//!         )
//!         .all(&self.db_connection)
//!         .await
//!         .map_err(/* repo error */)?;
//!     Ok((rows, Some(total)))
//! }
//! ```
//!
//! With a cursor the query resumes after it; without one, `page` becomes an offset.
//!
//! ## 3. Paging through ymir repositories
//!
//! Repositories that live in ymir take a `ListPage` instead. [`Page::list_page`] builds it from
//! the sort and the two columns that stand for "created" and "updated" in that repository.
//!
//! ```rust,ignore
//! let list_page =
//!     page.list_page(sort, ParticipantSort::SavedAt, ParticipantSort::LastInteraction)?;
//! let listed = self.repo().participant().find_page(&list_filter, &list_page).await?;
//! ```
//!
//! ## 4. Building the response in the service
//!
//! Clamp the page before querying, then let [`Paginated::from_page`] decide whether there is a
//! next page and encode its cursor from the last item. The cursor must use the same columns
//! the repository sorted by.
//!
//! ```rust,ignore
//! use common::paginated_spec::{Cursor, Page, Paginated, Sort};
//!
//! let page = page.clamped();
//! let (datasets, total) =
//!     self.repo.get_dataset_repo().get_all_datasets(&filters, &page, sort).await?;
//! let dtos: Vec<DatasetDto> = datasets.into_iter().map(Into::into).collect();
//! Ok(Paginated::from_page(dtos, &page, total, |d| {
//!     Cursor::encode_composite(&d.inner.dct_issued, &d.inner.id)
//! }))
//! ```
//!
//! `Paginated::from_window` does the same for queries that fetched `limit + 1` rows (see
//! `Page::fetch_limit`), and `map` converts the items while keeping cursor and total.
//!
//! ## 5. What the client gets
//!
//! ```json
//! { "items": [ ... ], "nextCursor": "MjAyNi0wOS0yOVQxMDo...", "total": 134 }
//! ```
//!
//! `nextCursor` is absent on the last page. HTTP handlers also send the total as
//! `X-Total-Count` through `ExtractedHeaders::response_headers_paged`.
//!
//! ## 6. Cursors
//!
//! A [`Cursor`] is URL-safe base64 over an RFC 3339 timestamp, optionally followed by `#` and the
//! row id. Clients treat it as opaque; a malformed one is a 400.
//!
//! ```rust,ignore
//! let cursor = Cursor::encode_composite(&row.created_at, &row.id);
//! let decoded = Cursor::decode(&cursor)?; // DecodedCursor { timestamp, id: Some(..) }
//! ```
//!
//! [`PaginationParams`] is the older limit/page pair; `to_page` turns it into a [`Page`].

pub mod cursor;
pub mod page;
pub mod paginated;
pub mod sea_orm_ext;
pub mod sort;

pub use cursor::{Cursor, DecodedCursor};
pub use page::{
    clamp_page_limit, default_limit, deserialize_opt_bool_from_str_or_bool,
    deserialize_opt_u32_from_str_or_int, deserialize_opt_u64_from_str_or_int,
    deserialize_u32_from_str_or_int, Page, PaginationParams, DEFAULT_PAGE_LIMIT, MAX_BATCH_IDS,
    MAX_PAGE_LIMIT,
};
pub use paginated::Paginated;
pub use sea_orm_ext::SelectCursorExt;
pub use sort::Sort;

#[cfg(test)]
mod tests;
