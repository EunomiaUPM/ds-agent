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

use super::{template_parameter_regex, TemplateMapString, TemplateString, TemplateVecString};
use crate::entities::connector_template::ConnectorTemplateDto;
use crate::entities::parameters::connector_template_walker::ConnectorTemplateWalker;
use regex::Regex;
use std::collections::HashMap;
use ymir::errors::{Errors, Outcome};

pub struct InstanceParametersResolver<'a> {
    connector_template: &'a ConnectorTemplateDto,
    params: &'a HashMap<String, serde_json::Value>,
}

impl<'a> InstanceParametersResolver<'a> {
    pub fn new(
        connector_template: &'a ConnectorTemplateDto,
        params: &'a HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            connector_template,
            params,
        }
    }

    pub fn resolve(&mut self) -> Outcome<ConnectorTemplateDto> {
        let mut template = self.connector_template.clone();
        self.walk(&mut template)?;
        Ok(template)
    }

    fn resolve_key(&self, key: &str) -> Option<serde_json::Value> {
        self.params.get(key).cloned()
    }

    fn try_exact_replacement(&self, re: &Regex, raw: &str) -> Option<serde_json::Value> {
        let caps = re.captures(raw)?;
        let full_match = caps.get(0)?.as_str();
        if full_match != raw {
            return None;
        }
        let key = caps.get(1)?.as_str();
        self.resolve_key(key)
    }

    fn try_string_interpolation(&self, re: &Regex, raw: &str) -> Option<serde_json::Value> {
        if !re.is_match(raw) {
            return None;
        }
        let mut new_string = raw.to_string();
        for caps in re.captures_iter(raw) {
            let full_match = &caps[0];
            let key = &caps[1];

            if let Some(val) = self.resolve_key(key) {
                let replacement_str = self.value_to_string(&val);
                new_string = new_string.replace(full_match, &replacement_str);
            }
        }
        Some(serde_json::Value::String(new_string))
    }

    fn value_to_string(&self, val: &serde_json::Value) -> String {
        match val {
            serde_json::Value::String(s) => s.clone(),
            _ => val.to_string(),
        }
    }

    fn value_resolver(&self, raw: &str) -> Option<serde_json::Value> {
        let re = template_parameter_regex();
        if let Some(val) = self.try_exact_replacement(re, raw) {
            return Some(val);
        }
        self.try_string_interpolation(re, raw)
    }
}

impl<'a> ConnectorTemplateWalker for InstanceParametersResolver<'a> {
    fn on_string(&mut self, field: &mut TemplateString) -> Outcome<()> {
        if let Some(val) = self.value_resolver(field.as_str()) {
            *field = match val {
                serde_json::Value::String(s) => s,
                v => v.to_string(),
            };
        }
        Ok(())
    }

    fn on_vec_string(&mut self, field: &mut TemplateVecString) -> Outcome<()> {
        match field {
            TemplateVecString::Template(tmpl) => {
                if let Some(val) = self.value_resolver(tmpl.as_str()) {
                    let list: Vec<String> = serde_json::from_value(val).map_err(|e| {
                        Errors::crazy(
                            format!("Failed to resolve TemplateVecString for '{tmpl}': {e}"),
                            None,
                        )
                    })?;
                    *field = TemplateVecString::Value(list);
                }
            }
            TemplateVecString::Value(list) => {
                let mut new_list = Vec::with_capacity(list.len());
                for item in list.iter() {
                    let mut resolved_item = item.clone();
                    if let Some(val) = self.value_resolver(item.as_str()) {
                        match val {
                            serde_json::Value::Array(arr) => {
                                for v in arr {
                                    new_list.push(match v {
                                        serde_json::Value::String(s) => s,
                                        other => other.to_string(),
                                    });
                                }
                                continue;
                            }
                            serde_json::Value::String(s) => resolved_item = s,
                            other => resolved_item = other.to_string(),
                        }
                    }
                    new_list.push(resolved_item);
                }
                *list = new_list;
            }
        }
        Ok(())
    }

    fn on_map_string(&mut self, field: &mut TemplateMapString) -> Outcome<()> {
        match field {
            TemplateMapString::Template(tmpl) => {
                if let Some(val) = self.value_resolver(tmpl.as_str()) {
                    let map: std::collections::HashMap<String, String> =
                        serde_json::from_value(val).map_err(|e| {
                            Errors::crazy(
                                format!("Failed to resolve TemplateMapString for '{tmpl}': {e}"),
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
                    if key == "__EXTRA__" {
                        if let Some(val) = self.value_resolver(value.as_str()) {
                            if let serde_json::Value::Object(obj) = val {
                                for (k, v) in obj {
                                    to_merge.insert(
                                        k,
                                        match v {
                                            serde_json::Value::String(s) => s,
                                            other => other.to_string(),
                                        },
                                    );
                                }
                                to_remove.push(key.clone());
                            } else {
                                *value = match val {
                                    serde_json::Value::String(s) => s,
                                    other => other.to_string(),
                                };
                            }
                        }
                    } else {
                        self.on_string(value)?;
                    }
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
