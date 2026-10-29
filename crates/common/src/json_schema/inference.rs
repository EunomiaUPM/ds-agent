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

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, NaiveDate};
use serde_json::{json, Map, Value};

use super::JSON_SCHEMA_DIALECT;

/// What the samples showed at one place of the documents.
#[derive(Default)]
struct Shape {
    types: BTreeSet<&'static str>,
    /// Each property with how many of the objects seen here had it.
    properties: BTreeMap<String, (Shape, usize)>,
    objects: usize,
    items: Option<Box<Shape>>,
    /// The format every string here matched; `None` once two disagree.
    format: Option<Option<&'static str>>,
}

/// Infers a schema that every sample satisfies; fields present in all samples are required.
pub struct JsonSchemaInference;

impl JsonSchemaInference {
    /// The schema of `samples`, each one a whole document.
    pub fn infer(samples: &[Value]) -> Value {
        let mut shape = Shape::default();
        for sample in samples {
            shape.observe(sample);
        }
        let mut schema = Map::new();
        schema.insert("$schema".into(), json!(JSON_SCHEMA_DIALECT));
        schema.extend(shape.schema());
        Value::Object(schema)
    }
}

impl Shape {
    fn observe(&mut self, value: &Value) {
        match value {
            Value::Null => {
                self.types.insert("null");
            }
            Value::Bool(_) => {
                self.types.insert("boolean");
            }
            Value::Number(n) => {
                self.types
                    .insert(if n.is_f64() { "number" } else { "integer" });
            }
            Value::String(s) => {
                self.types.insert("string");
                let format = Self::format_of(s);
                self.format = match self.format {
                    None => Some(format),
                    Some(seen) if seen == format => Some(seen),
                    Some(_) => Some(None),
                };
            }
            Value::Array(values) => {
                self.types.insert("array");
                let items = self.items.get_or_insert_with(Box::default);
                for item in values {
                    items.observe(item);
                }
            }
            Value::Object(object) => {
                self.types.insert("object");
                self.objects += 1;
                for (key, value) in object {
                    let (shape, seen) = self.properties.entry(key.clone()).or_default();
                    shape.observe(value);
                    *seen += 1;
                }
            }
        }
    }

    fn schema(&self) -> Map<String, Value> {
        let mut schema = Map::new();
        let mut types = self.types.clone();
        // An integer is a number: both seen means number.
        if types.contains("number") {
            types.remove("integer");
        }
        match types.len() {
            0 => {}
            1 => {
                schema.insert("type".into(), json!(types.iter().next()));
            }
            _ => {
                schema.insert("type".into(), json!(types));
            }
        }
        if let Some(Some(format)) = self.format {
            schema.insert("format".into(), json!(format));
        }
        if types.contains("object") {
            let properties: Map<String, Value> = self
                .properties
                .iter()
                .map(|(key, (shape, _))| (key.clone(), Value::Object(shape.schema())))
                .collect();
            let required: Vec<&String> = self
                .properties
                .iter()
                .filter(|(_, (_, seen))| *seen == self.objects)
                .map(|(key, _)| key)
                .collect();
            schema.insert("properties".into(), Value::Object(properties));
            if !required.is_empty() {
                schema.insert("required".into(), json!(required));
            }
        }
        if let Some(items) = &self.items {
            if !items.types.is_empty() {
                schema.insert("items".into(), Value::Object(items.schema()));
            }
        }
        schema
    }

    /// The JSON Schema format a string clearly is, if any.
    fn format_of(value: &str) -> Option<&'static str> {
        if DateTime::parse_from_rfc3339(value).is_ok() {
            Some("date-time")
        } else if value.len() == 10 && NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok() {
            Some("date")
        } else if uuid::Uuid::try_parse(value).is_ok() && value.len() == 36 {
            Some("uuid")
        } else if Self::is_email(value) {
            Some("email")
        } else if value.contains("://") && url::Url::parse(value).is_ok() {
            Some("uri")
        } else {
            None
        }
    }

    fn is_email(value: &str) -> bool {
        let Some((local, domain)) = value.split_once('@') else {
            return false;
        };
        !local.is_empty()
            && domain.contains('.')
            && !domain.starts_with('.')
            && !domain.ends_with('.')
            && !value.contains(char::is_whitespace)
    }
}
