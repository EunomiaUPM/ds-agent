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

use serde::{Deserialize, Serialize};

/// Standard collection sorting options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Sort {
    CreatedAtAsc,
    #[default]
    CreatedAtDesc,
    UpdatedAtDesc,
}

impl Sort {
    /// Returns true if the sort order is ascending.
    pub fn is_ascending(&self) -> bool {
        matches!(self, Self::CreatedAtAsc)
    }

    /// Returns true if the sort order is descending.
    pub fn is_descending(&self) -> bool {
        !self.is_ascending()
    }
}
