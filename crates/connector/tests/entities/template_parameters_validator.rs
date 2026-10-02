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

//! TemplateParametersValidator: placeholders found in a template checked against its
//! declared parameters for names, titles and types.

use connector::entities::parameters::FoundParameter;
use connector::entities::parameters::{FoundParameterType, ParameterDefinition, ParameterType};
use ymir::errors::Outcome;

use connector::entities::parameters::template_parameters_validator::TemplateParametersValidator;

fn validate(
    found: &[FoundParameter],
    defs: &[ParameterDefinition],
    exclude_runtime: bool,
    exclude_sys: bool,
) -> Outcome<()> {
    let mut v = TemplateParametersValidator::new(found, defs);
    if exclude_runtime {
        v = v.excluding_runtime_parameters();
    }
    if exclude_sys {
        v = v.excluding_sys_parameters();
    }
    v.validate()
}

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
    FoundParameter {
        name: name.to_string(),
        content_type,
    }
}

/// Found and declared parameters that match pass.
#[test]
fn ok_when_found_and_defined_match_exactly() {
    let defs = vec![
        make_def("HOST", ParameterType::String),
        make_def("PORT", ParameterType::Int),
    ];
    let found = vec![
        make_found("HOST", FoundParameterType::String),
        make_found("PORT", FoundParameterType::String),
    ];
    assert!(validate(&found, &defs, false, false).is_ok());
}

/// A template without placeholders or declarations passes.
#[test]
fn ok_when_both_empty() {
    let defs: Vec<ParameterDefinition> = vec![];
    let found: Vec<FoundParameter> = vec![];
    assert!(validate(&found, &defs, false, false).is_ok());
}

/// A placeholder used twice counts once.
#[test]
fn ok_when_duplicate_found_entries_deduplicated() {
    let defs = vec![make_def("HOST", ParameterType::String)];
    let found = vec![
        make_found("HOST", FoundParameterType::String),
        make_found("HOST", FoundParameterType::String),
    ];
    assert!(validate(&found, &defs, false, false).is_ok());
}

/// A placeholder without declaration is reported as undeclared.
#[test]
fn err_when_found_parameter_is_not_declared() {
    let defs = vec![make_def("HOST", ParameterType::String)];
    let found = vec![
        make_found("HOST", FoundParameterType::String),
        make_found("UNKNOWN", FoundParameterType::String),
    ];
    let msg = format!("{:?}", validate(&found, &defs, false, false).unwrap_err());
    assert!(
        msg.contains("Undeclared"),
        "expected undeclared error, got: {msg}"
    );
    assert!(msg.contains("UNKNOWN"));
}

/// A declaration the template never uses is reported.
#[test]
fn err_when_declared_parameter_is_not_used_in_template() {
    let defs = vec![
        make_def("HOST", ParameterType::String),
        make_def("UNUSED", ParameterType::String),
    ];
    let found = vec![make_found("HOST", FoundParameterType::String)];
    let msg = format!("{:?}", validate(&found, &defs, false, false).unwrap_err());
    assert!(
        msg.contains("Declared parameters not found"),
        "expected unused error, got: {msg}"
    );
    assert!(msg.contains("UNUSED"));
}

/// Undeclared and unused parameters are reported together.
#[test]
fn err_contains_both_issues_when_undeclared_and_unused_exist() {
    let defs = vec![
        make_def("HOST", ParameterType::String),
        make_def("UNUSED", ParameterType::String),
    ];
    let found = vec![
        make_found("HOST", FoundParameterType::String),
        make_found("GHOST", FoundParameterType::String),
    ];
    let msg = format!("{:?}", validate(&found, &defs, false, false).unwrap_err());
    assert!(msg.contains("GHOST"), "expected GHOST in error: {msg}");
    assert!(msg.contains("UNUSED"), "expected UNUSED in error: {msg}");
}

/// Undeclared names are listed in alphabetical order.
#[test]
fn err_lists_multiple_undeclared_parameters_sorted() {
    let defs: Vec<ParameterDefinition> = vec![];
    let found = vec![
        make_found("ZEBRA", FoundParameterType::String),
        make_found("ALPHA", FoundParameterType::String),
    ];
    let msg = format!("{:?}", validate(&found, &defs, false, false).unwrap_err());
    assert!(msg.find("ALPHA").unwrap() < msg.find("ZEBRA").unwrap());
}

