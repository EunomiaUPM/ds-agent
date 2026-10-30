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

//! Walks template content by position: lists, checks and expands its references.

use std::collections::HashMap;

use serde_json::{Map, Value};
use ymir::errors::{Errors, Outcome};

use crate::entities::policy_composition::{
    Slot, TemplateLibrary, TemplateRef, INCLUDE, REF, RULE_LISTS, WITH,
};
use crate::entities::policy_templates::placeholder::Placeholder;
use crate::entities::policy_templates::types::ParameterDefinition;
use crate::entities::policy_templates::{PolicyTemplateDto, PolicyTemplateKind};

/// Operators of a logical constraint, whose operands are constraints.
const LOGICAL: [&str; 4] = ["and", "or", "xone", "andSequence"];

/// What the walk does at each reference: replace it with a value.
type OnRef<'f> = dyn FnMut(&TemplateRef, Slot, &str) -> Outcome<Value> + 'f;

/// Flattens template content over a library holding every version it references.
pub struct CompositionResolver<'a> {
    library: &'a TemplateLibrary,
}

impl<'a> CompositionResolver<'a> {
    /// Deepest chain of references a composition may have.
    pub const MAX_DEPTH: usize = 8;

    pub fn new(library: &'a TemplateLibrary) -> Self {
        Self { library }
    }

    /// The references written directly in the content, with where each sits; a reference or an
    /// `@include` out of place fails.
    pub fn references(kind: PolicyTemplateKind, content: &Value) -> Outcome<Vec<(TemplateRef, Slot)>> {
        let mut found = Vec::new();
        Self::walk(kind, content, &mut |reference, slot, _| {
            found.push((reference.clone(), slot));
            Ok(Self::placeholder_for(slot))
        })?;
        Ok(found)
    }

    /// Checks a template about to be stored: each direct reference names a visible version of
    /// the right kind, binds every parameter it must and only parameters that exist, and binds
    /// parent values only to parameters the parent declares. The chain stays within depth.
    pub fn check(
        &self,
        kind: PolicyTemplateKind,
        content: &Value,
        parameters: &HashMap<String, ParameterDefinition>,
    ) -> Outcome<()> {
        for (reference, slot) in Self::references(kind, content)? {
            let child = self.child(&reference, slot)?;
            for (name, value) in &reference.with {
                if !child.parameters.contains_key(name) {
                    return Err(Errors::validation(
                        format!("{} has no parameter '{name}'", reference.name()),
                        None,
                    ));
                }
                let forwarded = value.as_str().and_then(Placeholder::name);
                if let Some(parent) = forwarded.filter(|p| !parameters.contains_key(*p)) {
                    return Err(Errors::validation(
                        format!("'{parent}' bound to {} is not a parameter of this template", reference.name()),
                        None,
                    ));
                }
            }
            if let Some((name, _)) = child
                .parameters
                .iter()
                .find(|(name, def)| def.default_value.is_none() && !reference.with.contains_key(*name))
            {
                return Err(Errors::validation(
                    format!("{} needs '{name}' bound in {WITH}", reference.name()),
                    None,
                ));
            }
            if 1 + self.depth(child, 1)? > Self::MAX_DEPTH {
                return Err(Errors::validation(
                    format!("references nest deeper than {}", Self::MAX_DEPTH),
                    None,
                ));
            }
        }
        Ok(())
    }

    /// The template's content filled with `values` (already resolved with
    /// [`PolicyTemplateDto::resolve_values`]) and every reference replaced by its expansion.
    pub fn expand(&self, template: &PolicyTemplateDto, values: &HashMap<String, Value>) -> Outcome<Value> {
        let mut stack = vec![template.reference()];
        self.expand_content(template.kind, &template.content, values, &mut stack)
    }

    /// What a reference with literal values flattens to, and the template it names.
    pub fn expand_reference(&self, reference: &TemplateRef) -> Outcome<(&'a PolicyTemplateDto, Value)> {
        let template = self.library.get(reference)?;
        let values = template.resolve_values(&reference.with)?;
        Ok((template, self.expand(template, &values)?))
    }

    /// Content of an ad hoc policy (no parameters of its own) with its references expanded.
    pub fn expand_policy(&self, content: &Value) -> Outcome<Value> {
        let mut stack = Vec::new();
        self.expand_content(PolicyTemplateKind::Policy, content, &HashMap::new(), &mut stack)
    }

