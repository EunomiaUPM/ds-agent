pub(crate) mod template_parameters_extractor;
pub(crate) mod connector_template_walker;
pub(crate) mod template_parameters_validator;
pub(crate) mod instance_parameters_validator;
pub(crate) mod instance_parameters_map;
pub(crate) mod instance_parameters_resolver;

use std::collections::HashMap;
use std::str::FromStr;
use regex::Regex;
use serde::{Deserialize, Serialize};
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
        match s {
            "SYS_URN"        => Ok(Self::SysUrn),
            "SYS_TOKEN"      => Ok(Self::SysToken),
            "SYS_TIMESTAMP"  => Ok(Self::SysTimestamp),
            "SYS_ISO8601"    => Ok(Self::SysIso8601),
            "SYS_OWN_URL"    => Ok(Self::SysOwnUrl { host_docker_internal: false }),
            "SYS_OWN_URL_DOCKER" => Ok(Self::SysOwnUrl { host_docker_internal: true }),
            _ => Err(Errors::validation(format!("{} system parameter not valid", s), None)),
        }
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

#[allow(dead_code)]
pub(crate) fn template_runtime_parameter_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    // Matches: RUNTIME_{TYPE}_RESPONSE_{jq_expr}
    // Example: RUNTIME_SUB_RESPONSE_{.id}
    //          RUNTIME_AUTH_RESPONSE_{.access_token}
    REGEX.get_or_init(|| {
        Regex::new(r"^RUNTIME_([A-Z_]+)_RESPONSE_\{(.+)\}$").expect("Invalid response key regex")
    })
}