/// With runtime parameters excluded, a `RUNTIME_` placeholder needs no declaration.
#[test]
fn ok_when_runtime_parameter_is_excluded() {
    let defs = vec![make_def("HOST", ParameterType::String)];
    let found = vec![
        make_found("HOST", FoundParameterType::String),
        make_found("RUNTIME_TIMESTAMP", FoundParameterType::String),
    ];
    assert!(validate(&found, &defs, true, false).is_ok());
}

/// Only `RUNTIME_` placeholders and no declarations pass when excluded.
#[test]
fn ok_when_only_runtime_parameters_found_and_no_definitions() {
    let defs: Vec<ParameterDefinition> = vec![];
    let found = vec![
        make_found("RUNTIME_URN", FoundParameterType::String),
        make_found("RUNTIME_TOKEN", FoundParameterType::String),
    ];
    assert!(validate(&found, &defs, true, false).is_ok());
}

/// Excluding runtime parameters still reports other undeclared ones, and only those.
#[test]
fn err_when_non_runtime_undeclared_alongside_runtime_excluded() {
    let defs = vec![make_def("HOST", ParameterType::String)];
    let found = vec![
        make_found("HOST", FoundParameterType::String),
        make_found("RUNTIME_TIMESTAMP", FoundParameterType::String),
        make_found("GHOST", FoundParameterType::String),
    ];
    let msg = format!("{:?}", validate(&found, &defs, true, false).unwrap_err());
    assert!(msg.contains("GHOST"), "expected GHOST in error: {msg}");
    assert!(
        !msg.contains("RUNTIME_TIMESTAMP"),
        "RUNTIME_ should be excluded: {msg}"
    );
}

/// Without the exclusion, a `RUNTIME_` placeholder must be declared.
#[test]
fn runtime_parameter_causes_error_when_exclude_runtime_is_false() {
    let defs: Vec<ParameterDefinition> = vec![];
    let found = vec![make_found("RUNTIME_TIMESTAMP", FoundParameterType::String)];
    let msg = format!("{:?}", validate(&found, &defs, false, false).unwrap_err());
    assert!(msg.contains("RUNTIME_TIMESTAMP"));
}

/// An Int parameter may fill a whole string field.
#[test]
fn ok_when_int_parameter_used_as_complete_int_template() {
    let defs = vec![make_def("PORT", ParameterType::Int)];
    let found = vec![make_found("PORT", FoundParameterType::String)];
    assert!(validate(&found, &defs, false, false).is_ok());
}

/// An Int parameter may be interpolated in a string.
#[test]
fn ok_when_int_parameter_interpolated_in_string() {
    let defs = vec![make_def("TIMEOUT", ParameterType::Int)];
    let found = vec![make_found("TIMEOUT", FoundParameterType::String)];
    assert!(validate(&found, &defs, false, false).is_ok());
}

/// A Boolean parameter may be interpolated in a string.
#[test]
fn ok_when_boolean_parameter_interpolated_in_string() {
    let defs = vec![make_def("ENABLED", ParameterType::Boolean)];
    let found = vec![make_found("ENABLED", FoundParameterType::String)];
    assert!(validate(&found, &defs, false, false).is_ok());
}

/// A VecString parameter may fill a string list field.
#[test]
fn ok_when_vec_string_parameter_used_as_complete_vec_template() {
    let defs = vec![make_def("TAGS", ParameterType::VecString)];
    let found = vec![make_found("TAGS", FoundParameterType::VecString)];
    assert!(validate(&found, &defs, false, false).is_ok());
}

/// A MapStringString parameter may fill a string map field.
#[test]
fn ok_when_map_string_parameter_used_as_complete_map_template() {
    let defs = vec![make_def("HEADERS", ParameterType::MapStringString)];
    let found = vec![make_found("HEADERS", FoundParameterType::MapString)];
    assert!(validate(&found, &defs, false, false).is_ok());
}

