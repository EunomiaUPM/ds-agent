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
use crate::entities::parameters::FoundParameterType;
use std::collections::{HashMap, HashSet};
use ymir::errors::{Errors, Outcome};

/// Validates that the set of parameter names found in a connector template
/// matches the set of [`ParameterDefinition`]s declared in `parameters[]`,
/// and that each usage is type-compatible with its declaration.
///
/// Four kinds of mismatches are detected and reported together:
///
/// - **Duplicate definitions** — two entries in `parameters[]` share the same
///   `name` or the same `title`.
/// - **Undeclared** — a placeholder `{{__NAME__}}` appears in the template but
///   no `ParameterDefinition` with that `name` exists.
/// - **Unused** — a `ParameterDefinition` is declared but its placeholder never
///   appears in the template.
/// - **Type mismatch** — a placeholder is used in a context whose type is
///   incompatible with the declared [`ParameterType`].
///
/// Duplicate occurrences in `parameters_found` are collapsed before comparison
/// so that using the same placeholder in multiple fields does not produce false
/// positives.
pub struct ParameterValidator<'a> {
    parameters: &'a [ParameterDefinition],
    /// When `true`, names that start with `RUNTIME_` or `SYS_` are silently
    /// ignored during validation. These names are injected by the runtime engine
    /// at execution time and are not expected to appear in `parameters[]`.
    exclude_runtime: bool,
}

/// Returns `true` when a parameter used in a `found` context can satisfy a
/// definition whose declared type is `defined`.
///
/// ## Compatibility matrix
///
/// | `FoundParameterType` | Compatible `ParameterType`(s)          | Rationale |
/// |----------------------|----------------------------------------|-----------|
/// | `String`             | `String`, `Int`, `Boolean`             | A placeholder embedded inside a string field (e.g. `"port={{__PORT__}}"`) coerces any scalar to its string representation. Composite types (`VecString`, `MapStringString`) cannot be meaningfully serialised into a single string slot. |
/// | `Int`                | `Int`                                  | The placeholder occupies an entire `TemplateInt::Template` slot; the value must be an integer. |
/// | `Boolean`            | `Boolean`                              | The placeholder occupies an entire `TemplateBoolean::Template` slot; the value must be a boolean. |
/// | `VecString`          | `VecString`                            | The placeholder either occupies an entire `TemplateVecString::Template` slot or appears as an element inside a `Value` vec. |
/// | `MapString`          | `MapStringString`                      | The placeholder either occupies an entire `TemplateMapString::Template` slot or appears as the value of the `__EXTRA__` key inside a `Value` map. |
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

impl<'a> ParameterValidator<'a> {
    pub fn new(parameters: &'a [ParameterDefinition], exclude_runtime: bool) -> Self {
        Self { parameters, exclude_runtime }
    }

    /// Validates `parameters_found` against the declared definitions.
    ///
    /// Returns `Ok(())` when all checks pass. Returns `Err` listing every
    /// issue found: duplicate definitions, undeclared names, unused names,
    /// and type mismatches.
    pub fn validate(&self, parameters_found: &[FoundParameter]) -> Outcome<()> {
        let found_filtered = self.filter_runtime(parameters_found);

        let found_names: HashSet<&str> = found_filtered.iter().map(|s| s.name.as_str()).collect();

        let defined_map: HashMap<&str, &ParameterDefinition> =
            self.parameters.iter().map(|p| (p.name.as_str(), p)).collect();

        let defined_names: HashSet<&str> = defined_map.keys().copied().collect();

        let mut errors: Vec<String> = Vec::new();

        errors.extend(self.check_duplicate_definitions());
        errors.extend(self.check_undeclared(&found_names, &defined_names));
        errors.extend(self.check_unused(&found_names, &defined_names));
        errors.extend(self.check_type_compatibility(&found_filtered, &defined_map));

        if errors.is_empty() {
            Ok(())
        } else {
            Err(Errors::parse(&errors.join("; "), None))
        }
    }

    /// Removes runtime-injected parameters from `found` when `exclude_runtime`
    /// is set. Names prefixed with `RUNTIME_` or `SYS_` are injected by the
    /// engine at execution time and must not appear in `parameters[]`.
    fn filter_runtime<'b>(&self, found: &'b [FoundParameter]) -> Vec<&'b FoundParameter> {
        found
            .iter()
            .filter(|s| !self.exclude_runtime || !s.name.starts_with("RUNTIME_"))
            .filter(|s| !self.exclude_runtime || !s.name.starts_with("SYS_"))
            .collect()
    }

    /// Checks that no two entries in `self.parameters` share the same `name`
    /// or the same `title`. Returns one error string per violated field.
    fn check_duplicate_definitions(&self) -> Vec<String> {
        let mut seen_names: HashSet<&str> = HashSet::new();
        let mut dup_names: Vec<&str> = Vec::new();
        let mut seen_titles: HashSet<&str> = HashSet::new();
        let mut dup_titles: Vec<&str> = Vec::new();

        for p in self.parameters {
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

    /// Returns an error string when placeholders appear in the template that
    /// have no matching entry in `parameters[]`.
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
        vec![format!("Undeclared parameters found in template: [{}]", undeclared.join(", "))]
    }

    /// Returns an error string when entries in `parameters[]` have no
    /// corresponding placeholder in the template.
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
        vec![format!("Declared parameters not found in template: [{}]", unused.join(", "))]
    }

    /// Returns an error string for every placeholder whose usage context is
    /// incompatible with its declared [`ParameterType`]. Duplicate entries
    /// (same name, same mismatch) are collapsed.
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
#[cfg(test)]
mod tests;
