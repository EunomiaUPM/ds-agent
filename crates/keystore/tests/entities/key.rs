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

//! Key and KeyPrefix: which key paths are valid.

use keystore::KeyPrefix;
use keystore::entities::key::Key;

/// A rooted path of segments made of letters, digits, `_`, `-` and `.` is a valid key.
#[test]
fn rooted_path_with_allowed_characters_is_valid() {
    for key in ["/a", "/connectors/http/token", "/app_1/v-2/config.json"] {
        assert_eq!(Key::new(key).unwrap().as_str(), key);
    }
}

/// A key must start with `/` and have at least one segment.
#[test]
fn key_must_be_rooted_and_non_empty() {
    assert!(Key::new("connectors/http").is_err());
    assert!(Key::new("/").is_err());
    assert!(Key::new("").is_err());
}

/// A trailing slash or an empty segment is rejected.
#[test]
fn trailing_slash_or_empty_segment_is_rejected() {
    assert!(Key::new("/connectors/").is_err());
    assert!(Key::new("/connectors//http").is_err());
}

/// Characters outside `a-z A-Z 0-9 _ - .` are rejected, including spaces.
#[test]
fn other_characters_are_rejected() {
    for key in ["/conn ectors", "/a/b?c", "/a/ñ", "/a/b*"] {
        assert!(Key::new(key).is_err(), "{key} should be rejected");
    }
}

/// A key prints as its path.
#[test]
fn key_displays_as_its_path() {
    assert_eq!(Key::new("/a/b").unwrap().to_string(), "/a/b");
}

/// A prefix is not validated: any subtree can be listed.
#[test]
fn prefix_is_kept_as_given() {
    assert_eq!(KeyPrefix::new("/connectors/").as_str(), "/connectors/");
}
