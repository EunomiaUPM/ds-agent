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

//! What the schema API answers.

use chrono::{DateTime, FixedOffset};
use common::json_schema::{SampleReport, SchemaIssue};
use serde::Serialize;
use serde_json::Value;
use ymir::types::participants::Visibility;

use crate::entities::schemas::DataSchema;

/// A schema version without its content.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaView {
    pub id: String,
    pub version: i32,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub visibility: Visibility,
    pub created_at: DateTime<FixedOffset>,
    pub sha256: String,
    /// Where anyone dereferences it; what `schema` / `conformsTo` point to.
    pub url: String,
}

impl SchemaView {
    pub fn assemble(schema: DataSchema, url: String) -> Self {
        Self {
            id: schema.id,
            version: schema.version,
            title: schema.title,
            description: schema.description,
            visibility: schema.visibility,
            created_at: schema.created_at,
            sha256: schema.sha256,
            url,
        }
    }
}

/// A resource whose own `schema` is a version.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaUsageView {
    pub id: String,
    pub kind: String,
    pub title: Option<String>,
}

/// A version with its content and who uses it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVersionView {
    #[serde(flatten)]
    pub summary: SchemaView,
    /// The content as published, `$id` included.
    pub schema: Value,
    pub used_by: Vec<SchemaUsageView>,
}

/// A schema's own problems and how each sample fared.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaValidationView {
    pub valid: bool,
    pub issues: Vec<SchemaIssue>,
    pub samples: Vec<SampleReport>,
}

/// What an external schema URL answered.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaUrlCheckView {
    pub url: String,
    pub reachable: bool,
    pub is_json_schema: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub issues: Vec<SchemaIssue>,
}
