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

//! Keyset pagination over the naive UTC timestamps of the events tables.

use common::paginated_spec::{Cursor, Page, Sort};
use sea_orm::sea_query::Condition;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Select};

pub(crate) struct NaiveKeyset;

impl NaiveKeyset {
    /// Orders by `(time, id)` and resumes after the page cursor, or skips to the page number.
    pub(crate) fn apply<E, T, I>(
        mut select: Select<E>,
        page: &Page,
        sort: &Sort,
        time_col: T,
        id_col: I,
    ) -> Select<E>
    where
        E: EntityTrait,
        T: ColumnTrait,
        I: ColumnTrait,
    {
        let ascending = sort.is_ascending();
        if let Some(cursor) = page.cursor.as_deref().and_then(|c| Cursor::decode(c).ok()) {
            let ts = cursor.timestamp_utc().naive_utc();
            let beyond = if ascending {
                time_col.gt(ts)
            } else {
                time_col.lt(ts)
            };
            let condition = match cursor.id {
                Some(id) => {
                    let tie = if ascending {
                        id_col.gt(id)
                    } else {
                        id_col.lt(id)
                    };
                    Condition::any()
                        .add(beyond)
                        .add(Condition::all().add(time_col.eq(ts)).add(tie))
                }
                None => Condition::all().add(beyond),
            };
            select = select.filter(condition);
        } else if let Some(p) = page.page.filter(|p| *p > 1) {
            select = select.offset(((p - 1) * page.limit) as u64);
        }
        select = if ascending {
            select.order_by_asc(time_col).order_by_asc(id_col)
        } else {
            select.order_by_desc(time_col).order_by_desc(id_col)
        };
        select.limit(page.limit as u64)
    }
}
