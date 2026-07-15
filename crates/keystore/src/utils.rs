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

use ymir::errors::{Errors, Outcome};

/// Keys must follow an absolute path format: `/seg1/seg2/seg3`.
/// Rules:
///  - Must start with `/`
///  - No trailing `/`
///  - No empty segments (i.e. no `//`)
///  - Each segment: `[a-zA-Z0-9_.-]+`
pub(crate) fn validate_key(key: &str) -> Outcome<()> {
    if !key.starts_with('/') {
        return Err(Errors::validation(
            format!("invalid key '{}': must start with '/'", key),
            None,
        ));
    }
    if key.len() == 1 {
        return Err(Errors::validation(
            format!("invalid key '{}': must have at least one segment", key),
            None,
        ));
    }
    if key.ends_with('/') {
        return Err(Errors::validation(
            format!("invalid key '{}': must not end with '/'", key),
            None,
        ));
    }
    for segment in key[1..].split('/') {
        if segment.is_empty() {
            return Err(Errors::validation(
                format!("invalid key '{}': empty segment (double slash)", key),
                None,
            ));
        }
        if !segment
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
        {
            return Err(Errors::validation(
                format!(
                    "invalid key '{}': segment '{}' contains invalid characters (allowed: a-z A-Z 0-9 _ - .)",
                    key, segment
                ),
                None,
            ));
        }
    }
    Ok(())
}
