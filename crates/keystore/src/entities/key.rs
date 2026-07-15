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

use crate::utils::validate_key;
use serde::{Deserialize, Serialize};
use ymir::errors::Outcome;

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Key(String);
impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Key {
    pub fn new(s: impl Into<String>) -> Outcome<Self> {
        let s = s.into();
        validate_key(&s)?;
        Ok(Self(s))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

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
