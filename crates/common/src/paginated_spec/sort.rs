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

//! Sort order definitions for collections and repositories.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Standard collection sorting options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Sort {
    CreatedAtAsc,
    #[default]
    CreatedAtDesc,
    UpdatedAtAsc,
    UpdatedAtDesc,
    #[serde(other)]
    Other,
}

impl Sort {
    /// Returns true if the sort order is ascending.
    pub fn is_ascending(&self) -> bool {
        matches!(self, Self::CreatedAtAsc | Self::UpdatedAtAsc)
    }

    /// Returns true if the sort order is descending.
    pub fn is_descending(&self) -> bool {
        !self.is_ascending()
    }

    /// Wire name of the sort order (snake_case, matching its serde representation).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CreatedAtAsc => "created_at_asc",
            Self::CreatedAtDesc => "created_at_desc",
            Self::UpdatedAtAsc => "updated_at_asc",
            Self::UpdatedAtDesc => "updated_at_desc",
            Self::Other => "other",
        }
    }
}

impl fmt::Display for Sort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Strict parser: unknown names are an error (unlike serde, which maps them to `Other`).
impl FromStr for Sort {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "created_at_asc" => Ok(Self::CreatedAtAsc),
            "created_at_desc" => Ok(Self::CreatedAtDesc),
            "updated_at_asc" => Ok(Self::UpdatedAtAsc),
            "updated_at_desc" => Ok(Self::UpdatedAtDesc),
            other => Err(format!("unknown sort: {other}")),
        }
    }
}
