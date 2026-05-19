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

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::entities::entry::SecretEntry;
use crate::entities::metadata::Metadata;
use crate::entities::version::Version;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretView {
    pub key: String,
    pub version: u64,
    // Expose the plaintext value. Callers must ensure the transport is secured (TLS).
    pub value: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
    pub updated_by: String,
}

impl From<SecretEntry> for SecretView {
    fn from(e: SecretEntry) -> Self {
        Self {
            key: e.metadata.key.to_string(),
            version: e.metadata.version.value(),
            value: e.value.expose().to_owned(),
            description: e.metadata.description,
            created_at: e.metadata.created_at,
            updated_at: e.metadata.updated_at,
            created_by: e.metadata.created_by,
            updated_by: e.metadata.updated_by,
        }
    }
}

/// List responses deliberately omit the secret value.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretMetadataView {
    pub key: String,
    pub version: u64,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
    pub updated_by: String,
}

impl From<Metadata> for SecretMetadataView {
    fn from(m: Metadata) -> Self {
        Self {
            key: m.key.to_string(),
            version: m.version.value(),
            description: m.description,
            created_at: m.created_at,
            updated_at: m.updated_at,
            created_by: m.created_by,
            updated_by: m.updated_by,
        }
    }
}

#[derive(Serialize)]
pub struct VersionResponse {
    pub version: u64,
}

impl From<Version> for VersionResponse {
    fn from(v: Version) -> Self {
        Self { version: v.value() }
    }
}
