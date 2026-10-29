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

//! Schema inference from samples, meta-schema checks and sample validation.

use serde_json::json;

use crate::json_schema::{JsonSchemaCheck, JsonSchemaInference, JSON_SCHEMA_DIALECT};

/// Scalars get their type, and the schema names the 2020-12 dialect.
#[test]
fn infers_scalar_types() {
    let schema = JsonSchemaInference::infer(&[json!({ "a": 1, "b": 1.5, "c": "x", "d": true })]);
    assert_eq!(schema["$schema"], JSON_SCHEMA_DIALECT);
    assert_eq!(schema["type"], "object");
    assert_eq!(schema["properties"]["a"]["type"], "integer");
    assert_eq!(schema["properties"]["b"]["type"], "number");
    assert_eq!(schema["properties"]["c"]["type"], "string");
    assert_eq!(schema["properties"]["d"]["type"], "boolean");
}

/// A field missing in one sample is optional; one in all of them is required.
#[test]
fn required_are_fields_in_every_sample() {
    let schema = JsonSchemaInference::infer(&[json!({ "id": 1, "name": "a" }), json!({ "id": 2 })]);
    assert_eq!(schema["required"], json!(["id"]));
}

/// Integer and decimal values together make a number; null joins the type list.
#[test]
fn merges_types_across_samples() {
    let schema =
        JsonSchemaInference::infer(&[json!({ "v": 1, "n": "x" }), json!({ "v": 2.5, "n": null })]);
    assert_eq!(schema["properties"]["v"]["type"], "number");
    assert_eq!(schema["properties"]["n"]["type"], json!(["null", "string"]));
}

/// Nested objects and arrays of objects are described all the way down.
#[test]
fn infers_nested_objects_and_arrays() {
    let schema = JsonSchemaInference::infer(&[json!({
        "station": { "lat": 40.4, "lon": -3.7 },
        "readings": [{ "t": 1 }, { "t": 2, "q": "ok" }]
    })]);
    let station = &schema["properties"]["station"];
    assert_eq!(station["type"], "object");
    assert_eq!(station["required"], json!(["lat", "lon"]));
    let items = &schema["properties"]["readings"]["items"];
    assert_eq!(items["type"], "object");
    assert_eq!(items["required"], json!(["t"]));
}

/// Strings that are clearly dates, emails, URIs or UUIDs carry the format.
#[test]
fn detects_string_formats() {
    let schema = JsonSchemaInference::infer(&[json!({
        "at": "2026-10-29T10:00:00Z",
        "day": "2026-10-29",
        "mail": "a@example.org",
        "web": "https://example.org/x",
        "id": "0e3f8a56-4b1c-4d0a-9f0e-6b5a1c2d3e4f",
        "plain": "hello"
    })]);
    let props = &schema["properties"];
    assert_eq!(props["at"]["format"], "date-time");
    assert_eq!(props["day"]["format"], "date");
    assert_eq!(props["mail"]["format"], "email");
    assert_eq!(props["web"]["format"], "uri");
    assert_eq!(props["id"]["format"], "uuid");
    assert!(props["plain"].get("format").is_none());
}

/// Two samples disagreeing on a string's format drop the format.
#[test]
fn disagreeing_formats_are_dropped() {
    let schema =
        JsonSchemaInference::infer(&[json!({ "x": "2026-10-29" }), json!({ "x": "hello" })]);
    assert!(schema["properties"]["x"].get("format").is_none());
}

/// The inferred schema is a valid JSON Schema that accepts its own samples.
#[test]
fn inferred_schema_accepts_its_samples() {
    let samples = [
        json!({ "id": 1, "tags": ["a"] }),
        json!({ "id": 2, "tags": [] }),
    ];
    let schema = JsonSchemaInference::infer(&samples);
    assert!(JsonSchemaCheck::schema(&schema).is_empty());
    let reports = JsonSchemaCheck::samples(&schema, &samples).unwrap();
    assert!(reports.iter().all(|r| r.valid));
}

/// A schema breaking the meta-schema is reported.
#[test]
fn reports_invalid_schema() {
    let issues = JsonSchemaCheck::schema(&json!({ "type": "nonsense" }));
    assert_eq!(issues.len(), 1);
    assert!(!JsonSchemaCheck::schema(&json!("text")).is_empty());
}

/// A sample that breaks the schema gets the path and reason.
#[test]
fn reports_invalid_samples_with_paths() {
    let schema = json!({
        "type": "object",
        "properties": { "id": { "type": "integer" } },
        "required": ["id"]
    });
    let reports =
        JsonSchemaCheck::samples(&schema, &[json!({ "id": "x" }), json!({ "id": 3 })]).unwrap();
    assert!(!reports[0].valid);
    assert_eq!(reports[0].issues[0].path, "/id");
    assert!(reports[1].valid);
}

/// Publishing sets `$id` to the URL and keeps a declared dialect.
#[test]
fn published_schema_names_its_url() {
    let published = JsonSchemaCheck::published(
        &json!({ "$schema": "https://json-schema.org/draft/2019-09/schema", "type": "object" }),
        "https://provider.example.org/s/1",
    );
    assert_eq!(published["$id"], "https://provider.example.org/s/1");
    assert_eq!(
        published["$schema"],
        "https://json-schema.org/draft/2019-09/schema"
    );
}
