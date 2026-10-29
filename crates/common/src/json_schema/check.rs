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

use serde::Serialize;
use serde_json::{json, Value};
use ymir::errors::{BadFormat, Errors, Outcome};

use super::JSON_SCHEMA_DIALECT;

/// One problem, at a JSON pointer into the document it is about.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SchemaIssue {
    pub path: String,
    pub message: String,
}

/// How one sample fared against a schema.
#[derive(Debug, Clone, Serialize)]
pub struct SampleReport {
    pub index: usize,
    pub valid: bool,
    pub issues: Vec<SchemaIssue>,
}

/// Checks schemas and samples; `$ref`s outside the schema are not resolved.
pub struct JsonSchemaCheck;

impl JsonSchemaCheck {
    /// What keeps `schema` from being a usable JSON Schema; empty when it is one.
    pub fn schema(schema: &Value) -> Vec<SchemaIssue> {
        if !schema.is_object() && !schema.is_boolean() {
            return vec![Self::issue("", "a JSON Schema is an object or a boolean")];
        }
        if let Err(e) = jsonschema::meta::validate(schema) {
            return vec![Self::issue(e.instance_path().as_str(), &e.to_string())];
        }
        match jsonschema::validator_for(schema) {
            Ok(_) => vec![],
            Err(e) => vec![Self::issue(e.schema_path().as_str(), &e.to_string())],
        }
    }

    /// Each sample against `schema`, which must be valid itself.
    pub fn samples(schema: &Value, samples: &[Value]) -> Outcome<Vec<SampleReport>> {
        let validator = jsonschema::validator_for(schema).map_err(|e| {
            Errors::format(
                BadFormat::Received,
                format!("invalid JSON Schema: {e}"),
                None,
            )
        })?;
        Ok(samples
            .iter()
            .enumerate()
            .map(|(index, sample)| {
                let issues: Vec<SchemaIssue> = validator
                    .iter_errors(sample)
                    .map(|e| Self::issue(e.instance_path().as_str(), &e.to_string()))
                    .collect();
                SampleReport {
                    index,
                    valid: issues.is_empty(),
                    issues,
                }
            })
            .collect())
    }

    /// `schema` as published at `id`: its `$id` is that URL and it names its dialect.
    pub fn published(schema: &Value, id: &str) -> Value {
        let mut published = schema.clone();
        if let Value::Object(object) = &mut published {
            object
                .entry("$schema")
                .or_insert_with(|| json!(JSON_SCHEMA_DIALECT));
            object.insert("$id".into(), json!(id));
        }
        published
    }

    fn issue(path: &str, message: &str) -> SchemaIssue {
        SchemaIssue {
            path: if path.is_empty() {
                "/".to_string()
            } else {
                path.to_string()
            },
            message: message.to_string(),
        }
    }
}
