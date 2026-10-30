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

//! Composition of policy templates: `{"@ref": "id@version", "with": {...}}` pieces inside template
//! content, and `@include` lists of whole policies, all flattened to plain ODRL.

pub mod composer;
pub mod resolver;

use std::collections::{BTreeSet, HashMap};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use ymir::errors::{Errors, Outcome};

use crate::entities::policy_templates::{ControlPoint, PolicyTemplateDto, PolicyTemplateKind};

/// Key naming the referenced template.
pub const REF: &str = "@ref";
/// Key of the parameter bindings next to `@ref`.
pub const WITH: &str = "with";
/// Key of a policy listing other policies whose rules it adds.
pub const INCLUDE: &str = "@include";
/// The three rule lists of an ODRL policy.
pub const RULE_LISTS: [&str; 3] = ["permission", "prohibition", "obligation"];

/// A reference to one template version, with the values bound to its parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateRef {
    pub id: String,
    pub version: String,
    /// Values by parameter name (`$name`); a value may itself name a parameter of the parent.
    #[serde(default)]
    pub with: HashMap<String, Value>,
}

impl TemplateRef {
    /// Whether a JSON value is a reference object.
    pub fn is_ref(value: &Value) -> bool {
        value.as_object().is_some_and(|o| o.contains_key(REF))
    }

    /// The reference an object spells; `extra` names the keys allowed beside `@ref` and `with`.
    pub fn parse(object: &Map<String, Value>, extra: &[&str], path: &str) -> Outcome<Self> {
        let name = object.get(REF).and_then(Value::as_str).ok_or_else(|| {
            Errors::validation(format!("{path}: {REF} must be a string 'id@version'"), None)
        })?;
        let (id, version) = name
            .rsplit_once('@')
            .filter(|(id, version)| !id.is_empty() && !version.is_empty())
            .ok_or_else(|| {
                Errors::validation(format!("{path}: '{name}' must be 'id@version'"), None)
            })?;
        if let Some(stray) = object.keys().find(|k| !matches!(k.as_str(), REF | WITH) && !extra.contains(&k.as_str())) {
            return Err(Errors::validation(format!("{path}: '{stray}' is not allowed beside {REF}"), None));
        }
        let with = match object.get(WITH) {
            None => HashMap::new(),
            Some(Value::Object(bindings)) => bindings.clone().into_iter().collect(),
            Some(_) => {
                return Err(Errors::validation(format!("{path}: {WITH} must be an object"), None))
            }
        };
        Ok(Self { id: id.to_string(), version: version.to_string(), with })
    }

    /// `id@version`.
    pub fn name(&self) -> String {
        format!("{}@{}", self.id, self.version)
    }
}

/// Where a reference sits, and so which kind of template it may name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    /// Where a constraint goes: a rule's `constraint` or a logical constraint's operands.
    Constraint,
    /// An item of one of the policy's rule lists.
    Rule(&'static str),
    /// An item of a policy's `@include`.
    Include,
}

impl Slot {
    pub fn kind(&self) -> PolicyTemplateKind {
        match self {
            Self::Constraint => PolicyTemplateKind::Constraint,
            Self::Rule(_) => PolicyTemplateKind::Rule,
            Self::Include => PolicyTemplateKind::Policy,
        }
    }
}

/// The template versions a composition may use, by `id@version`.
#[derive(Debug, Clone, Default)]
pub struct TemplateLibrary {
    templates: HashMap<String, PolicyTemplateDto>,
}

impl TemplateLibrary {
    pub fn new(templates: impl IntoIterator<Item = PolicyTemplateDto>) -> Self {
        Self { templates: templates.into_iter().map(|t| (t.reference(), t)).collect() }
    }

    pub fn insert(&mut self, template: PolicyTemplateDto) {
        self.templates.insert(template.reference(), template);
    }

    /// Every version held, as `(id, version)`.
    pub fn keys(&self) -> Vec<(String, String)> {
        let mut keys: Vec<(String, String)> =
            self.templates.values().map(|t| (t.id.clone(), t.version.clone())).collect();
        keys.sort();
        keys
    }

    pub fn contains(&self, reference: &TemplateRef) -> bool {
        self.templates.contains_key(&reference.name())
    }

