/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use super::*;
use serde_json::json;

fn make_template_parameters(name: &str, param_type: ParameterType) -> ParameterDefinition {
    ParameterDefinition {
        name: name.to_string(),
        title: name.to_string(),
        description: None,
        param_type,
        required: true,
        default_value: None,
    }
}

fn make_optional_param(name: &str, param_type: ParameterType) -> ParameterDefinition {
    ParameterDefinition {
        required: false,
        ..make_template_parameters(name, param_type)
    }
}

fn make_instance_parameters(name: &str, value: Value) -> (String, Value) {
    (name.to_string(), value)
}

// =========================================================================
// Base
// =========================================================================

#[test]
fn ok_when_all_params_match_definitions() {
    let defs = vec![
        make_template_parameters("HOST", ParameterType::String),
        make_template_parameters("PORT", ParameterType::Int),
    ];
    let validator = InstanceParametersValidator::new(&defs);
    let result = validator.validate(&HashMap::from([
        make_instance_parameters("HOST", json!("localhost")),
        make_instance_parameters("PORT", json!(8080)),
    ]));
    assert_eq!(result.len(), 0);
}

// =========================================================================
// Rule A: SYS OR RUNTIME
// =========================================================================

#[test]
fn ok_when_sys_or_runtime_param_is_not_provided() {
    let defs = vec![make_template_parameters("TOKEN", ParameterType::String)];
    let validator = InstanceParametersValidator::new(&defs);
    let result = validator.validate(&HashMap::new());
    assert_eq!(result.len(), 0);
}

#[test]
fn err_when_sys_param_is_manually_set() {
    let defs = vec![make_template_parameters("SYS_TOKEN", ParameterType::String)];
    let validator = InstanceParametersValidator::new(&defs);
    let result = validator.validate(&HashMap::from([make_instance_parameters(
        "TOKEN",
        json!("manual_value"),
    )]));
    assert_eq!(result.len(), 1);
    assert!(result[0].contains("TOKEN"));
    assert!(result[0].contains("auto-filled"));
}

#[test]
fn err_when_runtime_param_is_manually_set() {
    let defs = vec![make_template_parameters(
        "RUNTIME_TOKEN",
        ParameterType::String,
    )];
    let validator = InstanceParametersValidator::new(&defs);
    let result = validator.validate(&HashMap::from([make_instance_parameters(
        "TOKEN",
        json!("manual_value"),
    )]));
    assert_eq!(result.len(), 1);
    assert!(result[0].contains("TOKEN"));
    assert!(result[0].contains("auto-filled"));
}

// =========================================================================
// Rule B: Required vs Optional
// =========================================================================

#[test]
fn err_when_required_param_is_missing() {
    let defs = vec![make_template_parameters("HOST", ParameterType::String)];
    let validator = InstanceParametersValidator::new(&defs);
    let result = validator.validate(&HashMap::new());
    assert_eq!(result.len(), 1);
    assert!(result[0].contains("HOST"));
    assert!(result[0].contains("Missing required"));
}

#[test]
fn ok_when_optional_param_is_absent() {
    let defs = vec![make_optional_param("TIMEOUT", ParameterType::Int)];
    let validator = InstanceParametersValidator::new(&defs);
    let result = validator.validate(&HashMap::new());
    assert_eq!(result.len(), 0);
}

// =========================================================================
// Rule C: Type validation
// =========================================================================

#[test]
fn ok_when_string_param_receives_string_value() {
    let defs = vec![make_template_parameters("NAME", ParameterType::String)];
    let validator = InstanceParametersValidator::new(&defs);
    let result = validator.validate(&HashMap::from([make_instance_parameters(
        "NAME",
        json!("alice"),
    )]));
    assert_eq!(result.len(), 0);
}

#[test]
fn err_when_string_param_receives_int_value() {
    let defs = vec![make_template_parameters("NAME", ParameterType::String)];
    let validator = InstanceParametersValidator::new(&defs);
    let result = validator.validate(&HashMap::from([make_instance_parameters(
        "NAME",
        json!(42),
    )]));
    assert_eq!(result.len(), 1);
    assert!(result[0].contains("NAME"));
    assert!(result[0].contains("Type mismatch"));
}

#[test]
fn ok_when_int_param_receives_int_value() {
    let defs = vec![make_template_parameters("PORT", ParameterType::Int)];
    let validator = InstanceParametersValidator::new(&defs);
    let result = validator.validate(&HashMap::from([make_instance_parameters(
        "PORT",
        json!(3306),
    )]));
    assert_eq!(result.len(), 0);
}

#[test]
fn err_when_int_param_receives_string_value() {
    let defs = vec![make_template_parameters("PORT", ParameterType::Int)];
    let validator = InstanceParametersValidator::new(&defs);
    let result = validator.validate(&HashMap::from([make_instance_parameters(
        "PORT",
        json!("not_a_number"),
    )]));
    assert_eq!(result.len(), 1);
    assert!(result[0].contains("PORT"));
    assert!(result[0].contains("Type mismatch"));
}

