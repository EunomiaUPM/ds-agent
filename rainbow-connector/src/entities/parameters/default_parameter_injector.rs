use crate::entities::parameters::parameters::{ParameterDefinition, ParameterType};
use anyhow::anyhow;
use serde_json::{json, Value};
use std::collections::HashMap;
use ymir::errors::{Errors, Outcome};

pub struct DefaultParametersInjector;

impl DefaultParametersInjector {
    pub fn inject(
        definitions: &[ParameterDefinition],
        values: &mut HashMap<String, Value>,
    ) -> Outcome<()> {
        for def in definitions {
            // if value already set, do nothing
            if values.contains_key(&def.name) {
                continue;
            }
            // if default value set
            if let Some(default_str) = &def.default_value {
                let json_value = Self::cast_default_value(&def.name, &def.param_type, default_str)?;
                values.insert(def.name.clone(), json_value);
            }
        }
        Ok(())
    }

    /// convert string to value
    fn cast_default_value(
        param_name: &str,
        param_type: &ParameterType,
        raw_val: &str,
    ) -> Outcome<Value> {
        match param_type {
            ParameterType::String => Ok(json!(raw_val)),
            ParameterType::Int => {
                let parsed = raw_val.parse::<i64>().map_err(|_| {
                    Errors::crazy(format!("Template Definition Error: Default value '{}' for param '{}' is not a valid Integer.", raw_val, param_name), None)
                })?;
                Ok(json!(parsed))
            }
            ParameterType::Boolean => {
                let parsed = raw_val.to_lowercase().parse::<bool>().map_err(|_| {
                    Errors::crazy(format!("Template Definition Error: Default value '{}' for param '{}' is not a valid Boolean.", raw_val, param_name), None)
                })?;
                Ok(json!(parsed))
            }
            ParameterType::VecString | ParameterType::MapStringString => {
                let parsed: Value = serde_json::from_str(raw_val).map_err(|e| {
                    Errors::crazy(format!("Template Definition Error: Default value '{}' for complex param '{}' is not valid JSON: {}", raw_val, param_name, e), None)
                })?;

                if matches!(param_type, ParameterType::VecString) && !parsed.is_array() {
                    return Err(Errors::crazy(
                        format!("Default value for '{}' must be a JSON Array", param_name),
                        None,
                    ));
                }
                if matches!(param_type, ParameterType::MapStringString) && !parsed.is_object() {
                    return Err(Errors::crazy(
                        format!("Default value for '{}' must be a JSON Object", param_name),
                        None,
                    ));
                }

                Ok(parsed)
            }
        }
    }
}
