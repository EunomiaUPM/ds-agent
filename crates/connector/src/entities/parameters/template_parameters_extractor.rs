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
use super::{template_parameter_regex, template_sys_parameter_regex};
use super::{FoundParameterType, TemplateMapString, TemplateString};
use crate::entities::parameters::connector_template_walker::ConnectorTemplateWalker;
use crate::TemplateVecString;
use std::collections::HashMap;
use ymir::errors::Outcome;

pub struct TemplateParametersExtractor {
    found_parameters: Vec<FoundParameter>,
    regex_fn: fn() -> &'static regex::Regex,
}

impl Default for TemplateParametersExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateParametersExtractor {
    pub fn new() -> Self {
        Self {
            found_parameters: Vec::new(),
            regex_fn: template_parameter_regex,
        }
    }

    pub fn just_system_parameters(mut self) -> Self {
        self.regex_fn = template_sys_parameter_regex;
        self
    }

    pub fn found_parameters(&self) -> &[FoundParameter] {
        &self.found_parameters
    }

    fn scan_str(&mut self, value: &str, content_type: &FoundParameterType) {
        let re = (self.regex_fn)();
        for cap in re.captures_iter(value) {
            self.found_parameters.push(FoundParameter {
                name: cap[1].to_string(),
                content_type: *content_type,
            });
        }
    }

    fn vec_string_parameter_extractor(&mut self, template: &[String]) {
        for s in template {
            self.scan_str(s, &FoundParameterType::VecString);
        }
    }
    fn map_string_parameter_extractor(&mut self, template: &HashMap<String, String>) {
        if let Some(extra) = template.get("__EXTRA__") {
            self.scan_str(extra, &FoundParameterType::MapString);
        }
    }
}

impl ConnectorTemplateWalker for TemplateParametersExtractor {
    fn on_string(&mut self, field: &mut TemplateString) -> Outcome<()> {
        self.scan_str(field, &FoundParameterType::String);
        Ok(())
    }

    fn on_vec_string(&mut self, field: &mut TemplateVecString) -> Outcome<()> {
        match field {
            TemplateVecString::Template(t) => self.scan_str(t, &FoundParameterType::VecString),
            TemplateVecString::Value(v) => self.vec_string_parameter_extractor(v),
        };
        Ok(())
    }

    fn on_map_string(&mut self, field: &mut TemplateMapString) -> Outcome<()> {
        match field {
            TemplateMapString::Template(t) => self.scan_str(t, &FoundParameterType::MapString),
            TemplateMapString::Value(v) => self.map_string_parameter_extractor(v),
        };
        Ok(())
    }
}
