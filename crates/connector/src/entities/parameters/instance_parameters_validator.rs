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
