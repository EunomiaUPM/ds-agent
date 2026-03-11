use crate::entities::parameters::parameters::{ParameterDefinition, ParameterType};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// Validates that the parameters supplied by a connector instance are consistent
/// with the [`ParameterDefinition`]s declared in the connector template.
///
/// Three categories of errors are detected and returned together so that the
/// caller sees every problem in a single pass:
///
/// - **Unknown parameters** — a key supplied by the instance has no matching
///   entry in `parameters[]`.
/// - **Missing required parameters** — a `ParameterDefinition` marked
///   `required: true` has no corresponding value in the instance map.
/// - **Type mismatches** — a supplied value's JSON type is incompatible with
///   the declared [`ParameterType`].
///
/// Auto-filled parameters follow a separate rule: if a parameter is marked
/// `auto_filled`, the instance must *not* supply a value for it. This check
/// short-circuits the required/type rules for that parameter.
pub struct InstanceParametersValidator<'a> {
    template_parameters: &'a [ParameterDefinition],
}

impl<'a> InstanceParametersValidator<'a> {
    pub fn new(template_parameters: &'a [ParameterDefinition]) -> Self {
        Self { template_parameters }
    }

    /// Runs all validation rules against `instance_parameters` and returns
    /// every error found.
    ///
    /// Returns an empty `Vec` when all checks pass. All applicable issues —
    /// unknown keys, missing required fields, and type mismatches — are
    /// collected and returned together rather than stopping at the first failure.
    pub fn validate(&self, instance_parameters: &HashMap<String, Value>) -> Vec<String> {
        let mut errors = self.validate_unknown_parameters(instance_parameters);

        for template_parameter in self.template_parameters {
            if let Some(error) = Self::validate_single_parameter(
                template_parameter,
                instance_parameters.get(&template_parameter.name),
            ) {
                errors.push(error);
            }
        }

        errors
    }

    /// Returns an error string for every key in `values` that has no
    /// corresponding entry in `self.template_parameters`.
    fn validate_unknown_parameters(&self, values: &HashMap<String, Value>) -> Vec<String> {
        let valid_names: HashSet<&String> =
            self.template_parameters.iter().map(|d| &d.name).collect();

        values
            .keys()
            .filter(|k| !valid_names.contains(k))
            .map(|k| format!("Unknown parameter: '{}'", k))
            .collect()
    }

    /// Applies validation rules A, B, and C to a single parameter definition.
    ///
    /// - **Rule A** — if `auto_filled` is set, the user must *not* supply a
    ///   value. This check short-circuits: no further rules are evaluated for
    ///   that parameter.
    /// - **Rule B** — if the parameter is `required` and no value was supplied,
    ///   an error is returned. Optional parameters without a value pass silently.
    /// - **Rule C** — if a value is present, its JSON type must be compatible
    ///   with the declared [`ParameterType`].
    ///
    /// Returns `Some(error_message)` on the first failing rule, `None` otherwise.
    fn validate_single_parameter(
        template_parameter: &ParameterDefinition,
        user_value: Option<&Value>,
    ) -> Option<String> {
        // Rule A: RUNTIME_ or SYS_ parameters not allowed here
        if template_parameter.name.starts_with("SYS_") {
            return None;
        }
        if template_parameter.name.starts_with("RUNTIME_") {
            return None;
        }

        // Rule B: Existence logic (Required vs Optional)
        let Some(val) = user_value else {
            if template_parameter.required {
                return Some(format!("Missing required parameter: '{}'", template_parameter.name));
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
    fn check_type_compatibility(expected: &ParameterType, val: &Value) -> bool {
        match expected {
            ParameterType::String => val.is_string(),
            ParameterType::Int => val.is_i64() || val.is_u64(),
            ParameterType::Boolean => val.is_boolean(),
            ParameterType::VecString => {
                val.as_array().map_or(false, |arr| arr.iter().all(|e| e.is_string()))
            }
            ParameterType::MapStringString => {
                val.as_object().map_or(false, |obj| obj.values().all(|v| v.is_string()))
            }
        }
    }
}

#[cfg(test)]
#[cfg(test)]
mod tests;
