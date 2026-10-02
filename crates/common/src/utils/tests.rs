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

//! URN parsing helpers: `parse_urn` and `ParseUrnExt`.

use crate::utils::{parse_urn, ParseUrnExt};

/// A valid URN parses and prints back unchanged.
#[test]
fn parse_urn_accepts_a_valid_urn() {
    let urn = parse_urn("urn:uuid:12345678-1234-1234-1234-123456789abc").unwrap();
    assert_eq!(
        urn.to_string(),
        "urn:uuid:12345678-1234-1234-1234-123456789abc"
    );
}

/// A string that is not a URN is rejected.
#[test]
fn parse_urn_rejects_a_non_urn() {
    let res = parse_urn("not-a-urn");
    assert!(res.is_err());
}

/// The extension trait parses a `&str` the same way.
#[test]
fn parse_urn_ext_parses_from_str() {
    let urn = "urn:uuid:12345678-1234-1234-1234-123456789abc"
        .parse_urn()
        .unwrap();
    assert_eq!(
        urn.to_string(),
        "urn:uuid:12345678-1234-1234-1234-123456789abc"
    );
}
