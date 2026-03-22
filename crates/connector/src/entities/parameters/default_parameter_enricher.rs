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

//! Injects declared default values for parameters the user left unset.
//!
//! This is phase 1b of the parameter enrichment pipeline — it runs after
//! [`SysParameterEnricher`] and before [`TemplateParametersResolver`].
//!
//! [`SysParameterEnricher`]: super::sys_parameter_enricher::SysParameterEnricher
//! [`TemplateParametersResolver`]: super::template_parameters_resolver::TemplateParametersResolver

use crate::entities::parameters::default_parameter_injector::DefaultParametersInjector;
use crate::entities::parameters::parameters::ParameterDefinition;
use crate::entities::parameters::ParameterEnricher;
use serde_json::Value;
use std::collections::HashMap;
use ymir::errors::Outcome;

/// Enriches the parameter map with declared default values.
///
/// For each [`ParameterDefinition`] that carries a `default_value`, the
/// enricher inserts that value only when the key is **absent** from the
/// parameter map — it never overwrites a value supplied by the user or by an
/// earlier pipeline step (e.g. [`SysParameterEnricher`]).
///
/// [`SysParameterEnricher`]: super::sys_parameter_enricher::SysParameterEnricher
///
/// # Example
///
/// ```ignore
/// SysParameterEnricher::new(&template_spec, &own_url).enrich(&mut params)?;
/// DefaultParameterEnricher::new(&template_spec.parameters).enrich(&mut params)?;
/// ```
pub struct DefaultParameterEnricher<'a> {
    definitions: &'a [ParameterDefinition],
}

impl<'a> DefaultParameterEnricher<'a> {
    pub fn new(definitions: &'a [ParameterDefinition]) -> Self {
        Self { definitions }
    }
}

impl ParameterEnricher for DefaultParameterEnricher<'_> {
    /// Fills in the declared `default_value` for every parameter that has one
    /// and whose key is not yet present in `params`.
    fn enrich(&self, params: &mut HashMap<String, Value>) -> Outcome<()> {
        DefaultParametersInjector::inject(self.definitions, params)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod test {
    use super::*;
    use crate::entities::parameters::parameters::ParameterType;
    use serde_json::json;

    fn def(name: &str, param_type: ParameterType, default: Option<&str>) -> ParameterDefinition {
        ParameterDefinition {
            name: name.to_string(),
            title: name.to_string(),
            description: None,
            param_type,
            required: false,
            default_value: default.map(str::to_string),
        }
    }

    #[test]
    fn enrich_inserts_string_default() {
        let defs = vec![def("REGION", ParameterType::String, Some("us-east-1"))];
        let mut params = HashMap::new();

        DefaultParameterEnricher::new(&defs)
            .enrich(&mut params)
            .unwrap();

        assert_eq!(params["REGION"], json!("us-east-1"));
    }

    #[test]
    fn enrich_inserts_int_default() {
        let defs = vec![def("TIMEOUT", ParameterType::Int, Some("30"))];
        let mut params = HashMap::new();

        DefaultParameterEnricher::new(&defs)
            .enrich(&mut params)
            .unwrap();

        assert_eq!(params["TIMEOUT"], json!(30i64));
    }

    #[test]
    fn enrich_inserts_boolean_default() {
        let defs = vec![def("ENABLED", ParameterType::Boolean, Some("true"))];
        let mut params = HashMap::new();

        DefaultParameterEnricher::new(&defs)
            .enrich(&mut params)
            .unwrap();

        assert_eq!(params["ENABLED"], json!(true));
    }

    #[test]
    fn enrich_inserts_vec_string_default() {
        let defs = vec![def(
            "TAGS",
            ParameterType::VecString,
            Some(r#"["prod","eu"]"#),
        )];
        let mut params = HashMap::new();

        DefaultParameterEnricher::new(&defs)
            .enrich(&mut params)
            .unwrap();

        assert_eq!(params["TAGS"], json!(["prod", "eu"]));
    }

    #[test]
    fn enrich_inserts_map_default() {
        let defs = vec![def(
            "ENV",
            ParameterType::MapStringString,
            Some(r#"{"KEY":"val"}"#),
        )];
        let mut params = HashMap::new();

        DefaultParameterEnricher::new(&defs)
            .enrich(&mut params)
            .unwrap();

        assert_eq!(params["ENV"], json!({"KEY": "val"}));
    }

    #[test]
    fn enrich_does_not_overwrite_existing_value() {
        let defs = vec![def("REGION", ParameterType::String, Some("us-east-1"))];
        let mut params = HashMap::from([("REGION".to_string(), json!("eu-west-1"))]);

        DefaultParameterEnricher::new(&defs)
            .enrich(&mut params)
            .unwrap();

        assert_eq!(
            params["REGION"],
            json!("eu-west-1"),
            "user value must not be overwritten"
        );
    }

    #[test]
    fn enrich_skips_param_without_default() {
        let defs = vec![def("HOST", ParameterType::String, None)];
        let mut params = HashMap::new();

        DefaultParameterEnricher::new(&defs)
            .enrich(&mut params)
            .unwrap();

        assert!(
            !params.contains_key("HOST"),
            "absent default must not inject anything"
        );
    }

    #[test]
    fn enrich_returns_err_on_malformed_int_default() {
        let defs = vec![def("PORT", ParameterType::Int, Some("not_a_number"))];
        let mut params = HashMap::new();

        let result = DefaultParameterEnricher::new(&defs).enrich(&mut params);

        assert!(result.is_err(), "malformed default should return an error");
    }

    #[test]
    fn enrich_multiple_defaults_at_once() {
        let defs = vec![
            def("REGION", ParameterType::String, Some("us-east-1")),
            def("TIMEOUT", ParameterType::Int, Some("60")),
        ];
        let mut params = HashMap::new();

        DefaultParameterEnricher::new(&defs)
            .enrich(&mut params)
            .unwrap();

        assert_eq!(params["REGION"], json!("us-east-1"));
        assert_eq!(params["TIMEOUT"], json!(60i64));
    }
}
