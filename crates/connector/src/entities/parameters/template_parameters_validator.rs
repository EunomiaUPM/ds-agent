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
    matches!(
        (found, defined),
        (FoundParameterType::String, ParameterType::String)
            | (FoundParameterType::String, ParameterType::Int)
            | (FoundParameterType::String, ParameterType::Boolean)
            | (FoundParameterType::VecString, ParameterType::VecString)
            | (
                FoundParameterType::MapString,
                ParameterType::MapStringString
            )
    )
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
            Err(Errors::validation(errors.join("; "), None))
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