    fn expand_content(
        &self,
        kind: PolicyTemplateKind,
        content: &Value,
        values: &HashMap<String, Value>,
        stack: &mut Vec<String>,
    ) -> Outcome<Value> {
        let mut filled = content.clone();
        Placeholder::fill(&mut filled, values);
        Self::walk(kind, &filled, &mut |reference, slot, path| {
            let child = self.child(reference, slot)?;
            if stack.contains(&reference.name()) {
                return Err(Errors::validation(format!("{path}: {} references itself", reference.name()), None));
            }
            if stack.len() > Self::MAX_DEPTH {
                return Err(Errors::validation(
                    format!("{path}: references nest deeper than {}", Self::MAX_DEPTH),
                    None,
                ));
            }
            let child_values = child.resolve_values(&reference.with)?;
            stack.push(reference.name());
            let expanded = self.expand_content(child.kind, &child.content, &child_values, stack);
            stack.pop();
            let expanded = expanded?;
            Ok(match slot {
                Slot::Rule(list) => expanded.get(list).cloned().unwrap_or(Value::Null),
                _ => expanded,
            })
        })
    }

    /// The referenced template, if it is of the kind the slot takes.
    fn child(&self, reference: &TemplateRef, slot: Slot) -> Outcome<&'a PolicyTemplateDto> {
        let child = self.library.get(reference)?;
        if child.kind != slot.kind() {
            return Err(Errors::validation(
                format!("{} is a {}, but a {} goes here", reference.name(), child.kind, slot.kind()),
                None,
            ));
        }
        if let Slot::Rule(list) = slot {
            if child.content.get(list).is_none() {
                return Err(Errors::validation(
                    format!("{} is not a {list} rule", reference.name()),
                    None,
                ));
            }
        }
        Ok(child)
    }

    /// Longest chain of references below a template.
    fn depth(&self, template: &PolicyTemplateDto, level: usize) -> Outcome<usize> {
        if level > Self::MAX_DEPTH {
            return Ok(level);
        }
        let mut deepest = 0;
        for (reference, _) in Self::references(template.kind, &template.content)? {
            let child = self.library.get(&reference)?;
            deepest = deepest.max(1 + self.depth(child, level + 1)?);
        }
        Ok(deepest)
    }

    /// What stands in for a reference when only listing them.
    fn placeholder_for(slot: Slot) -> Value {
        match slot {
            Slot::Include => Value::Object(Map::new()),
            _ => Value::Null,
        }
    }

    /// The content with each reference replaced by what `on_ref` returns.
    fn walk(kind: PolicyTemplateKind, content: &Value, on_ref: &mut OnRef<'_>) -> Outcome<Value> {
        match kind {
            PolicyTemplateKind::Constraint => Self::walk_constraint(content, "content", on_ref),
            PolicyTemplateKind::Rule => Self::walk_rule_content(content, on_ref),
            PolicyTemplateKind::Policy => Self::walk_policy(content, "content", on_ref),
        }
    }

    fn walk_policy(content: &Value, path: &str, on_ref: &mut OnRef<'_>) -> Outcome<Value> {
        let object = Self::object(content, path)?;
        let mut out = Map::new();
        let mut included = Vec::new();
        for (key, value) in object {
            let at = format!("{path}.{key}");
            if let Some(list) = RULE_LISTS.iter().find(|l| *l == key) {
                let rules = Self::array(value, &at)?;
                let mut walked = Vec::with_capacity(rules.len());
                for (i, rule) in rules.iter().enumerate() {
                    walked.push(Self::walk_rule(rule, list, &format!("{at}[{i}]"), on_ref)?);
                }
                out.insert(key.clone(), Value::Array(walked));
            } else if key == INCLUDE {
                for (i, item) in Self::array(value, &at)?.iter().enumerate() {
                    let item_path = format!("{at}[{i}]");
                    let reference = TemplateRef::parse(Self::object(item, &item_path)?, &[], &item_path)?;
                    included.push(on_ref(&reference, Slot::Include, &item_path)?);
                }
            } else {
                Self::forbid_refs(value, &at)?;
                out.insert(key.clone(), value.clone());
            }
        }
        for policy in included {
            for list in RULE_LISTS {
                if let Some(Value::Array(rules)) = policy.get(list) {
                    let target = out.entry(list).or_insert_with(|| Value::Array(Vec::new()));
                    if let Value::Array(existing) = target {
                        existing.extend(rules.iter().cloned());
                    }
                }
            }
        }
        Ok(Value::Object(out))
    }

    /// Rule content is an object with one rule list name and one rule.
    fn walk_rule_content(content: &Value, on_ref: &mut OnRef<'_>) -> Outcome<Value> {
        let object = Self::object(content, "content")?;
        let mut keys = object.keys();
        let (Some(key), None) = (keys.next(), keys.next()) else {
            return Err(Errors::validation(
                "content: a rule is an object with exactly one of permission, prohibition or obligation",
                None,
            ));
        };
        let Some(list) = RULE_LISTS.iter().find(|l| *l == key) else {
            return Err(Errors::validation(format!("content: '{key}' is not a rule list"), None));
        };
        let rule = Self::walk_rule(&object[key.as_str()], list, &format!("content.{key}"), on_ref)?;
        let mut out = Map::new();
        out.insert(key.clone(), rule);
        Ok(Value::Object(out))
    }

    fn walk_rule(rule: &Value, list: &'static str, path: &str, on_ref: &mut OnRef<'_>) -> Outcome<Value> {
        if TemplateRef::is_ref(rule) {
            let reference = TemplateRef::parse(Self::object(rule, path)?, &[], path)?;
            return on_ref(&reference, Slot::Rule(list), path);
        }
        let object = Self::object(rule, path)?;
        let mut out = Map::new();
        for (key, value) in object {
            let at = format!("{path}.{key}");
            let walked = match key.as_str() {
                "constraint" => Self::walk_constraints(value, &at, on_ref)?,
                "duty" => Self::walk_duties(value, &at, on_ref)?,
                _ => {
                    Self::forbid_refs(value, &at)?;
                    value.clone()
                }
            };
            out.insert(key.clone(), walked);
        }
        Ok(Value::Object(out))
    }

    /// A duty is a rule body without references of its own; its constraints may have them.
    fn walk_duties(value: &Value, path: &str, on_ref: &mut OnRef<'_>) -> Outcome<Value> {
        let walk_one = |duty: &Value, at: &str, on_ref: &mut OnRef<'_>| -> Outcome<Value> {
            let object = Self::object(duty, at)?;
            let mut out = Map::new();
            for (key, value) in object {
                let inner = format!("{at}.{key}");
                let walked = if key == "constraint" {
                    Self::walk_constraints(value, &inner, on_ref)?
                } else {
                    Self::forbid_refs(value, &inner)?;
                    value.clone()
                };
                out.insert(key.clone(), walked);
            }
            Ok(Value::Object(out))
        };
        match value {
            Value::Array(duties) => duties
                .iter()
                .enumerate()
                .map(|(i, duty)| walk_one(duty, &format!("{path}[{i}]"), on_ref))
                .collect::<Outcome<Vec<_>>>()
                .map(Value::Array),
            other => walk_one(other, path, on_ref),
        }
    }

    fn walk_constraints(value: &Value, path: &str, on_ref: &mut OnRef<'_>) -> Outcome<Value> {
        Self::array(value, path)?
            .iter()
            .enumerate()
            .map(|(i, constraint)| Self::walk_constraint(constraint, &format!("{path}[{i}]"), on_ref))
            .collect::<Outcome<Vec<_>>>()
            .map(Value::Array)
    }

    fn walk_constraint(constraint: &Value, path: &str, on_ref: &mut OnRef<'_>) -> Outcome<Value> {
        if TemplateRef::is_ref(constraint) {
            let reference = TemplateRef::parse(Self::object(constraint, path)?, &[], path)?;
            return on_ref(&reference, Slot::Constraint, path);
        }
        let object = Self::object(constraint, path)?;
        let mut out = Map::new();
        for (key, value) in object {
            let at = format!("{path}.{key}");
            let walked = if LOGICAL.contains(&key.as_str()) {
                Self::walk_constraints(value, &at, on_ref)?
            } else {
                Self::forbid_refs(value, &at)?;
                value.clone()
            };
            out.insert(key.clone(), walked);
        }
        Ok(Value::Object(out))
    }

    /// Fails when a reference or an `@include` sits where no template can go.
    fn forbid_refs(value: &Value, path: &str) -> Outcome<()> {
        match value {
            Value::Object(object) => {
                if let Some(key) = [REF, INCLUDE].into_iter().find(|k| object.contains_key(*k)) {
                    return Err(Errors::validation(format!("{path}: {key} cannot go here"), None));
                }
                object.iter().try_for_each(|(k, v)| Self::forbid_refs(v, &format!("{path}.{k}")))
            }
            Value::Array(items) => items
                .iter()
                .enumerate()
                .try_for_each(|(i, v)| Self::forbid_refs(v, &format!("{path}[{i}]"))),
            _ => Ok(()),
        }
    }

    fn object<'v>(value: &'v Value, path: &str) -> Outcome<&'v Map<String, Value>> {
        value
            .as_object()
            .ok_or_else(|| Errors::validation(format!("{path} must be an object"), None))
    }

    fn array<'v>(value: &'v Value, path: &str) -> Outcome<&'v Vec<Value>> {
        value
            .as_array()
            .ok_or_else(|| Errors::validation(format!("{path} must be a list"), None))
    }
}
