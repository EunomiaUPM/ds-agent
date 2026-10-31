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

//! JSON Schemas the catalog hosts for `conformsTo`: each version is immutable and has its own
//! public URL, so what a consumer agreed to never changes underneath.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ymir::types::oauth::RolePath;
use ymir::types::participants::Visibility;

/// One stored version of a schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSchema {
    pub id: String,
    pub version: i32,
    pub user_id: String,
    pub user_role: RolePath,
    pub visibility: Visibility,
    pub created_at: DateTime<FixedOffset>,
    pub title: String,
    pub description: Option<String>,
    /// The JSON Schema as it was saved, without the `$id` its URL gives it.
    pub content: Value,
    /// SHA-256 of `content`, hex.
    pub sha256: String,
}

ymir::impl_owned!(DataSchema);

/// A new schema, stored as its version 1.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NewSchemaDto {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub schema: Value,
    /// Private by default; a private schema is not served at its public URL.
    #[serde(default)]
    pub visibility: Option<Visibility>,
}

/// The next version of a schema; title and description default to the previous ones.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NewSchemaVersionDto {
    pub schema: Value,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// Sample documents to infer a schema from.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InferSchemaDto {
    pub samples: Vec<Value>,
}

/// A schema to check, and optionally samples to check against it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ValidateSchemaDto {
    pub schema: Value,
    #[serde(default)]
    pub samples: Vec<Value>,
}

/// An external URL said to hold a JSON Schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CheckSchemaUrlDto {
    pub url: String,
}
