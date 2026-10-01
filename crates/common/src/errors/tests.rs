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

//! ResourceError and NotFoundExt: how a missing resource becomes a 404.

use ymir::errors::Errors;

use crate::errors::{NotFoundExt, ResourceError};

/// A not-found error carries the resource id, a readable reason and status 404.
#[test]
fn not_found_error_carries_resource_and_404() {
    let err = ResourceError::not_found("item-42", "transfer process");
    match err {
        Errors::MissingResourceError {
            resource_id,
            reason,
            info,
            ..
        } => {
            assert_eq!(resource_id, "item-42");
            assert_eq!(reason, "transfer process not found");
            assert_eq!(info.status_code, 404);
        }
        other => panic!("expected MissingResourceError, got {other:?}"),
    }
}

/// `or_not_found` keeps a present value.
#[test]
fn or_not_found_keeps_a_present_value() {
    let opt: Option<i32> = Some(100);
    let result = opt.or_not_found("item-1", "entity");
    assert_eq!(result.unwrap(), 100);
}

/// `or_not_found` turns `None` into a 404 for that resource.
#[test]
fn or_not_found_turns_none_into_404() {
    let opt: Option<i32> = None;
    let result = opt.or_not_found("item-1", "entity");
    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
        Errors::MissingResourceError {
            resource_id,
            reason,
            info,
            ..
        } => {
            assert_eq!(resource_id, "item-1");
            assert_eq!(reason, "entity not found");
            assert_eq!(info.status_code, 404);
        }
        other => panic!("expected MissingResourceError, got {other:?}"),
    }
}
