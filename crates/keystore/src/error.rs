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

use crate::entities::key::Key;
use crate::entities::version::Version;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum RegistryError {
    #[error("entry not found: {0}")]
    NotFound(Key),

    #[error("entry already exists: {0}")]
    AlreadyExists(Key),

    #[error("version conflict on {key}: expected {expected:?}, actual {actual:?}")]
    VersionConflict {
        key: Key,
        expected: Version,
        actual: Version,
    },

    #[error("invalid key: {0}")]
    InvalidKey(String),

    #[error("deserialization failed for key {key}: {source}")]
    Deserialize {
        key: Key,
        #[source]
        source: serde_json::Error,
    },

    #[error("unauthorized")]
    Unauthorized,

    #[error("backend error: {0}")]
    Backend(#[source] Box<dyn std::error::Error + Send + Sync>),
}
