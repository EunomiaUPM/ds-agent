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
