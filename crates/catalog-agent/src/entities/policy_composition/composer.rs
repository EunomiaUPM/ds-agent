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

//! Adds the parts of a composed offer up into one plain ODRL policy.

use std::collections::{BTreeSet, HashMap};

use serde_json::{Map, Value};
use urn::Urn;
use ymir::errors::{Errors, Outcome};

use crate::entities::policy_composition::resolver::CompositionResolver;
use crate::entities::policy_composition::{OfferPart, TemplateLibrary, RULE_LISTS};
use crate::entities::policy_templates::{ControlPoint, PolicyTemplateKind};

/// Composes offers over a library and the offers the parts copy.
pub struct OfferComposer<'a> {
    library: &'a TemplateLibrary,
    offers: &'a HashMap<Urn, Value>,
}

impl<'a> OfferComposer<'a> {
    pub fn new(library: &'a TemplateLibrary, offers: &'a HashMap<Urn, Value>) -> Self {
        Self { library, offers }
    }

    /// Template versions, as `(id, version)`, the parts name directly, inline ODRL included.
    pub fn keys(parts: &[OfferPart]) -> Outcome<Vec<(String, String)>> {
        let mut keys = Vec::new();
        for part in parts {
            match part {
                OfferPart::Template { reference, .. } => keys.push((reference.id.clone(), reference.version.clone())),
                OfferPart::Odrl(content) => {
                    for (reference, _) in CompositionResolver::references(PolicyTemplateKind::Policy, content)? {
                        keys.push((reference.id, reference.version));
                    }
                }
                OfferPart::Offer(_) => {}
            }
        }
        Ok(keys)
    }

    /// The parts added up in order: rules join their lists, and each constraint part joins every
    /// rule of the list it names, once all rules are in.
    pub fn compose(&self, parts: &[OfferPart]) -> Outcome<Value> {
        let resolver = CompositionResolver::new(self.library);
        let mut policy = Map::new();
        let mut constraints: Vec<(&'static str, Value)> = Vec::new();
        for (i, part) in parts.iter().enumerate() {
            let path = format!("parts[{i}]");
            match part {
                OfferPart::Template { reference, into } => {
                    let (template, flat) = resolver.expand_reference(reference)?;
                    match (template.kind, into) {
                        (PolicyTemplateKind::Constraint, Some(list)) => constraints.push((list, flat)),
                        (PolicyTemplateKind::Constraint, None) => {
                            return Err(Errors::validation(
                                format!("{path}: {} is a constraint; say which rules it joins with 'into'", reference.name()),
                                None,
                            ))
                        }
                        (_, Some(_)) => {
                            return Err(Errors::validation(
                                format!("{path}: 'into' is only for constraints"),
                                None,
                            ))
                        }
                        _ => Self::merge(&mut policy, &flat),
                    }
                }
                OfferPart::Offer(id) => {
                    let odrl = self.offers.get(id).ok_or_else(|| {
                        Errors::validation(format!("{path}: offer {id} does not exist or is not visible"), None)
                    })?;
                    Self::merge(&mut policy, odrl);
                }
                OfferPart::Odrl(content) => Self::merge(&mut policy, &resolver.expand_policy(content)?),
            }
        }
        for (list, constraint) in constraints {
            let rules = policy
                .get_mut(list)
                .and_then(Value::as_array_mut)
                .filter(|rules| !rules.is_empty())
                .ok_or_else(|| Errors::validation(format!("there is no {list} rule to add a constraint to"), None))?;
            for rule in rules {
                if let Some(rule) = rule.as_object_mut() {
                    let target = rule.entry("constraint").or_insert_with(|| Value::Array(Vec::new()));
                    if let Value::Array(existing) = target {
                        existing.push(constraint.clone());
                    }
                }
            }
        }
        Ok(Value::Object(policy))
    }

    /// Control points of every template the parts use, below them included.
    pub fn control_points(&self, parts: &[OfferPart]) -> Outcome<BTreeSet<ControlPoint>> {
        let mut points = BTreeSet::new();
        for (id, version) in Self::keys(parts)? {
            let reference = crate::entities::policy_composition::TemplateRef { id, version, with: HashMap::new() };
            if let Ok(template) = self.library.get(&reference) {
                points.extend(self.library.control_points(template));
            }
        }
        Ok(points)
    }

    /// Adds a policy's rule lists, or a single rule (`{"permission": {...}}`), to the composition.
    fn merge(policy: &mut Map<String, Value>, piece: &Value) {
        for list in RULE_LISTS {
            let incoming: Vec<Value> = match piece.get(list) {
                Some(Value::Array(rules)) => rules.clone(),
                Some(rule @ Value::Object(_)) => vec![rule.clone()],
                _ => continue,
            };
            let target = policy.entry(list).or_insert_with(|| Value::Array(Vec::new()));
            if let Value::Array(existing) = target {
                existing.extend(incoming);
            }
        }
    }
}
