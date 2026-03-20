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


use crate::entities::parameters::parameters::{ParameterDefinition, ParameterType};
use crate::entities::parameters::template_parameters_extractor::FoundParameter;
use crate::entities::parameters::template_parameters_validator::ParameterValidator;
use crate::entities::parameters::FoundParameterType;

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

fn make_found(name: &str, content_type: FoundParameterType) -> FoundParameter {
    FoundParameter { name: name.to_string(), content_type }
}

// =========================================================================
// Name matching
// =========================================================================

#[test]
fn ok_when_found_and_defined_match_exactly() {
    let defs = vec![make_def("HOST", ParameterType::String), make_def("PORT", ParameterType::Int)];
    let found = vec![
        make_found("HOST", FoundParameterType::String),
        make_found("PORT", FoundParameterType::Int),
    ];
    assert!(ParameterValidator::new(&defs, false).validate(&found).is_ok());
}

#[test]
fn ok_when_both_empty() {
    let defs: Vec<ParameterDefinition> = vec![];
    let found: Vec<FoundParameter> = vec![];
    assert!(ParameterValidator::new(&defs, false).validate(&found).is_ok());
}

#[test]
fn ok_when_duplicate_found_entries_deduplicated() {
    let defs = vec![make_def("HOST", ParameterType::String)];
    let found = vec![
        make_found("HOST", FoundParameterType::String),
        make_found("HOST", FoundParameterType::String),
    ];
    assert!(ParameterValidator::new(&defs, false).validate(&found).is_ok());
}

#[test]
fn err_when_found_parameter_is_not_declared() {
    let defs = vec![make_def("HOST", ParameterType::String)];
    let found = vec![
        make_found("HOST", FoundParameterType::String),
        make_found("UNKNOWN", FoundParameterType::String),
    ];
    let err = ParameterValidator::new(&defs, false).validate(&found).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Undeclared"), "expected undeclared error, got: {msg}");
    assert!(msg.contains("UNKNOWN"));
}

#[test]
fn err_when_declared_parameter_is_not_used_in_template() {
    let defs =
        vec![make_def("HOST", ParameterType::String), make_def("UNUSED", ParameterType::String)];
    let found = vec![make_found("HOST", FoundParameterType::String)];
    let err = ParameterValidator::new(&defs, false).validate(&found).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("Declared parameters not found"),
        "expected unused error, got: {msg}"
    );
    assert!(msg.contains("UNUSED"));
}

#[test]
fn err_contains_both_issues_when_undeclared_and_unused_exist() {
    let defs =
        vec![make_def("HOST", ParameterType::String), make_def("UNUSED", ParameterType::String)];
    let found = vec![
        make_found("HOST", FoundParameterType::String),
        make_found("GHOST", FoundParameterType::String),
    ];
    let err = ParameterValidator::new(&defs, false).validate(&found).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("GHOST"), "expected GHOST in error: {msg}");
    assert!(msg.contains("UNUSED"), "expected UNUSED in error: {msg}");
}

#[test]
fn err_lists_multiple_undeclared_parameters_sorted() {
    let defs: Vec<ParameterDefinition> = vec![];
    let found = vec![
        make_found("ZEBRA", FoundParameterType::String),
        make_found("ALPHA", FoundParameterType::String),
    ];
    let err = ParameterValidator::new(&defs, false).validate(&found).unwrap_err();
    let msg = err.to_string();
    let alpha_pos = msg.find("ALPHA").unwrap();
    let zebra_pos = msg.find("ZEBRA").unwrap();
    assert!(alpha_pos < zebra_pos);
}

// =========================================================================
// exclude_runtime = true
// =========================================================================

#[test]
fn ok_when_runtime_parameter_is_excluded() {
    let defs = vec![make_def("HOST", ParameterType::String)];
    let found = vec![
        make_found("HOST", FoundParameterType::String),
        make_found("RUNTIME_TIMESTAMP", FoundParameterType::String),
    ];
    assert!(ParameterValidator::new(&defs, true).validate(&found).is_ok());
}

#[test]
fn ok_when_only_runtime_parameters_found_and_no_definitions() {
    let defs: Vec<ParameterDefinition> = vec![];
    let found = vec![
        make_found("RUNTIME_URN", FoundParameterType::String),
        make_found("RUNTIME_TOKEN", FoundParameterType::String),
    ];
    assert!(ParameterValidator::new(&defs, true).validate(&found).is_ok());
}

#[test]
fn err_when_non_runtime_undeclared_alongside_runtime_excluded() {
    let defs = vec![make_def("HOST", ParameterType::String)];
    let found = vec![
        make_found("HOST", FoundParameterType::String),
        make_found("RUNTIME_TIMESTAMP", FoundParameterType::String),
        make_found("GHOST", FoundParameterType::String),
    ];
    let err = ParameterValidator::new(&defs, true).validate(&found).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("GHOST"), "expected GHOST in error: {msg}");
    assert!(
        !msg.contains("RUNTIME_TIMESTAMP"),
        "RUNTIME_ should be excluded: {msg}"
    );
}

#[test]
fn runtime_parameter_causes_error_when_exclude_runtime_is_false() {
    let defs: Vec<ParameterDefinition> = vec![];
    let found = vec![make_found("RUNTIME_TIMESTAMP", FoundParameterType::String)];
    let err = ParameterValidator::new(&defs, false).validate(&found).unwrap_err();
    assert!(err.to_string().contains("RUNTIME_TIMESTAMP"));
}

