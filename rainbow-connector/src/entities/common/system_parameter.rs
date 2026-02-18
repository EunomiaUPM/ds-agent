use crate::entities::common::parameters::{ParameterAutoFilledType, ParameterDefinition};
use crate::entities::common::system_context::SystemContext;
use chrono::Utc;
use serde_json::{json, Value};
use std::collections::HashMap;
use urn::UrnBuilder;

pub struct SystemParameterInjector;

impl SystemParameterInjector {
    pub fn inject(
        definitions: &[ParameterDefinition],
        values: &mut HashMap<String, Value>,
        _context: &SystemContext,
    ) {
        for def in definitions {
            // only auto_fillables
            if !def.auto_fillable.auto_filled {
                continue;
            }

            let filled_type = match &def.auto_fillable.auto_filled_type {
                Some(t) => t,
                None => continue, // if no type, ignore
            };

            let value_to_inject = match filled_type {
                ParameterAutoFilledType::SysUrn => {
                    json!(UrnBuilder::new("uuid", &*uuid::Uuid::new_v4().to_string())
                        .build()
                        .unwrap()
                        .to_string())
                }
                ParameterAutoFilledType::SysToken => json!(uuid::Uuid::new_v4().to_string()),
                ParameterAutoFilledType::SysTimestamp => json!(Utc::now().timestamp()),
                ParameterAutoFilledType::SysIso8601 => json!(Utc::now().to_rfc3339()),
                // SysOwnUrl is a runtime-only param resolved at perform_subscribe time;
                // skip injection here so the {{__SYS_OWN_URL__}} template stays
                // unresolved in the stored interaction spec.
                ParameterAutoFilledType::SysOwnUrl { .. } => continue,
            };

            // Inject value in parameter definition
            values.insert(def.name.clone(), value_to_inject);
        }
    }
}
