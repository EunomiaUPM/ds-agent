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

//! Version: entry versions for optimistic concurrency.

use keystore::entities::version::Version;

/// Versions start at 1 and each update bumps them by one.
#[test]
fn versions_start_at_one_and_bump_by_one() {
    assert_eq!(Version::INITIAL.value(), 1);
    assert_eq!(Version::INITIAL.next().value(), 2);
    assert_eq!(Version::new(7).next(), Version::new(8));
}

/// Versions are ordered, so a stale one compares lower.
#[test]
fn versions_are_ordered() {
    assert!(Version::new(2) > Version::INITIAL);
}
