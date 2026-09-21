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

use common::errors::{NotFoundExt, ResourceError};
use ymir::errors::Errors;

#[test]
fn test_resource_error_not_found() {
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
        other => panic!("expected MissingResourceError, got {:?}", other),
    }
}

#[test]
fn test_not_found_ext_some() {
    let opt: Option<i32> = Some(100);
    let result = opt.or_not_found("item-1", "entity");
    assert_eq!(result.unwrap(), 100);
}

#[test]
fn test_not_found_ext_none() {
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
        other => panic!("expected MissingResourceError, got {:?}", other),
    }
}

#[test]
fn test_parse_urn_valid() {
    use common::utils::parse_urn;
    let urn = parse_urn("urn:uuid:12345678-1234-1234-1234-123456789abc").unwrap();
    assert_eq!(
        urn.to_string(),
        "urn:uuid:12345678-1234-1234-1234-123456789abc"
    );
}

#[test]
fn test_parse_urn_invalid() {
    use common::utils::parse_urn;
    let res = parse_urn("not-a-urn");
    assert!(res.is_err());
}

#[test]
fn test_parse_urn_ext() {
    use common::utils::ParseUrnExt;
    let urn = "urn:uuid:12345678-1234-1234-1234-123456789abc"
        .parse_urn()
        .unwrap();
    assert_eq!(
        urn.to_string(),
        "urn:uuid:12345678-1234-1234-1234-123456789abc"
    );
}
