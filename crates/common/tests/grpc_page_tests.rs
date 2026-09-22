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

use common::grpc::{PageMeta, PageParams};
use common::paginated_spec::{Paginated, Sort, DEFAULT_PAGE_LIMIT};
use tonic::Code;

#[test]
fn zero_limit_and_empty_strings_mean_defaults() {
    let (page, sort) = PageParams::from_proto(0, "", "").unwrap();
    assert_eq!(page.limit, DEFAULT_PAGE_LIMIT);
    assert_eq!(page.cursor, None);
    assert_eq!(sort, Sort::default());
}

#[test]
fn explicit_values_are_passed_through() {
    let (page, sort) = PageParams::from_proto(5, "abc", "updated_at_asc").unwrap();
    assert_eq!(page.limit, 5);
    assert_eq!(page.cursor.as_deref(), Some("abc"));
    assert_eq!(sort, Sort::UpdatedAtAsc);
}

#[test]
fn unknown_sort_is_invalid_argument() {
    let err = PageParams::from_proto(5, "", "sideways").unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert_eq!(err.message(), "sort: unknown sort: sideways");
}

#[test]
fn page_meta_maps_missing_cursor_and_total_to_proto_defaults() {
    let p: Paginated<u8> = Paginated::new(vec![1, 2], Some("next".into()), Some(42));
    assert_eq!(
        PageMeta::from(&p),
        PageMeta {
            next_cursor: "next".into(),
            total: 42
        }
    );

    let last: Paginated<u8> = Paginated::new(vec![3], None, None);
    assert_eq!(
        PageMeta::from(&last),
        PageMeta {
            next_cursor: String::new(),
            total: 0
        }
    );
}
