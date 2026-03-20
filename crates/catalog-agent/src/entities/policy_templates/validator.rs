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

use crate::entities::policy_templates::NewPolicyTemplateDto;
use regex::Regex;
use std::collections::HashSet;
use thiserror::Error;
use ymir::errors::{Errors, Outcome, RepoIntoErrors};

const REGEX_ID: &str = r"^[a-zA-Z0-9\-:._]+$";
const REGEX_VERSION: &str = r"^[a-zA-Z0-9.]+$";
const REGEX_PARAM: &str = r"^\$[a-zA-Z][a-zA-Z0-9_-]*$";

impl NewPolicyTemplateDto {
    pub fn validate_dto(&self) -> Outcome<()> {
        // syntactic validation already performed by serde deserializing
        // template name just alphanumeric,-,:,.,_
        if let Some(id) = &self.id {
            self.validate_value_and_regex(id, REGEX_ID).map_err(|_| {
                PolicyTemplateError::InvalidFormat {
                    field: "id".to_string(),
                    value: id.clone(),
                    pattern: REGEX_ID,
                }
                .into_errors()
            })?;
        }
        // version just alphanumeric and .
        if let Some(ver) = &self.version {
            self.validate_value_and_regex(ver, REGEX_VERSION)
                .map_err(|_| {
                    PolicyTemplateError::InvalidFormat {
                        field: "version".to_string(),
                        value: ver.clone(),
                        pattern: REGEX_VERSION,
                    }
                    .into_errors()
                })?;
        }
        // parameters regex
        // detect existence in JSON of $parameter in literal value
        let parameters = self.detect_parameters()?;
        let unique_params: HashSet<&String> = parameters.iter().collect();

        for param in unique_params {
            self.validate_value_and_regex(param, REGEX_PARAM)
                .map_err(|_| {
                    PolicyTemplateError::InvalidParameterSyntax {
                        parameter: param.to_string(),
                    }
                    .into_errors()
                })?;
            self.validate_parameter_existence(param)?;
        }
        Ok(())
    }
}

pub trait NewPolicyTemplateDtoValidator: Send + Sync {
    fn validate_value_and_regex(&self, value: &str, regex: &str) -> Outcome<()> {
        let re = Regex::new(regex)
            .map_err(|e| Errors::crazy("Invalid regular expression", Some(Box::new(e))))?;
        if !re.is_match(value) {
            return Err(PolicyTemplateError::InvalidFormat {
                field: "unknown".to_string(),
                value: value.to_string(),
                pattern: "regex_mismatch",
            }
            .into_errors());
        }
        Ok(())
    }
    fn detect_parameters(&self) -> Outcome<Vec<String>>;
    fn validate_parameter_existence(&self, parameter: &str) -> Outcome<()>;
}

impl NewPolicyTemplateDtoValidator for NewPolicyTemplateDto {
    fn detect_parameters(&self) -> Outcome<Vec<String>> {
        let content_value = serde_json::to_value(&self.content)?;
        let mut params = Vec::new();
        find_params_recursive(&content_value, &mut params);
        Ok(params)
    }
    fn validate_parameter_existence(&self, parameter: &str) -> Outcome<()> {
        if !self.parameters.contains_key(parameter) {
            return Err(PolicyTemplateError::MissingParameterDefinition {
                parameter: parameter.to_string(),
            }
            .into_errors());
        }
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum PolicyTemplateError {
    #[error(
        "Invalid format for field '{field}'. Value '{value}' does not match pattern '{pattern}'"
    )]
    InvalidFormat {
        field: String,
        value: String,
        pattern: &'static str,
    },
    #[error("Invalid parameter syntax: '{parameter}'. Parameters must start with '$' followed by alphanumeric chars.")]
    InvalidParameterSyntax { parameter: String },
    #[error("Parameter '{parameter}' is used in 'content' ODRL but is not defined in the 'parameters' section.")]
    MissingParameterDefinition { parameter: String },
    #[error("Failed to process template content JSON: {0}")]
    ContentProcessingError(#[from] serde_json::Error),
    #[error("Internal validation configuration error: {0}")]
    RegexError(#[from] regex::Error),
}

impl RepoIntoErrors for PolicyTemplateError {}

fn find_params_recursive(value: &serde_json::Value, collector: &mut Vec<String>) {
    match value {
        serde_json::Value::String(s) => {
            if s.starts_with('$') {
                collector.push(s.clone());
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                find_params_recursive(item, collector);
            }
        }
        serde_json::Value::Object(map) => {
            for (_, v) in map {
                find_params_recursive(v, collector);
            }
        }
        _ => {}
    }
}