/// A String parameter cannot be resolved into a VecString field.
#[test]
fn err_when_string_parameter_used_in_vec_context() {
    let defs = vec![make_def("HOST", ParameterType::String)];
    let found = vec![make_found("HOST", FoundParameterType::VecString)];
    let msg = format!("{:?}", validate(&found, &defs, false, false).unwrap_err());
    assert!(
        msg.contains("Type mismatches"),
        "expected type error, got: {msg}"
    );
    assert!(msg.contains("HOST"));
}

/// Vec<String> cannot be coerced into a string interpolation context.
#[test]
fn err_when_vec_string_parameter_interpolated_in_string() {
    let defs = vec![make_def("TAGS", ParameterType::VecString)];
    let found = vec![make_found("TAGS", FoundParameterType::String)];
    let msg = format!("{:?}", validate(&found, &defs, false, false).unwrap_err());
    assert!(
        msg.contains("Type mismatches"),
        "expected type error, got: {msg}"
    );
    assert!(msg.contains("TAGS"));
}

/// A MapStringString parameter cannot be interpolated in a string.
#[test]
fn err_when_map_string_parameter_interpolated_in_string() {
    let defs = vec![make_def("HEADERS", ParameterType::MapStringString)];
    let found = vec![make_found("HEADERS", FoundParameterType::String)];
    let msg = format!("{:?}", validate(&found, &defs, false, false).unwrap_err());
    assert!(
        msg.contains("Type mismatches"),
        "expected type error, got: {msg}"
    );
    assert!(msg.contains("HEADERS"));
}

/// HOST appears twice with the same incompatible type; error should appear once.
#[test]
fn err_deduplicates_repeated_type_mismatch_for_same_parameter() {
    let defs = vec![make_def("HOST", ParameterType::String)];
    let found = vec![
        make_found("HOST", FoundParameterType::VecString),
        make_found("HOST", FoundParameterType::VecString),
    ];
    let msg = format!("{:#}", validate(&found, &defs, false, false).unwrap_err());
    // "'HOST': used as" appears once means the type error was deduplicated
    assert_eq!(
        msg.matches("used as VecString").count(),
        1,
        "duplicate type error should be deduplicated"
    );
}

/// Two declarations with the same name are rejected.
#[test]
fn err_when_two_definitions_share_the_same_name() {
    let defs = vec![
        make_def("HOST", ParameterType::String),
        make_def("HOST", ParameterType::Int),
    ];
    let found = vec![make_found("HOST", FoundParameterType::String)];
    let msg = format!("{:?}", validate(&found, &defs, false, false).unwrap_err());
    assert!(
        msg.contains("Duplicate parameter names"),
        "expected dup-name error, got: {msg}"
    );
    assert!(msg.contains("HOST"));
}

/// Two declarations with the same title are rejected.
#[test]
fn err_when_two_definitions_share_the_same_title() {
    let mut def_a = make_def("HOST", ParameterType::String);
    def_a.title = "Hostname".to_string();
    let mut def_b = make_def("PORT", ParameterType::Int);
    def_b.title = "Hostname".to_string();

    let found = vec![
        make_found("HOST", FoundParameterType::String),
        make_found("PORT", FoundParameterType::String),
    ];
    let msg = format!(
        "{:?}",
        validate(&found, &[def_a, def_b], false, false).unwrap_err()
    );
    assert!(
        msg.contains("Duplicate parameter titles"),
        "expected dup-title error, got: {msg}"
    );
    assert!(msg.contains("Hostname"));
}

/// Name and type problems are reported in one error.
#[test]
fn err_reports_name_and_type_issues_together() {
    let defs = vec![make_def("HOST", ParameterType::String)];
    let found = vec![
        make_found("HOST", FoundParameterType::VecString), // type mismatch
        make_found("GHOST", FoundParameterType::String),   // undeclared
    ];
    let msg = format!("{:?}", validate(&found, &defs, false, false).unwrap_err());
    assert!(msg.contains("GHOST"), "expected undeclared GHOST: {msg}");
    assert!(
        msg.contains("Type mismatches"),
        "expected type mismatch: {msg}"
    );
    assert!(msg.contains("HOST"), "expected HOST in type error: {msg}");
}
