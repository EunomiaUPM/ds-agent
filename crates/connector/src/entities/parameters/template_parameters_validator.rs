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

use super::FoundParameter;
use super::{FoundParameterType, ParameterDefinition, ParameterType};
use std::collections::{HashMap, HashSet};
use ymir::errors::{Errors, Outcome};

pub struct TemplateParametersValidator<'a> {
    parameters_found: &'a [FoundParameter],
    parameters_in_definition: &'a [ParameterDefinition],
    exclude_sys_parameters: bool,
    exclude_runtime_parameters: bool,
}

fn is_compatible(found: &FoundParameterType, defined: &ParameterType) -> bool {
    match (found, defined) {
        (FoundParameterType::String, ParameterType::String) => true,
        (FoundParameterType::String, ParameterType::Int) => true,
        (FoundParameterType::String, ParameterType::Boolean) => true,
        (FoundParameterType::VecString, ParameterType::VecString) => true,
        (FoundParameterType::MapString, ParameterType::MapStringString) => true,
        _ => false,
    }
}

impl<'a> TemplateParametersValidator<'a> {
    pub fn new(
        parameters_found: &'a [FoundParameter],
        parameters_in_definition: &'a [ParameterDefinition],
    ) -> Self {
        Self {
            parameters_found,
            parameters_in_definition,
            exclude_sys_parameters: false,
            exclude_runtime_parameters: false,
        }
    }
    pub fn excluding_sys_parameters(mut self) -> Self {
        self.exclude_sys_parameters = true;
        self
    }
    pub fn excluding_runtime_parameters(mut self) -> Self {
        self.exclude_runtime_parameters = true;
        self
    }
    pub fn validate(&self) -> Outcome<()> {
        let mut errors: Vec<String> = Vec::new();
        let found_filtered = self.filter_out_sys_runtime(self.parameters_found);
        let found_names: HashSet<&str> = found_filtered.iter().map(|s| s.name.as_str()).collect();
        let defined_map: HashMap<&str, &ParameterDefinition> = self
            .parameters_in_definition
            .iter()
            .map(|p| (p.name.as_str(), p))
            .collect();
        let defined_names: HashSet<&str> = defined_map.keys().copied().collect();

        errors.extend(self.check_duplicate_definitions());
        errors.extend(self.check_undeclared(&found_names, &defined_names));
        errors.extend(self.check_unused(&found_names, &defined_names));
        errors.extend(self.check_type_compatibility(&found_filtered, &defined_map));

        if errors.is_empty() {
            Ok(())
        } else {
            Err(Errors::validation(&errors.join("; "), None))
        }
    }

