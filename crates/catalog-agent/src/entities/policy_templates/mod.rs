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

//! Policy templates: pieces of ODRL with `$parameter` placeholders that can reference each other.

pub mod placeholder;
pub mod types;
pub mod validator;

use std::collections::HashMap;
use std::fmt::Display;

use common::dsp_common::odrl::OdrlPolicyInfo;
use common::eunomia::document::JsonLd;
use ymir::oauth::Owner;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ymir::errors::{Errors, Outcome};
use ymir::types::oauth::RolePath;
use ymir::types::participants::Visibility;

use crate::entities::commands::NewPolicyTemplateCommand;
use crate::entities::policy_instantiation::validators::ValidatorFactory;
use crate::entities::policy_templates::types::{LocalizedText, ParameterDefinition};

type DateTimeWithTimeZone = chrono::DateTime<chrono::FixedOffset>;

/// Value of a parameter, as JSON: a boolean, a number, a string or a list of strings.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum ParameterValue {
    Boolean(bool),
    Number(serde_json::Number),
    Text(String),
    List(Vec<String>),
}

impl ParameterValue {
    /// The value as it goes into the policy.
    pub fn to_json(&self) -> Value {
        match self {
            Self::Boolean(b) => Value::Bool(*b),
            Self::Number(n) => Value::Number(n.clone()),
            Self::Text(t) => Value::String(t.clone()),
            Self::List(items) => items.iter().cloned().map(Value::String).collect(),
        }
    }
}

/// What a template's content is: one constraint, one rule or a whole policy.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "camelCase")]
pub enum PolicyTemplateKind {
    Constraint,
    Rule,
    #[default]
    Policy,
}

impl PolicyTemplateKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Constraint => "constraint",
            Self::Rule => "rule",
            Self::Policy => "policy",
        }
    }
}

impl Display for PolicyTemplateKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for PolicyTemplateKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "constraint" => Ok(Self::Constraint),
            "rule" => Ok(Self::Rule),
            "policy" => Ok(Self::Policy),
            other => Err(format!("unknown template kind: {other}")),
        }
    }
}

/// Where a piece is meant to be enforced, as the control points of the policy plan group them.
/// Informative only: nothing evaluates policies yet.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "camelCase")]
pub enum ControlPoint {
    /// Catalog requests and public profiles (P1, P1b).
    Catalog,
    /// Opening and agreeing a contract (P2–P6).
    Negotiation,
    /// Requesting and starting a transfer (P7, P8).
    Transfer,
    /// Each pull request or push of the dataplane (P9, P10).
    Dataplane,
    /// The agreement's lifetime (P11).
    Validity,
    /// Duties and obligations after use (P12).
    Duties,
}

/// Policy template as returned by the API. `content` keeps its `@ref` and `@include` pieces.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PolicyTemplateDto {
    pub id: String,
    pub user_id: String,
    pub user_role: RolePath,
    pub visibility: Visibility,
    pub version: String,
    #[serde(default)]
    pub kind: PolicyTemplateKind,
    #[serde(default)]
    pub control_points: Vec<ControlPoint>,
    pub date: DateTimeWithTimeZone,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<LocalizedText>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<LocalizedText>,
    pub author: String,
    pub content: Value,
    #[serde(default)]
    pub parameters: HashMap<String, ParameterDefinition>,
}

/// New policy template or new version of one.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NewPolicyTemplateDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Who else sees it; private by default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<Visibility>,
    /// Owner an in-process flow asks for; only honoured for the root.
    #[serde(skip)]
    pub owner: Option<Owner>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// `policy` when not given.
    #[serde(default)]
    pub kind: PolicyTemplateKind,
    #[serde(default)]
    pub control_points: Vec<ControlPoint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<DateTimeWithTimeZone>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<LocalizedText>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<LocalizedText>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    pub content: Value,
    #[serde(default)]
    pub parameters: HashMap<String, ParameterDefinition>,
}

ymir::impl_owned!(PolicyTemplateDto);

impl PolicyTemplateDto {
    /// `id@version`, the way references name it.
    pub fn reference(&self) -> String {
        format!("{}@{}", self.id, self.version)
    }

    /// The values the content is filled with: the given ones over the defaults, each checked
    /// against its definition. Unknown or missing parameters fail.
    pub fn resolve_values(&self, given: &HashMap<String, Value>) -> Outcome<HashMap<String, Value>> {
        if let Some(unknown) = given.keys().find(|k| !self.parameters.contains_key(*k)) {
            return Err(Errors::validation(
                format!("{} has no parameter '{unknown}'", self.reference()),
                None,
            ));
        }
        let mut values = HashMap::with_capacity(self.parameters.len());
        for (name, definition) in &self.parameters {
            let value = match (given.get(name), &definition.default_value) {
                (Some(value), _) => value.clone(),
                (None, Some(default)) => default.to_json(),
                (None, None) => {
                    return Err(Errors::validation(
                        format!("{} needs a value for '{name}'", self.reference()),
                        None,
                    ))
                }
            };
            let typed: ParameterValue = serde_json::from_value(value.clone()).map_err(|_| {
                Errors::validation(format!("'{name}' of {} cannot be {value}", self.reference()), None)
            })?;
            ValidatorFactory::get_validator(definition.data_type)
                .validate(&typed, &definition.restrictions)
                .map_err(|e| {
                    Errors::validation(format!("'{name}' of {}: {e}", self.reference()), None)
                })?;
            values.insert(name.clone(), value);
        }
        Ok(values)
    }
}

impl NewPolicyTemplateDto {
    /// The content as stored: a policy loses the ODRL `@context` and `@type` it may come with.
    pub fn stored_content(&self) -> Outcome<Value> {
        match self.kind {
            PolicyTemplateKind::Policy => JsonLd::<OdrlPolicyInfo>::untag(self.content.clone())
                .map_err(|e| Errors::validation(format!("content: {e}"), None)),
            _ => Ok(self.content.clone()),
        }
    }

    /// The command creating it for `owner`; fails when content or parameters cannot be serialized.
    pub fn into_command(self, owner: Owner) -> Outcome<NewPolicyTemplateCommand> {
        Ok(NewPolicyTemplateCommand {
            content: self.stored_content()?,
            id: self.id,
            owner,
            version: self.version,
            kind: self.kind,
            control_points: self.control_points,
            date: self.date,
            author: self.author,
            title: self.title.map(serde_json::to_value).transpose()?,
            description: self.description.map(serde_json::to_value).transpose()?,
            parameters: serde_json::to_value(self.parameters)?,
            references: Vec::new(),
        })
    }
}