// =========================================================================
// Type compatibility
// =========================================================================

#[test]
fn ok_when_int_parameter_used_as_complete_int_template() {
    let defs = vec![make_def("PORT", ParameterType::Int)];
    let found = vec![make_found("PORT", FoundParameterType::Int)];
    assert!(ParameterValidator::new(&defs, false).validate(&found).is_ok());
}

#[test]
fn ok_when_int_parameter_interpolated_in_string() {
    // "timeout is {{__TIMEOUT__}} seconds" → FoundParameterType::String
    // compatible with ParameterType::Int (scalar coercion)
    let defs = vec![make_def("TIMEOUT", ParameterType::Int)];
    let found = vec![make_found("TIMEOUT", FoundParameterType::String)];
    assert!(ParameterValidator::new(&defs, false).validate(&found).is_ok());
}

#[test]
fn ok_when_boolean_parameter_interpolated_in_string() {
    let defs = vec![make_def("ENABLED", ParameterType::Boolean)];
    let found = vec![make_found("ENABLED", FoundParameterType::String)];
    assert!(ParameterValidator::new(&defs, false).validate(&found).is_ok());
}

#[test]
fn ok_when_vec_string_parameter_used_as_complete_vec_template() {
    let defs = vec![make_def("TAGS", ParameterType::VecString)];
    let found = vec![make_found("TAGS", FoundParameterType::VecString)];
    assert!(ParameterValidator::new(&defs, false).validate(&found).is_ok());
}

#[test]
fn ok_when_map_string_parameter_used_as_complete_map_template() {
    let defs = vec![make_def("HEADERS", ParameterType::MapStringString)];
    let found = vec![make_found("HEADERS", FoundParameterType::MapString)];
    assert!(ParameterValidator::new(&defs, false).validate(&found).is_ok());
}

#[test]
fn err_when_string_parameter_used_as_int_template() {
    // A String parameter cannot satisfy a TemplateInt::Template context.
    let defs = vec![make_def("HOST", ParameterType::String)];
    let found = vec![make_found("HOST", FoundParameterType::Int)];
    let err = ParameterValidator::new(&defs, false).validate(&found).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Type mismatches"), "expected type error, got: {msg}");
    assert!(msg.contains("HOST"));
}

#[test]
fn err_when_vec_string_parameter_interpolated_in_string() {
    // Vec<String> cannot be coerced into a string interpolation context.
    let defs = vec![make_def("TAGS", ParameterType::VecString)];
    let found = vec![make_found("TAGS", FoundParameterType::String)];
    let err = ParameterValidator::new(&defs, false).validate(&found).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Type mismatches"), "expected type error, got: {msg}");
    assert!(msg.contains("TAGS"));
}

#[test]
fn err_when_map_string_parameter_interpolated_in_string() {
    let defs = vec![make_def("HEADERS", ParameterType::MapStringString)];
    let found = vec![make_found("HEADERS", FoundParameterType::String)];
    let err = ParameterValidator::new(&defs, false).validate(&found).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Type mismatches"), "expected type error, got: {msg}");
    assert!(msg.contains("HEADERS"));
}

#[test]
fn err_deduplicates_repeated_type_mismatch_for_same_parameter() {
    // HOST appears twice with the same incompatible type; error should appear once.
    let defs = vec![make_def("HOST", ParameterType::String)];
    let found = vec![
        make_found("HOST", FoundParameterType::Int),
        make_found("HOST", FoundParameterType::Int),
    ];
    let err = ParameterValidator::new(&defs, false).validate(&found).unwrap_err();
    let msg = err.to_string();
    assert_eq!(
        msg.matches("HOST").count(),
        1,
        "duplicate type error should be deduplicated"
    );
}

// =========================================================================
// Duplicate definitions
// =========================================================================

#[test]
fn err_when_two_definitions_share_the_same_name() {
    let defs = vec![make_def("HOST", ParameterType::String), make_def("HOST", ParameterType::Int)];
    let found = vec![make_found("HOST", FoundParameterType::String)];
    let err = ParameterValidator::new(&defs, false).validate(&found).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("Duplicate parameter names"),
        "expected dup-name error, got: {msg}"
    );
    assert!(msg.contains("HOST"));
}

#[test]
fn err_when_two_definitions_share_the_same_title() {
    let mut def_a = make_def("HOST", ParameterType::String);
    def_a.title = "Hostname".to_string();
    let mut def_b = make_def("PORT", ParameterType::Int);
    def_b.title = "Hostname".to_string();

    let found = vec![
        make_found("HOST", FoundParameterType::String),
        make_found("PORT", FoundParameterType::Int),
    ];
    let err = ParameterValidator::new(&[def_a, def_b], false).validate(&found).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("Duplicate parameter titles"),
        "expected dup-title error, got: {msg}"
    );
    assert!(msg.contains("Hostname"));
}

#[test]
fn err_reports_name_and_type_issues_together() {
    let defs = vec![make_def("HOST", ParameterType::String)];
    let found = vec![
        make_found("HOST", FoundParameterType::Int), // type mismatch
        make_found("GHOST", FoundParameterType::String), // undeclared
    ];
    let err = ParameterValidator::new(&defs, false).validate(&found).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("GHOST"), "expected undeclared GHOST: {msg}");
    assert!(msg.contains("Type mismatches"), "expected type mismatch: {msg}");
    assert!(msg.contains("HOST"), "expected HOST in type error: {msg}");
}