    fn filter_out_sys_runtime<'b>(&self, found: &'b [FoundParameter]) -> Vec<&'b FoundParameter> {
        found
            .iter()
            .filter(|s| !self.exclude_runtime_parameters || !s.name.starts_with("RUNTIME_"))
            .filter(|s| !self.exclude_sys_parameters || !s.name.starts_with("SYS_"))
            .collect()
    }

    fn check_duplicate_definitions(&self) -> Vec<String> {
        let mut seen_names: HashSet<&str> = HashSet::new();
        let mut dup_names: Vec<&str> = Vec::new();
        let mut seen_titles: HashSet<&str> = HashSet::new();
        let mut dup_titles: Vec<&str> = Vec::new();

        for p in self.parameters_in_definition {
            if !seen_names.insert(p.name.as_str()) {
                dup_names.push(p.name.as_str());
            }
            if !seen_titles.insert(p.title.as_str()) {
                dup_titles.push(p.title.as_str());
            }
        }

        let mut errors = Vec::new();

        if !dup_names.is_empty() {
            dup_names.sort();
            dup_names.dedup();
            errors.push(format!(
                "Duplicate parameter names in definition: [{}]",
                dup_names.join(", ")
            ));
        }
        if !dup_titles.is_empty() {
            dup_titles.sort();
            dup_titles.dedup();
            errors.push(format!(
                "Duplicate parameter titles in definition: [{}]",
                dup_titles.join(", ")
            ));
        }

        errors
    }

    fn check_undeclared(
        &self,
        found_names: &HashSet<&str>,
        defined_names: &HashSet<&str>,
    ) -> Vec<String> {
        let mut undeclared: Vec<&str> = found_names.difference(defined_names).copied().collect();

        if undeclared.is_empty() {
            return Vec::new();
        }

        undeclared.sort();
        vec![format!(
            "Undeclared parameters found in template: [{}]",
            undeclared.join(", ")
        )]
    }

    fn check_unused(
        &self,
        found_names: &HashSet<&str>,
        defined_names: &HashSet<&str>,
    ) -> Vec<String> {
        let mut unused: Vec<&str> = defined_names.difference(found_names).copied().collect();

        if unused.is_empty() {
            return Vec::new();
        }

        unused.sort();
        vec![format!(
            "Declared parameters not found in template: [{}]",
            unused.join(", ")
        )]
    }

    fn check_type_compatibility(
        &self,
        found_filtered: &[&FoundParameter],
        defined_map: &HashMap<&str, &ParameterDefinition>,
    ) -> Vec<String> {
        let mut type_errors: Vec<String> = found_filtered
            .iter()
            .filter_map(|fp| {
                defined_map.get(fp.name.as_str()).and_then(|def| {
                    if !is_compatible(&fp.content_type, &def.param_type) {
                        Some(format!(
                            "'{}': used as {:?} but declared as {:?}",
                            fp.name, fp.content_type, def.param_type
                        ))
                    } else {
                        None
                    }
                })
            })
            .collect();

        type_errors.sort();
        type_errors.dedup();

        if type_errors.is_empty() {
            Vec::new()
        } else {
            vec![format!("Type mismatches: [{}]", type_errors.join(", "))]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::FoundParameter;
    use super::super::FoundParameterType;
    use super::super::{ParameterDefinition, ParameterType};
    use crate::entities::parameters::template_parameters_validator::TemplateParametersValidator;
    use ymir::errors::Outcome;
    // =========================================================================
    // Helpers
    // =========================================================================

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

    // =========================================================================
    // Name matching
    // =========================================================================

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

    #[test]
    fn ok_when_both_empty() {
        let defs: Vec<ParameterDefinition> = vec![];
        let found: Vec<FoundParameter> = vec![];
        assert!(validate(&found, &defs, false, false).is_ok());
    }

    #[test]
    fn ok_when_duplicate_found_entries_deduplicated() {
        let defs = vec![make_def("HOST", ParameterType::String)];
        let found = vec![
            make_found("HOST", FoundParameterType::String),
            make_found("HOST", FoundParameterType::String),
        ];
        assert!(validate(&found, &defs, false, false).is_ok());
    }

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
        assert!(validate(&found, &defs, true, false).is_ok());
    }

    #[test]
    fn ok_when_only_runtime_parameters_found_and_no_definitions() {
        let defs: Vec<ParameterDefinition> = vec![];
        let found = vec![
            make_found("RUNTIME_URN", FoundParameterType::String),
            make_found("RUNTIME_TOKEN", FoundParameterType::String),
        ];
        assert!(validate(&found, &defs, true, false).is_ok());
    }

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

    #[test]
    fn runtime_parameter_causes_error_when_exclude_runtime_is_false() {
        let defs: Vec<ParameterDefinition> = vec![];
        let found = vec![make_found("RUNTIME_TIMESTAMP", FoundParameterType::String)];
        let msg = format!("{:?}", validate(&found, &defs, false, false).unwrap_err());
        assert!(msg.contains("RUNTIME_TIMESTAMP"));
    }

    // =========================================================================
    // Type compatibility
    // =========================================================================

    #[test]
    fn ok_when_int_parameter_used_as_complete_int_template() {
        let defs = vec![make_def("PORT", ParameterType::Int)];
        let found = vec![make_found("PORT", FoundParameterType::String)];
        assert!(validate(&found, &defs, false, false).is_ok());
    }

    #[test]
    fn ok_when_int_parameter_interpolated_in_string() {
        let defs = vec![make_def("TIMEOUT", ParameterType::Int)];
        let found = vec![make_found("TIMEOUT", FoundParameterType::String)];
        assert!(validate(&found, &defs, false, false).is_ok());
    }

    #[test]
    fn ok_when_boolean_parameter_interpolated_in_string() {
        let defs = vec![make_def("ENABLED", ParameterType::Boolean)];
        let found = vec![make_found("ENABLED", FoundParameterType::String)];
        assert!(validate(&found, &defs, false, false).is_ok());
    }

    #[test]
    fn ok_when_vec_string_parameter_used_as_complete_vec_template() {
        let defs = vec![make_def("TAGS", ParameterType::VecString)];
        let found = vec![make_found("TAGS", FoundParameterType::VecString)];
        assert!(validate(&found, &defs, false, false).is_ok());
    }

    #[test]
    fn ok_when_map_string_parameter_used_as_complete_map_template() {
        let defs = vec![make_def("HEADERS", ParameterType::MapStringString)];
        let found = vec![make_found("HEADERS", FoundParameterType::MapString)];
        assert!(validate(&found, &defs, false, false).is_ok());
    }

    #[test]
    fn err_when_string_parameter_used_in_vec_context() {
        // A String parameter cannot be resolved into a VecString field.
        let defs = vec![make_def("HOST", ParameterType::String)];
        let found = vec![make_found("HOST", FoundParameterType::VecString)];
        let msg = format!("{:?}", validate(&found, &defs, false, false).unwrap_err());
        assert!(
            msg.contains("Type mismatches"),
            "expected type error, got: {msg}"
        );
        assert!(msg.contains("HOST"));
    }

    #[test]
    fn err_when_vec_string_parameter_interpolated_in_string() {
        // Vec<String> cannot be coerced into a string interpolation context.
        let defs = vec![make_def("TAGS", ParameterType::VecString)];
        let found = vec![make_found("TAGS", FoundParameterType::String)];
        let msg = format!("{:?}", validate(&found, &defs, false, false).unwrap_err());
        assert!(
            msg.contains("Type mismatches"),
            "expected type error, got: {msg}"
        );
        assert!(msg.contains("TAGS"));
    }

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

    #[test]
    fn err_deduplicates_repeated_type_mismatch_for_same_parameter() {
        // HOST appears twice with the same incompatible type; error should appear once.
        let defs = vec![make_def("HOST", ParameterType::String)];
        let found = vec![
            make_found("HOST", FoundParameterType::VecString),
            make_found("HOST", FoundParameterType::VecString),
        ];
        let msg = format!("{:?}", validate(&found, &defs, false, false).unwrap_err());
        // "'HOST': used as" appears once means the type error was deduplicated
        assert_eq!(
            msg.matches("used as VecString").count(),
            1,
            "duplicate type error should be deduplicated"
        );
    }

    // =========================================================================
    // Duplicate definitions
    // =========================================================================

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
}
