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

//! Walks a [`ConnectorTemplateDto`] and applies a [`ParameterResolverBehavior`]
//! to every templatable field in-place.
//!
//! This is the mutable counterpart of [`ParameterExtractorVisitor`]:
//!
//! | Visitor | Direction | Behavior trait |
//! |---|---|---|
//! | [`ParameterExtractorVisitor`] | read-only | [`ParameterExtractorBehavior`] |
//! | [`TemplateResolverVisitor`] | mutable | [`ParameterResolverBehavior`] |
//!
//! Both visitors implement [`ConnectorTemplateWalker`], which houses the shared
//! structural traversal over auth and interaction fields.
//!
//! # Usage
//!
//! ```ignore
//! let resolver = TemplateParametersResolver::new(&params);
//! TemplateResolverVisitor::new(&resolver).apply(&mut template_spec)?;
//! ```
//!
//! [`ParameterExtractorVisitor`]: super::template_parameters_visitor::ParameterExtractorVisitor
//! [`ParameterExtractorBehavior`]: super::template_parameters_extractor::ParameterExtractorBehavior
//! [`ParameterResolverBehavior`]: super::template_parameters_resolver::ParameterResolverBehavior
//! [`ConnectorTemplateWalker`]: super::template_walker::ConnectorTemplateWalker

use crate::entities::connector_template::ConnectorTemplateDto;
use crate::entities::parameters::template_parameters_resolver::ParameterResolverBehavior;
use crate::entities::parameters::template_walker::ConnectorTemplateWalker;
use crate::entities::parameters::{TemplateMapString, TemplateVecString};
use crate::ProtocolSpec;
use serde_json::Value;
use ymir::errors::{Errors, Outcome};

/// Resolves every `{{__PARAM__}}` placeholder in a [`ConnectorTemplateDto`]
/// in-place using the supplied [`ParameterResolverBehavior`].
///
/// The traversal order is identical to [`ParameterExtractorVisitor`]:
/// authentication fields are visited first, then interaction fields.
///
/// [`ParameterExtractorVisitor`]: super::template_parameters_visitor::ParameterExtractorVisitor
pub struct TemplateResolverVisitor<'a> {
    resolver: &'a mut dyn ParameterResolverBehavior,
}

impl<'a> TemplateResolverVisitor<'a> {
    pub fn new(resolver: &'a mut dyn ParameterResolverBehavior) -> Self {
        Self { resolver }
    }

    /// Resolve all `{{__PARAM__}}` placeholders in `template` using
    /// the resolver supplied at construction time, mutating the template in-place.
    pub fn apply(&mut self, template: &mut ConnectorTemplateDto) -> Outcome<()> {
        self.walk(template)
    }

    /// Resolve all placeholders in a single protocol specification in-place.
    pub fn apply_protocol(&mut self, spec: &mut ProtocolSpec) -> Outcome<()> {
        self.walk_protocol(spec)
    }
}

impl ConnectorTemplateWalker for TemplateResolverVisitor<'_> {
    fn on_string(&mut self, field: &mut String) -> Outcome<()> {
        if let Some(val) = self.resolver.resolve(field.as_str()) {
            *field = match val {
                Value::String(s) => s,
                v => v.to_string(),
            };
        }
        Ok(())
    }

    /// Resolve a `TemplateVecString` field.
    ///
    /// - `Template` variant: the whole value resolves to a `Vec<String>`.
    /// - `Value` variant: each element is individually interpolated. If an element
    ///   is exactly a `{{__NAME__}}` placeholder and it resolves to an array,
    ///   the array is spliced into the list.
    fn on_vec_string(&mut self, field: &mut TemplateVecString) -> Outcome<()> {
        match field {
            TemplateVecString::Template(tmpl) => {
                if let Some(val) = self.resolver.resolve(tmpl.as_str()) {
                    let list: Vec<String> = serde_json::from_value(val).map_err(|e| {
                        Errors::crazy(
                            format!("Failed to resolve TemplateVecString for '{}': {}", tmpl, e),
                            None,
                        )
                    })?;
                    *field = TemplateVecString::Value(list);
                }
            }
            TemplateVecString::Value(list) => {
                let mut new_list = Vec::with_capacity(list.len());
                for (idx, item) in list.iter().enumerate() {
                    self.resolver.enter_scope(&idx.to_string());
                    let mut resolved_item = item.clone();
                    if let Some(val) = self.resolver.resolve(item.as_str()) {
                        match val {
                            Value::Array(arr) => {
                                for v in arr {
                                    new_list.push(match v {
                                        Value::String(s) => s,
                                        other => other.to_string(),
                                    });
                                }
                                self.resolver.exit_scope();
                                continue;
                            }
                            Value::String(s) => resolved_item = s,
                            other => resolved_item = other.to_string(),
                        }
                    }
                    new_list.push(resolved_item);
                    self.resolver.exit_scope();
                }
                *list = new_list;
            }
        }
        Ok(())
    }

    /// Resolve a `TemplateMapString` field.
    ///
    /// - `Template` variant: the whole value resolves to a `HashMap<String, String>`.
    /// - `Value` variant: each map _value_ is individually interpolated (keys are static).
    ///
    /// Special key: `__EXTRA__`
    /// If a key is exactly `__EXTRA__` and its value (after resolution) is a JSON object,
    /// that object is flattened and its keys are merged into the top-level map,
    /// and the `__EXTRA__` key is removed.
    fn on_map_string(&mut self, field: &mut TemplateMapString) -> Outcome<()> {
        match field {
            TemplateMapString::Template(tmpl) => {
                if let Some(val) = self.resolver.resolve(tmpl.as_str()) {
                    let map: std::collections::HashMap<String, String> =
                        serde_json::from_value(val).map_err(|e| {
                            Errors::crazy(
                                format!(
                                    "Failed to resolve TemplateMapString for '{}': {}",
                                    tmpl, e
                                ),
                                None,
                            )
                        })?;
                    *field = TemplateMapString::Value(map);
                }
            }
            TemplateMapString::Value(map) => {
                let mut to_merge = std::collections::HashMap::new();
                let mut to_remove = Vec::new();

                for (key, value) in map.iter_mut() {
                    self.resolver.enter_scope(key);
                    if key == "__EXTRA__" {
                        if let Some(val) = self.resolver.resolve(value.as_str()) {
                            if let Value::Object(obj) = val {
                                for (k, v) in obj {
                                    to_merge.insert(
                                        k,
                                        match v {
                                            Value::String(s) => s,
                                            other => other.to_string(),
                                        },
                                    );
                                }
                                to_remove.push(key.clone());
                            } else {
                                // Fallback: just resolve the string normally
                                *value = match val {
                                    Value::String(s) => s,
                                    other => other.to_string(),
                                };
                            }
                        }
                    } else {
                        self.on_string(value)?;
                    }
                    self.resolver.exit_scope();
                }

                for k in to_remove {
                    map.remove(&k);
                }
                for (k, v) in to_merge {
                    map.insert(k, v);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
