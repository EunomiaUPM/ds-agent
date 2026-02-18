use crate::entities::common::parameter_mutator::TemplateMutator;
use crate::entities::common::parameter_visitor::{ExpectedType, ParameterVisitor};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
pub enum ParameterAutoFilledType {
    SysUrn,
    SysToken,
    SysTimestamp,
    SysIso8601,
    SysOwnUrl {
        host_docker_internal: bool
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoFillable {
    #[serde(default)]
    pub auto_filled: bool,
    pub auto_filled_type: Option<ParameterAutoFilledType>,
}

impl Default for AutoFillable {
    fn default() -> Self {
        Self { auto_filled: false, auto_filled_type: None }
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
    #[serde(flatten)]
    pub auto_fillable: AutoFillable,
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

pub trait TemplateVisitable {
    fn accept<V: ParameterVisitor>(&mut self, visitor: &mut V) -> Result<(), V::Error>;
}

pub trait TemplateMutable {
    fn accept_mutator<V: TemplateMutator>(&mut self, visitor: &mut V) -> Result<(), V::Error>;
}

impl TemplateVisitable for TemplateString {
    fn accept<V: ParameterVisitor>(&mut self, visitor: &mut V) -> Result<(), V::Error> {
        let _ = visitor.scan_template_candidate(self.as_str(), ExpectedType::AnyString);
        Ok(())
    }
}

impl TemplateMutable for TemplateString {
    fn accept_mutator<V: TemplateMutator>(&mut self, visitor: &mut V) -> Result<(), V::Error> {
        if let Some(resolved_val) = visitor.resolve(self) {
            *self = match resolved_val {
                serde_json::Value::String(s) => s,
                v => v.to_string(), // Si llega un número, lo volvemos string
            };
        }
        Ok(())
    }
}

impl TemplateVisitable for TemplateInt {
    fn accept<V: ParameterVisitor>(&mut self, visitor: &mut V) -> Result<(), V::Error> {
        match self {
            TemplateInt::Value(_) => Ok(()),
            TemplateInt::Template(str) => {
                visitor.scan_template_candidate(str, ExpectedType::StrictInt);
                Ok(())
            }
        }
    }
}

impl TemplateMutable for TemplateInt {
    fn accept_mutator<V: TemplateMutator>(&mut self, visitor: &mut V) -> Result<(), V::Error> {
        if let TemplateInt::Template(tmpl) = self {
            if let Some(val) = visitor.resolve(tmpl) {
                // Simplificación: Confiamos en que es un número.
                // serde_json::from_value maneja la conversión de Number(json) a i64(rust)
                let int_val: i64 = serde_json::from_value(val).map_err(|e| {
                    anyhow!("Failed to convert resolved value to i64 for '{}': {}", tmpl, e)
                })?;

                *self = TemplateInt::Value(int_val);
            }
        }
        Ok(())
    }
}

impl TemplateVisitable for TemplateBoolean {
    fn accept<V: ParameterVisitor>(&mut self, visitor: &mut V) -> Result<(), V::Error> {
        match self {
            TemplateBoolean::Value(_) => Ok(()),
            TemplateBoolean::Template(s) => {
                visitor.scan_template_candidate(s, ExpectedType::StrictBool);
                Ok(())
            }
        }
    }
}

impl TemplateMutable for TemplateBoolean {
    fn accept_mutator<V: TemplateMutator>(&mut self, visitor: &mut V) -> Result<(), V::Error> {
        if let TemplateBoolean::Template(tmpl) = self {
            if let Some(val) = visitor.resolve(tmpl) {
                let bool_val: bool = serde_json::from_value(val).map_err(|e| {
                    anyhow!("Failed to convert resolved value to bool for '{}': {}", tmpl, e)
                })?;

                *self = TemplateBoolean::Value(bool_val);
            }
        }
        Ok(())
    }
}

impl TemplateVisitable for TemplateVecString {
    fn accept<V: ParameterVisitor>(&mut self, visitor: &mut V) -> Result<(), V::Error> {
        match self {
            TemplateVecString::Value(list) => {
                for (index, item) in list.iter_mut().enumerate() {
                    visitor.enter_scope(&index.to_string());
                    visitor.scan_template_candidate(item, ExpectedType::AnyString);
                    visitor.exit_scope();
                }
            }
            TemplateVecString::Template(s) => {
                visitor.scan_template_candidate(s, ExpectedType::StrictVec);
            }
        }
        Ok(())
    }
}

impl TemplateMutable for TemplateVecString {
    fn accept_mutator<V: TemplateMutator>(&mut self, visitor: &mut V) -> Result<(), V::Error> {
        match self {
            // CASE A: replace array (ej: "{{__tags__}}")
            TemplateVecString::Template(tmpl) => {
                if let Some(val) = visitor.resolve(tmpl) {
                    // Simplificación masiva: El validador ya aseguró que es Vec<String>
                    let list: Vec<String> = serde_json::from_value(val).map_err(|e| {
                        anyhow!(
                            "Failed to convert resolved value to Vec<String> for '{}': {}",
                            tmpl,
                            e
                        )
                    })?;

                    *self = TemplateVecString::Value(list);
                }
            }
            // CASE B: interpolate within array (ex: ["prod", "{{__region__}}"])
            TemplateVecString::Value(list) => {
                for (idx, item) in list.iter_mut().enumerate() {
                    visitor.enter_scope(&idx.to_string());
                    item.accept_mutator(visitor)?;
                    visitor.exit_scope();
                }
            }
        }
        Ok(())
    }
}

impl TemplateVisitable for TemplateMapString {
    fn accept<V: ParameterVisitor>(&mut self, visitor: &mut V) -> Result<(), V::Error> {
        match self {
            TemplateMapString::Value(map) => {
                for (key, value) in map.iter_mut() {
                    visitor.enter_scope(key);
                    visitor.scan_template_candidate(value, ExpectedType::AnyString);
                    visitor.exit_scope();
                }
            }
            TemplateMapString::Template(s) => {
                visitor.scan_template_candidate(s, ExpectedType::StrictMap);
            }
        }
        Ok(())
    }
}

impl TemplateMutable for TemplateMapString {
    fn accept_mutator<V: TemplateMutator>(&mut self, visitor: &mut V) -> Result<(), V::Error> {
        match self {
            // CASE A: replace map (ex: "{{__env_vars__}}")
            TemplateMapString::Template(tmpl) => {
                if let Some(val) = visitor.resolve(tmpl) {
                    // Simplification: The validator already ensured it's Map<String, String>
                    let map: HashMap<String, String> = serde_json::from_value(val)
                        .map_err(|e| anyhow!("Failed to convert resolved value to Map<String, String> for '{}': {}", tmpl, e))?;

                    *self = TemplateMapString::Value(map);
                }
            }
            // CASE B: interpolate within map (ex: ["prod", "{{__region__}}"])
            TemplateMapString::Value(map) => {
                for (key, value) in map.iter_mut() {
                    visitor.enter_scope(key);
                    value.accept_mutator(visitor)?;
                    visitor.exit_scope();
                }
            }
        }
        Ok(())
    }
}
