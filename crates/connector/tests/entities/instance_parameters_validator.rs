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

//! InstanceParametersValidator: instance parameters checked against the template's
//! definitions for presence, type and unknown keys.

use connector::entities::parameters::{ParameterDefinition, ParameterType};
use std::collections::HashMap;
use ymir::errors::Outcome;

use connector::entities::parameters::instance_parameters_validator::*;
use serde_json::json;

fn make_def(name: &str, param_type: ParameterType) -> ParameterDefinition {
    ParameterDefinition {
        name: name.to_string(),
        title: name.to_string(),
        description: None,
        param_type,
        required: true,
        default_value: None,
    }
}

fn make_optional(name: &str, param_type: ParameterType) -> ParameterDefinition {
    ParameterDefinition {
        required: false,
        ..make_def(name, param_type)
    }
}

fn validate(
    instance: HashMap<String, serde_json::Value>,
    defs: &[ParameterDefinition],
) -> Outcome<()> {
    InstanceParametersValidator::new(&instance, defs).validate()
}

fn err_msg(result: Outcome<()>) -> String {
    format!("{:#}", result.unwrap_err())
}

/// Parameters that match their definitions pass.
#[test]
fn ok_when_all_params_match_definitions() {
    let defs = vec![
        make_def("HOST", ParameterType::String),
        make_def("PORT", ParameterType::Int),
    ];
    assert!(validate(
        HashMap::from([
            ("HOST".to_string(), json!("localhost")),
            ("PORT".to_string(), json!(8080)),
        ]),
        &defs,
    )
    .is_ok());
}

/// A `SYS_` parameter is never required from the caller.
#[test]
fn ok_when_sys_param_is_absent_from_instance() {
    // SYS_ definitions are never required from the caller — system fills them.
    let defs = vec![make_def("SYS_TOKEN", ParameterType::String)];
    assert!(validate(HashMap::new(), &defs).is_ok());
}

/// A `RUNTIME_` parameter is never required from the caller.
#[test]
fn ok_when_runtime_param_is_absent_from_instance() {
    let defs = vec![make_def("RUNTIME_URN", ParameterType::String)];
    assert!(validate(HashMap::new(), &defs).is_ok());
}

/// A `SYS_` parameter given explicitly is accepted without type check.
#[test]
fn ok_when_sys_param_is_present_in_instance() {
    // Providing a SYS_ param explicitly is accepted (no type-check, no rejection).
    let defs = vec![make_def("SYS_TOKEN", ParameterType::String)];
    assert!(validate(
        HashMap::from([("SYS_TOKEN".to_string(), json!("whatever"))]),
        &defs,
    )
    .is_ok());
}

/// A key with no definition is unknown, whatever its prefix.
#[test]
fn err_when_instance_provides_key_not_in_defs() {
    // A key absent from defs is always unknown — regardless of prefix.
    let defs = vec![make_def("HOST", ParameterType::String)];
    let msg = err_msg(validate(
        HashMap::from([
            ("HOST".to_string(), json!("localhost")),
            ("GHOST".to_string(), json!("x")),
        ]),
        &defs,
    ));
    assert!(msg.contains("Unknown parameter"), "got: {msg}");
    assert!(msg.contains("GHOST"), "got: {msg}");
}

/// A missing required parameter is reported by name.
#[test]
fn err_when_required_param_is_missing() {
    let defs = vec![make_def("HOST", ParameterType::String)];
    let msg = err_msg(validate(HashMap::new(), &defs));
    assert!(msg.contains("Missing required"), "got: {msg}");
    assert!(msg.contains("HOST"), "got: {msg}");
}

/// An optional parameter may be left out.
#[test]
fn ok_when_optional_param_is_absent() {
    let defs = vec![make_optional("TIMEOUT", ParameterType::Int)];
    assert!(validate(HashMap::new(), &defs).is_ok());
}

/// A String parameter accepts a string.
#[test]
fn ok_when_string_param_receives_string_value() {
    let defs = vec![make_def("NAME", ParameterType::String)];
    assert!(validate(HashMap::from([("NAME".to_string(), json!("alice"))]), &defs,).is_ok());
}

/// A String parameter rejects a number.
#[test]
fn err_when_string_param_receives_int_value() {
    let defs = vec![make_def("NAME", ParameterType::String)];
    let msg = err_msg(validate(
        HashMap::from([("NAME".to_string(), json!(42))]),
        &defs,
    ));
    assert!(msg.contains("Type mismatch"), "got: {msg}");
    assert!(msg.contains("NAME"), "got: {msg}");
}

/// An Int parameter accepts an integer.
#[test]
fn ok_when_int_param_receives_int_value() {
    let defs = vec![make_def("PORT", ParameterType::Int)];
    assert!(validate(HashMap::from([("PORT".to_string(), json!(3306))]), &defs,).is_ok());
}

