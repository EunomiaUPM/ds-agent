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

pub(crate) mod connector_template_walker;
pub(crate) mod instance_parameters_map;
pub(crate) mod instance_parameters_resolver;
pub(crate) mod instance_parameters_validator;
pub(crate) mod jq;
pub mod keystore_lookup;
pub(crate) mod runtime_parameters_resolver;
pub(crate) mod template_parameters_extractor;
pub(crate) mod template_parameters_validator;

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::OnceLock;
use ymir::errors::Errors;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ParameterType {
    String,
    Int,
    Boolean,
    #[serde(rename = "VEC<STRING>")]
    VecString,
    #[serde(rename = "MAP<STRING,STRING>")]
    MapStringString,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SysParameterType {
    SysUrn,
    SysToken,
    SysTimestamp,
    SysIso8601,
    SysOwnUrl { host_docker_internal: bool },
}

impl FromStr for SysParameterType {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // SYS_OWN_URL and SYS_OWN_URL_DOCKER share a variant with different field values —
        // serde cannot represent these two aliases as a plain string, so they stay explicit.
        match s {
            "SYS_OWN_URL" => {
                return Ok(Self::SysOwnUrl {
                    host_docker_internal: false,
                })
            }
            "SYS_OWN_URL_DOCKER" => {
                return Ok(Self::SysOwnUrl {
                    host_docker_internal: true,
                })
            }
            _ => {}
        }
        // All unit variants delegate to serde so new variants don't require a FromStr update.
        serde_json::from_value(serde_json::Value::String(s.to_string()))
            .map_err(|_| Errors::validation(format!("{} system parameter not valid", s), None))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParameterDefinition {
    pub name: String,
    pub title: String,
    pub description: Option<String>,
    pub param_type: ParameterType,
    pub required: bool,
    pub default_value: Option<String>,
}

pub type TemplateString = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TemplateInt {
    Value(i64),
    Template(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TemplateBoolean {
    Value(bool),
    Template(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TemplateVecString {
    Value(Vec<String>),
    Template(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TemplateMapString {
    Value(HashMap<String, String>),
    Template(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FoundParameterType {
    String,
    VecString,
    MapString,
}

#[derive(Clone)]
pub struct FoundParameter {
    pub name: String,
    pub content_type: FoundParameterType,
}

pub(crate) fn template_parameter_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\{\{\s*__(.*?)__\s*\}\}").expect("Invalid Regex"))
}

pub(crate) fn template_sys_parameter_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\{\{\s*__(SYS.*?)__\s*\}\}").expect("Invalid Regex"))
}

/// Matches `{{__RUNTIME_JSON_{<jq_path>}__}}` and captures the jq path.
/// Example: `{{__RUNTIME_JSON_{subscribe.data.ID}__}}` - capture group 1 = `subscribe.data.ID`
pub(crate) fn template_runtime_json_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"\{\{\s*__RUNTIME_JSON_\{(.+?)}__\s*}}").expect("Invalid RUNTIME_JSON regex")
    })
}

/// Matches `{{__RUNTIME_PARAMETER_{/key}__}}` and captures the key path.
pub(crate) fn template_runtime_parameter_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"\{\{\s*__RUNTIME_PARAMETER_\{([^}]+)}__\s*\}\}")
            .expect("Invalid RUNTIME_PARAMETER regex")
    })
}

/// Matches `{{__RUNTIME_SECRET_{/key}__}}` and captures the key path.
pub(crate) fn template_runtime_secret_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"\{\{\s*__RUNTIME_SECRET_\{([^}]+)}__\s*\}\}")
            .expect("Invalid RUNTIME_SECRET regex")
    })
}
