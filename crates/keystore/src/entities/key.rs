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

//! Keys and key prefixes.

use serde::{Deserialize, Serialize};
use ymir::errors::{Errors, Outcome};

/// Path-like key such as `/connectors/http/token`, validated on creation.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Key(String);
impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Key {
    /// Fails unless the key passes `validate`.
    pub fn new(s: impl Into<String>) -> Outcome<Self> {
        let s = s.into();
        Self::validate(&s)?;
        Ok(Self(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Starts with `/`, has no empty segments, no trailing `/`, and only `a-z A-Z 0-9 _ - .`.
    pub fn validate(key: &str) -> Outcome<()> {
        if !key.starts_with('/') {
            return Err(Errors::validation(
                format!("invalid key '{key}': must start with '/'"),
                None,
            ));
        }
        if key.len() == 1 {
            return Err(Errors::validation(
                format!("invalid key '{key}': must have at least one segment"),
                None,
            ));
        }
        if key.ends_with('/') {
            return Err(Errors::validation(
                format!("invalid key '{key}': must not end with '/'"),
                None,
            ));
        }
        for segment in key[1..].split('/') {
            if segment.is_empty() {
                return Err(Errors::validation(
                    format!("invalid key '{key}': empty segment (double slash)"),
                    None,
                ));
            }
            if !segment
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
            {
                return Err(Errors::validation(
                    format!(
                        "invalid key '{key}': segment '{segment}' contains invalid characters (allowed: a-z A-Z 0-9 _ - .)"
                    ),
                    None,
                ));
            }
        }
        Ok(())
    }
}

/// Key prefix used to list a subtree; not validated.
#[derive(Clone, Debug, Default)]
pub struct KeyPrefix(String);

impl KeyPrefix {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