    /// The referenced version; it fails as missing when the caller cannot see it either.
    pub fn get(&self, reference: &TemplateRef) -> Outcome<&PolicyTemplateDto> {
        self.templates.get(&reference.name()).ok_or_else(|| {
            Errors::validation(format!("{} does not exist or is not visible", reference.name()), None)
        })
    }

    /// The control points of the template and every piece below it.
    pub fn control_points(&self, template: &PolicyTemplateDto) -> BTreeSet<ControlPoint> {
        let mut points: BTreeSet<ControlPoint> = template.control_points.iter().copied().collect();
        let mut pending = vec![template.clone()];
        let mut seen = BTreeSet::new();
        while let Some(current) = pending.pop() {
            if !seen.insert(current.reference()) {
                continue;
            }
            let children = resolver::CompositionResolver::references(current.kind, &current.content)
                .unwrap_or_default();
            for (reference, _) in children {
                if let Ok(child) = self.get(&reference) {
                    points.extend(child.control_points.iter().copied());
                    pending.push(child.clone());
                }
            }
        }
        points
    }
}

/// Asks for the plain ODRL of a stored template version, or of a draft, filled with `parameters`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct TemplatePreviewDto {
    /// Stored version to preview, with `version`; leave both out to preview `template`.
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    /// A template not stored yet.
    #[serde(default)]
    pub template: Option<crate::entities::policy_templates::NewPolicyTemplateDto>,
    #[serde(default)]
    pub parameters: HashMap<String, crate::entities::policy_templates::ParameterValue>,
}

impl TemplatePreviewDto {
    /// The parameter values as JSON.
    pub fn values(&self) -> HashMap<String, Value> {
        self.parameters.iter().map(|(k, v)| (k.clone(), v.to_json())).collect()
    }
}

/// Key of an offer part naming an existing offer whose ODRL it copies.
pub const OFFER: &str = "@offer";
/// Key of a constraint part naming the rule list it is added to.
pub const INTO: &str = "into";

/// One piece of a composed offer.
#[derive(Debug, Clone, PartialEq)]
pub enum OfferPart {
    /// A template version with its values; a constraint names the rule list it joins.
    Template { reference: TemplateRef, into: Option<&'static str> },
    /// An existing offer, copied as it is.
    Offer(urn::Urn),
    /// ODRL written by hand; it may hold references of its own.
    Odrl(Value),
}

impl OfferPart {
    pub fn parse(value: &Value, path: &str) -> Outcome<Self> {
        let object = value
            .as_object()
            .ok_or_else(|| Errors::validation(format!("{path} must be an object"), None))?;
        if object.contains_key(REF) {
            let reference = TemplateRef::parse(object, &[INTO], path)?;
            let into = match object.get(INTO) {
                None => None,
                Some(list) => Some(
                    RULE_LISTS
                        .into_iter()
                        .find(|l| list.as_str() == Some(l))
                        .ok_or_else(|| {
                            Errors::validation(
                                format!("{path}.{INTO} must be permission, prohibition or obligation"),
                                None,
                            )
                        })?,
                ),
            };
            return Ok(Self::Template { reference, into });
        }
        if let Some(offer) = object.get(OFFER) {
            if object.len() > 1 {
                return Err(Errors::validation(format!("{path}: {OFFER} goes alone"), None));
            }
            let id = offer
                .as_str()
                .and_then(|s| s.parse::<urn::Urn>().ok())
                .ok_or_else(|| Errors::validation(format!("{path}.{OFFER} must be an offer URN"), None))?;
            return Ok(Self::Offer(id));
        }
        Ok(Self::Odrl(value.clone()))
    }
}

/// A new offer on a catalog resource, composed from parts that add up in order.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NewPolicyCompositionDto {
    pub entity_id: urn::Urn,
    pub entity_type: crate::entities::odrl_offer::CatalogEntityTypes,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Who else sees it; private by default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<ymir::types::participants::Visibility>,
    pub parts: Vec<Value>,
}

/// The parts of a composition without the resource it would go on, to preview it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct PolicyCompositionPreviewDto {
    pub parts: Vec<Value>,
}
