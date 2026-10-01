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

//! Shared event payloads.

use serde::{Deserialize, Serialize};

/// Payload of every `delete` event: the id of the removed record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityDeletedDto {
    pub id: String,
}

impl EntityDeletedDto {
    pub fn new(id: impl ToString) -> Self {
        Self { id: id.to_string() }
    }
}
