pub(crate) mod template_parameters_extractor;
pub(crate) mod connector_template_walker;
pub(crate) mod template_parameters_validator;
pub(crate) mod instance_parameters_validator;
pub(crate) mod instance_parameters_map;
pub(crate) mod instance_parameters_resolver;

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(rename_all = "UPPERCASE")]
// pub enum ParameterType {
//     String,
//     Int,
//     Boolean,
//     #[serde(rename = "VEC<STRING>")]
//     VecString,
//     #[serde(rename = "MAP<STRING,STRING>")]
//     MapStringString,
// }
//
// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
// pub enum SysParameterType {
//     SysUrn,
//     SysToken,
//     SysTimestamp,
//     SysIso8601,
//     SysOwnUrl { host_docker_internal: bool },
// }
//
// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct ParameterDefinition {
//     pub name: String,
//     pub title: String,
//     pub description: Option<String>,
//     pub param_type: ParameterType,
//     pub required: bool,
//     pub default_value: Option<String>,
// }
//
// pub type TemplateString = String;
//
// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(untagged)]
// pub enum TemplateInt {
//     Value(i64),
//     Template(String),
// }
//
// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(untagged)]
// pub enum TemplateBoolean {
//     Value(bool),
//     Template(String),
// }
//
// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(untagged)]
// pub enum TemplateVecString {
//     Value(Vec<String>),
//     Template(String),
// }
//
// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(untagged)]
// pub enum TemplateMapString {
//     Value(HashMap<String, String>),
//     Template(String),
// }

// #[derive(Debug, Clone, Copy, PartialEq)]
// pub enum FoundParameterType {
//     String,
//     VecString,
//     MapString,
// }

pub(crate) fn template_parameter_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\{\{\s*__(.*?)__\s*\}\}").expect("Invalid Regex"))
}

pub(crate) fn template_sys_parameter_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\{\{\s*__(SYS.*?)__\s*\}\}").expect("Invalid Regex"))
}