#[test]
fn ok_when_boolean_param_receives_bool_value() {
    let defs = vec![make_template_parameters("ENABLED", ParameterType::Boolean)];
    let validator = InstanceParametersValidator::new(&defs);
    let result = validator.validate(&HashMap::from([make_instance_parameters(
        "ENABLED",
        json!(true),
    )]));
    assert_eq!(result.len(), 0);
}

#[test]
fn err_when_boolean_param_receives_string_value() {
    // The string "true" does not satisfy ParameterType::Boolean.
    let defs = vec![make_template_parameters("ENABLED", ParameterType::Boolean)];
    let validator = InstanceParametersValidator::new(&defs);
    let result = validator.validate(&HashMap::from([make_instance_parameters(
        "ENABLED",
        json!("true"),
    )]));
    assert_eq!(result.len(), 1);
    assert!(result[0].contains("ENABLED"));
    assert!(result[0].contains("Type mismatch"));
}

#[test]
fn ok_when_vec_string_param_receives_string_array() {
    let defs = vec![make_template_parameters("TAGS", ParameterType::VecString)];
    let validator = InstanceParametersValidator::new(&defs);
    let result = validator.validate(&HashMap::from([make_instance_parameters(
        "TAGS",
        json!(["prod", "us-east"]),
    )]));
    assert_eq!(result.len(), 0);
}

#[test]
fn err_when_vec_string_param_receives_mixed_array() {
    // Arrays containing non-string elements are rejected.
    let defs = vec![make_template_parameters("TAGS", ParameterType::VecString)];
    let validator = InstanceParametersValidator::new(&defs);
    let result = validator.validate(&HashMap::from([make_instance_parameters(
        "TAGS",
        json!(["prod", 42]),
    )]));
    assert_eq!(result.len(), 1);
    assert!(result[0].contains("TAGS"));
    assert!(result[0].contains("Type mismatch"));
}

#[test]
fn ok_when_map_param_receives_string_string_object() {
    let defs = vec![make_template_parameters(
        "ENV",
        ParameterType::MapStringString,
    )];
    let validator = InstanceParametersValidator::new(&defs);
    let result = validator.validate(&HashMap::from([make_instance_parameters(
        "ENV",
        json!({"KEY": "value", "REGION": "eu-west"}),
    )]));
    assert_eq!(result.len(), 0);
}

#[test]
fn err_when_map_param_receives_non_string_value() {
    // Objects whose values are not strings are rejected.
    let defs = vec![make_template_parameters(
        "ENV",
        ParameterType::MapStringString,
    )];
    let validator = InstanceParametersValidator::new(&defs);
    let result = validator.validate(&HashMap::from([make_instance_parameters(
        "ENV",
        json!({"KEY": "value", "PORT": 8080}),
    )]));
    assert_eq!(result.len(), 1);
    assert!(result[0].contains("ENV"));
    assert!(result[0].contains("Type mismatch"));
}

// =========================================================================
// Unknown parameters
// =========================================================================

#[test]
fn err_when_instance_provides_unknown_key() {
    let defs = vec![make_template_parameters("HOST", ParameterType::String)];
    let validator = InstanceParametersValidator::new(&defs);
    let result = validator.validate(&HashMap::from([
        make_instance_parameters("HOST", json!("localhost")),
        make_instance_parameters("GHOST", json!("unknown")),
    ]));
    assert_eq!(result.len(), 1);
    assert!(result[0].contains("GHOST"));
    assert!(result[0].contains("Unknown parameter"));
}

#[test]
fn err_lists_all_unknown_keys() {
    let defs = vec![make_template_parameters("HOST", ParameterType::String)];
    let validator = InstanceParametersValidator::new(&defs);
    let result = validator.validate(&HashMap::from([
        make_instance_parameters("HOST", json!("localhost")),
        make_instance_parameters("FOO", json!("x")),
        make_instance_parameters("BAR", json!("y")),
    ]));
    assert_eq!(result.len(), 2);
}

// =========================================================================
// Error accumulation
// =========================================================================

#[test]
fn ok_when_all_optional_params_are_absent() {
    let defs = vec![
        make_optional_param("TIMEOUT", ParameterType::Int),
        make_optional_param("RETRIES", ParameterType::Int),
    ];
    let validator = InstanceParametersValidator::new(&defs);
    let result = validator.validate(&HashMap::new());
    assert_eq!(result.len(), 0);
}

#[test]
fn err_accumulates_type_mismatch_unknown_and_missing_together() {
    // HOST wrong type, PORT missing, GHOST unknown — all three reported at once.
    let defs = vec![
        make_template_parameters("HOST", ParameterType::String),
        make_template_parameters("PORT", ParameterType::Int),
    ];
    let validator = InstanceParametersValidator::new(&defs);
    let result = validator.validate(&HashMap::from([
        make_instance_parameters("HOST", json!(999)),
        make_instance_parameters("GHOST", json!("unknown")),
    ]));
    assert_eq!(result.len(), 3);
}
