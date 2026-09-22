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

//! Cursor pagination bridging between proto list requests/responses and `Page`/`Sort`/`Paginated`.

use tonic::Status;

use crate::grpc::field::ProtoField;
use crate::paginated_spec::{Page, Paginated, Sort, DEFAULT_PAGE_LIMIT};

/// Parses the `limit` / `cursor` / `sort` triple every proto list request carries.
pub struct PageParams;

impl PageParams {
    /// `limit == 0` means the default limit; empty cursor / sort mean first page / default order.
    pub fn from_proto(limit: u32, cursor: &str, sort: &str) -> Result<(Page, Sort), Status> {
        let limit = if limit == 0 {
            DEFAULT_PAGE_LIMIT
        } else {
            limit
        };
        let page = Page::new(limit, cursor.non_empty().map(str::to_owned));
        let sort = sort.opt_parsed::<Sort>("sort")?.unwrap_or_default();
        Ok((page, sort))
    }
}

/// Proto-shaped page metadata (`""` = no next page, `0` = unknown total).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageMeta {
    pub next_cursor: String,
    pub total: u64,
}

impl<T> From<&Paginated<T>> for PageMeta {
    fn from(p: &Paginated<T>) -> Self {
        Self {
            next_cursor: p.next_cursor.clone().unwrap_or_default(),
            total: p.total.unwrap_or(0),
        }
    }
}
