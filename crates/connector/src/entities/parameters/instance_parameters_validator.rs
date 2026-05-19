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

use super::{ParameterDefinition, ParameterType};
use std::collections::{HashMap, HashSet};
use ymir::errors::{Errors, Outcome};

pub struct InstanceParametersValidator<'a> {
    parameters_in_instance: &'a HashMap<String, serde_json::Value>,
    parameters_in_definition: &'a [ParameterDefinition],
}

impl<'a> InstanceParametersValidator<'a> {
    pub fn new(
        parameters_in_instance: &'a HashMap<String, serde_json::Value>,
        parameters_in_definition: &'a [ParameterDefinition],
    ) -> Self {
        Self {
            parameters_in_instance,
            parameters_in_definition,
        }
    }

    pub fn validate(&self) -> Outcome<()> {
        // if unknown parameter
        let mut errors = self.yield_error_on_unknown_parameters(self.parameters_in_instance);
        // if each single parameter
        for template_parameter in self.parameters_in_definition {
            if let Some(error) = Self::validate_single_parameter(
                template_parameter,
                self.parameters_in_instance.get(&template_parameter.name),
            ) {
                errors.push(error);
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(Errors::validation(&errors.join("; "), None))
        }
    }

    /// Returns an error string for every key in `values` that has no
    /// corresponding entry in `self.template_parameters`.
    fn yield_error_on_unknown_parameters(
        &self,
        values: &HashMap<String, serde_json::Value>,
    ) -> Vec<String> {
        let valid_names: HashSet<&String> = self
            .parameters_in_definition
            .iter()
            .map(|d| &d.name)
            .collect();
        values
            .keys()
            .filter(|k| !valid_names.contains(k))
            .map(|k| format!("Unknown parameter: '{}'", k))
            .collect()
    }

    /// Returns `Some(error_message)` on the first failing rule, `None` otherwise.
    fn validate_single_parameter(
        template_parameter: &ParameterDefinition,
        instance_parameter: Option<&serde_json::Value>,
    ) -> Option<String> {
        // Rule A: RUNTIME_ or SYS_ parameters ignored here
        if template_parameter.name.starts_with("SYS_") {
            return None;
        }
        if template_parameter.name.starts_with("RUNTIME_") {
            return None;
        }

        // Rule B: Existence logic (Required vs Optional)
        let Some(val) = instance_parameter else {
            if template_parameter.required {
                return Some(format!(
                    "Missing required parameter: '{}'",
                    template_parameter.name
                ));
            }
            return None;
        };

        // Rule C: Type Validation (only if value exists)
        if !Self::check_type_compatibility(&template_parameter.param_type, val) {
            return Some(format!(
                "Type mismatch for '{}'. Expected {:?}, got: {}",
                template_parameter.name, template_parameter.param_type, val
            ));
        }

        None
    }

    /// Returns `true` when `val`'s JSON type satisfies the expectation of
    /// `expected`.
    ///
    /// ## Type compatibility table
    ///
    /// | `ParameterType`   | Accepted `serde_json::Value` variants                 |
    /// |-------------------|-------------------------------------------------------|
    /// | `String`          | `Value::String`                                       |
    /// | `Int`             | `Value::Number` (i64 or u64)                          |
    /// | `Boolean`         | `Value::Bool`                                         |
    /// | `VecString`       | `Value::Array` where every element is `Value::String` |
    /// | `MapStringString` | `Value::Object` where every value is `Value::String`  |
    fn check_type_compatibility(
        expected_type: &ParameterType,
        actual_value: &serde_json::Value,
    ) -> bool {
        match expected_type {
            ParameterType::String => actual_value.is_string(),
            ParameterType::Int => actual_value.is_i64() || actual_value.is_u64(),
            ParameterType::Boolean => actual_value.is_boolean(),
            ParameterType::VecString => actual_value
                .as_array()
                .map_or(false, |arr| arr.iter().all(|e| e.is_string())),
            ParameterType::MapStringString => actual_value
                .as_object()
                .map_or(false, |obj| obj.values().all(|v| v.is_string())),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;
    use ymir::errors::Outcome;

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

    // =========================================================================
    // Base
    // =========================================================================

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

    // =========================================================================
    // Rule A: SYS_ and RUNTIME_ definitions are silently skipped
    // =========================================================================

    #[test]
    fn ok_when_sys_param_is_absent_from_instance() {
        // SYS_ definitions are never required from the caller — system fills them.
        let defs = vec![make_def("SYS_TOKEN", ParameterType::String)];
        assert!(validate(HashMap::new(), &defs).is_ok());
    }

    #[test]
    fn ok_when_runtime_param_is_absent_from_instance() {
        let defs = vec![make_def("RUNTIME_URN", ParameterType::String)];
        assert!(validate(HashMap::new(), &defs).is_ok());
    }

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

    // =========================================================================
    // Rule B: Required vs Optional
    // =========================================================================

    #[test]
    fn err_when_required_param_is_missing() {
        let defs = vec![make_def("HOST", ParameterType::String)];
        let msg = err_msg(validate(HashMap::new(), &defs));
        assert!(msg.contains("Missing required"), "got: {msg}");
        assert!(msg.contains("HOST"), "got: {msg}");
    }

    #[test]
    fn ok_when_optional_param_is_absent() {
        let defs = vec![make_optional("TIMEOUT", ParameterType::Int)];
        assert!(validate(HashMap::new(), &defs).is_ok());
    }

    // =========================================================================
    // Rule C: Type validation
    // =========================================================================

    #[test]
    fn ok_when_string_param_receives_string_value() {
        let defs = vec![make_def("NAME", ParameterType::String)];
        assert!(validate(HashMap::from([("NAME".to_string(), json!("alice"))]), &defs,).is_ok());
    }

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

    #[test]
    fn ok_when_int_param_receives_int_value() {
        let defs = vec![make_def("PORT", ParameterType::Int)];
        assert!(validate(HashMap::from([("PORT".to_string(), json!(3306))]), &defs,).is_ok());
    }

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

    #[test]
    fn ok_when_boolean_param_receives_bool_value() {
        let defs = vec![make_def("ENABLED", ParameterType::Boolean)];
        assert!(validate(HashMap::from([("ENABLED".to_string(), json!(true))]), &defs,).is_ok());
    }

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

    #[test]
    fn ok_when_vec_string_param_receives_string_array() {
        let defs = vec![make_def("TAGS", ParameterType::VecString)];
        assert!(validate(
            HashMap::from([("TAGS".to_string(), json!(["prod", "us-east"]))]),
            &defs,
        )
        .is_ok());
    }

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

    // =========================================================================
    // Unknown parameters
    // =========================================================================

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

    // =========================================================================
    // Error accumulation
    // =========================================================================

    #[test]
    fn ok_when_all_optional_params_are_absent() {
        let defs = vec![
            make_optional("TIMEOUT", ParameterType::Int),
            make_optional("RETRIES", ParameterType::Int),
        ];
        assert!(validate(HashMap::new(), &defs).is_ok());
    }

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
}
