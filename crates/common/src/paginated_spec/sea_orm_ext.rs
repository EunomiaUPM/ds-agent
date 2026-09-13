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

//! SeaORM pagination extensions for cursor-based keyset pagination.

use std::borrow::Borrow;

use crate::paginated_spec::{Cursor, Page, Sort};
use sea_orm::sea_query::Condition;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Select};

/// Extension trait providing cursor-based pagination and sorting for SeaORM `Select<E>`.
pub trait SelectCursorExt<E: EntityTrait> {
    /// Applies cursor pagination and sorting using a timestamp column.
    fn apply_cursor_pagination<C: ColumnTrait, S: Borrow<Sort>>(
        self,
        page: &Page,
        sort: S,
        time_col: C,
    ) -> Self;

    /// Applies cursor pagination with deterministic tie-breaking using timestamp and ID columns.
    fn apply_cursor_pagination_with_tie_break<C: ColumnTrait, Id: ColumnTrait, S: Borrow<Sort>>(
        self,
        page: &Page,
        sort: S,
        time_col: C,
        id_col: Id,
    ) -> Self;
}

impl<E: EntityTrait> SelectCursorExt<E> for Select<E> {
    fn apply_cursor_pagination<C: ColumnTrait, S: Borrow<Sort>>(
        mut self,
        page: &Page,
        sort: S,
        time_col: C,
    ) -> Self {
        let sort = *sort.borrow();
        if let Some(cursor) = &page.cursor {
            if let Ok(c) = Cursor::decode(cursor) {
                self = match sort {
                    Sort::CreatedAtAsc => self.filter(time_col.gt(c.timestamp)),
                    _ => self.filter(time_col.lt(c.timestamp)),
                };
            }
        }

        self = match sort {
            Sort::CreatedAtAsc => self.order_by_asc(time_col),
            _ => self.order_by_desc(time_col),
        };

        self.limit(page.limit as u64)
    }

    fn apply_cursor_pagination_with_tie_break<C: ColumnTrait, Id: ColumnTrait, S: Borrow<Sort>>(
        mut self,
        page: &Page,
        sort: S,
        time_col: C,
        id_col: Id,
    ) -> Self {
        let sort = *sort.borrow();
        if let Some(cursor) = &page.cursor {
            if let Ok(c) = Cursor::decode(cursor) {
                match &c.id {
                    Some(id) => {
                        let cond = match sort {
                            Sort::CreatedAtAsc => Condition::any()
                                .add(time_col.gt(c.timestamp))
                                .add(
                                    Condition::all()
                                        .add(time_col.eq(c.timestamp))
                                        .add(id_col.gt(id.clone())),
                                ),
                            _ => Condition::any()
                                .add(time_col.lt(c.timestamp))
                                .add(
                                    Condition::all()
                                        .add(time_col.eq(c.timestamp))
                                        .add(id_col.lt(id.clone())),
                                ),
                        };
                        self = self.filter(cond);
                    }
                    None => {
                        self = match sort {
                            Sort::CreatedAtAsc => self.filter(time_col.gt(c.timestamp)),
                            _ => self.filter(time_col.lt(c.timestamp)),
                        };
                    }
                }
            }
        }

        self = match sort {
            Sort::CreatedAtAsc => self.order_by_asc(time_col).order_by_asc(id_col),
            _ => self.order_by_desc(time_col).order_by_desc(id_col),
        };

        self.limit(page.limit as u64)
    }
}