/// An Int parameter rejects a string.
#[test]
fn err_when_int_param_receives_string_value() {
    let defs = vec![make_def("PORT", ParameterType::Int)];
    let msg = err_msg(validate(
        HashMap::from([("PORT".to_string(), json!("not_a_number"))]),
        &defs,
    ));
    assert!(msg.contains("Type mismatch"), "got: {msg}");
    assert!(msg.contains("PORT"), "got: {msg}");
}

/// A Boolean parameter accepts a bool.
#[test]
fn ok_when_boolean_param_receives_bool_value() {
    let defs = vec![make_def("ENABLED", ParameterType::Boolean)];
    assert!(validate(HashMap::from([("ENABLED".to_string(), json!(true))]), &defs,).is_ok());
}

/// A Boolean parameter rejects the string "true".
#[test]
fn err_when_boolean_param_receives_string_value() {
    // The string "true" does not satisfy ParameterType::Boolean.
    let defs = vec![make_def("ENABLED", ParameterType::Boolean)];
    let msg = err_msg(validate(
        HashMap::from([("ENABLED".to_string(), json!("true"))]),
        &defs,
    ));
    assert!(msg.contains("Type mismatch"), "got: {msg}");
    assert!(msg.contains("ENABLED"), "got: {msg}");
}

/// A VecString parameter accepts an array of strings.
#[test]
fn ok_when_vec_string_param_receives_string_array() {
    let defs = vec![make_def("TAGS", ParameterType::VecString)];
    assert!(validate(
        HashMap::from([("TAGS".to_string(), json!(["prod", "us-east"]))]),
        &defs,
    )
    .is_ok());
}

/// A VecString parameter rejects an array with a non-string.
#[test]
fn err_when_vec_string_param_receives_mixed_array() {
    let defs = vec![make_def("TAGS", ParameterType::VecString)];
    let msg = err_msg(validate(
        HashMap::from([("TAGS".to_string(), json!(["prod", 42]))]),
        &defs,
    ));
    assert!(msg.contains("Type mismatch"), "got: {msg}");
    assert!(msg.contains("TAGS"), "got: {msg}");
}

/// A MapStringString parameter accepts an object of strings.
#[test]
fn ok_when_map_param_receives_string_string_object() {
    let defs = vec![make_def("ENV", ParameterType::MapStringString)];
    assert!(validate(
        HashMap::from([(
            "ENV".to_string(),
            json!({"KEY": "value", "REGION": "eu-west"})
        )]),
        &defs,
    )
    .is_ok());
}

/// A MapStringString parameter rejects an object with a non-string value.
#[test]
fn err_when_map_param_receives_non_string_value() {
    let defs = vec![make_def("ENV", ParameterType::MapStringString)];
    let msg = err_msg(validate(
        HashMap::from([("ENV".to_string(), json!({"KEY": "value", "PORT": 8080}))]),
        &defs,
    ));
    assert!(msg.contains("Type mismatch"), "got: {msg}");
    assert!(msg.contains("ENV"), "got: {msg}");
}

/// An unknown key is reported by name next to valid ones.
#[test]
fn err_when_instance_provides_unknown_key() {
    let defs = vec![make_def("HOST", ParameterType::String)];
    let msg = err_msg(validate(
        HashMap::from([
            ("HOST".to_string(), json!("localhost")),
            ("GHOST".to_string(), json!("unknown")),
        ]),
        &defs,
    ));
    assert!(msg.contains("Unknown parameter"), "got: {msg}");
    assert!(msg.contains("GHOST"), "got: {msg}");
}

/// Every unknown key gets its own message.
#[test]
fn err_lists_all_unknown_keys() {
    let defs = vec![make_def("HOST", ParameterType::String)];
    let msg = err_msg(validate(
        HashMap::from([
            ("HOST".to_string(), json!("localhost")),
            ("FOO".to_string(), json!("x")),
            ("BAR".to_string(), json!("y")),
        ]),
        &defs,
    ));
    // Two unknown keys - two "Unknown parameter" messages separated by "; "
    assert_eq!(msg.matches("Unknown parameter").count(), 2, "got: {msg}");
}

/// Leaving out every optional parameter passes.
#[test]
fn ok_when_all_optional_params_are_absent() {
    let defs = vec![
        make_optional("TIMEOUT", ParameterType::Int),
        make_optional("RETRIES", ParameterType::Int),
    ];
    assert!(validate(HashMap::new(), &defs).is_ok());
}

/// Type mismatches, missing and unknown keys are all reported at once.
#[test]
fn err_accumulates_type_mismatch_unknown_and_missing_together() {
    // HOST wrong type, PORT missing, GHOST unknown — all three reported at once.
    let defs = vec![
        make_def("HOST", ParameterType::String),
        make_def("PORT", ParameterType::Int),
    ];
    let msg = err_msg(validate(
        HashMap::from([
            ("HOST".to_string(), json!(999)),
            ("GHOST".to_string(), json!("unknown")),
        ]),
        &defs,
    ));
    assert!(msg.contains("Type mismatch"), "got: {msg}");
    assert!(msg.contains("Missing required"), "got: {msg}");
    assert!(msg.contains("Unknown parameter"), "got: {msg}");
}
