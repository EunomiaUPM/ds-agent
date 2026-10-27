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

//! A resource's description as RDF: one node object in expanded JSON-LD, whatever context it
//! arrived in. Languages, datatypes and repeated values survive as they were sent.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use urn::Urn;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::entities::resource_kind::ResourceKind;
use crate::entities::vocab::Vocab;

/// One object of a triple, borrowed from the node.
#[derive(Debug, Clone, PartialEq)]
pub enum Term<'a> {
    Iri(&'a str),
    Literal(Literal<'a>),
    /// A node with no `@id` (a blank node) or one described inline.
    Node(&'a Map<String, Value>),
}

/// A literal: its lexical form plus language or datatype.
#[derive(Debug, Clone, PartialEq)]
pub struct Literal<'a> {
    pub value: &'a Value,
    pub language: Option<&'a str>,
    pub datatype: Option<&'a str>,
}

impl Literal<'_> {
    /// The lexical form: strings as they are, numbers and booleans printed.
    pub fn lexical(&self) -> String {
        match self.value {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        }
    }
}

/// The expanded node of one resource: `@id`, `@type` and one array of values per predicate.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ResourceGraph(Map<String, Value>);

impl ResourceGraph {
    /// An empty description of `id` as an instance of `class`.
    pub fn new(id: &Urn, class: &str) -> Self {
        let mut node = Map::new();
        node.insert("@id".into(), json!(id.to_string()));
        node.insert("@type".into(), json!([class]));
        Self(node)
    }

    /// Wraps an expanded node object; anything else is malformed.
    pub fn from_node(node: Value) -> Outcome<Self> {
        match node {
            Value::Object(map) => Ok(Self(map)),
            _ => Err(Errors::format(
                BadFormat::Received,
                "a resource description must be a JSON-LD node object",
                None,
            )),
        }
    }

    pub fn id(&self) -> Option<&str> {
        self.0.get("@id").and_then(Value::as_str)
    }

    pub fn set_id(&mut self, id: &str) {
        self.0.insert("@id".into(), json!(id));
    }

    pub fn types(&self) -> Vec<&str> {
        match self.0.get("@type") {
            Some(Value::Array(types)) => types.iter().filter_map(Value::as_str).collect(),
            Some(Value::String(t)) => vec![t.as_str()],
            _ => vec![],
        }
    }

    pub fn has_type(&self, class: &str) -> bool {
        self.types().contains(&class)
    }

    /// Makes `class` the first type, keeping any other the description had.
    pub fn ensure_type(&mut self, class: &str) {
        let mut types: Vec<String> = self.types().into_iter().map(str::to_string).collect();
        types.retain(|t| t != class);
        types.insert(0, class.to_string());
        self.0.insert("@type".into(), json!(types));
    }

    /// The predicates with values, never the `@` keywords.
    pub fn predicates(&self) -> impl Iterator<Item = &str> {
        self.0
            .keys()
            .map(String::as_str)
            .filter(|key| !key.starts_with('@'))
    }

    /// The raw expanded values of `predicate`.
    pub fn values(&self, predicate: &str) -> &[Value] {
        match self.0.get(predicate) {
            Some(Value::Array(values)) => values,
            _ => &[],
        }
    }

    pub fn has(&self, predicate: &str) -> bool {
        !self.values(predicate).is_empty()
    }

    pub fn count(&self, predicate: &str) -> usize {
        self.values(predicate).len()
    }

    /// The values of `predicate` as terms.
    pub fn terms(&self, predicate: &str) -> Vec<Term<'_>> {
        self.values(predicate)
            .iter()
            .filter_map(Self::term_of)
            .collect()
    }

    /// Every IRI object of `predicate`, inline nodes with an `@id` included.
    pub fn iris(&self, predicate: &str) -> Vec<&str> {
        self.values(predicate)
            .iter()
            .filter(|v| v.get("@value").is_none())
            .filter_map(|v| v.get("@id").and_then(Value::as_str))
            .collect()
    }

    pub fn first_iri(&self, predicate: &str) -> Option<&str> {
        self.iris(predicate).into_iter().next()
    }

    /// Every literal object of `predicate`.
    pub fn literals(&self, predicate: &str) -> Vec<Literal<'_>> {
        self.terms(predicate)
            .into_iter()
            .filter_map(|term| match term {
                Term::Literal(literal) => Some(literal),
                _ => None,
            })
            .collect()
    }

    /// One text of `predicate`: in the first language of `preferred` it has, else untagged, else
    /// any.
    pub fn text(&self, predicate: &str, preferred: &[&str]) -> Option<String> {
        let literals = self.literals(predicate);
        preferred
            .iter()
            .find_map(|lang| literals.iter().find(|l| l.language == Some(*lang)))
            .or_else(|| literals.iter().find(|l| l.language.is_none()))
            .or_else(|| literals.first())
            .map(Literal::lexical)
    }

    /// The literal or IRI value of `predicate` as text: what a single-valued lookup reads.
    pub fn first_value(&self, predicate: &str) -> Option<String> {
        self.terms(predicate)
            .into_iter()
            .find_map(|term| match term {
                Term::Iri(iri) => Some(iri.to_string()),
                Term::Literal(literal) => Some(literal.lexical()),
                Term::Node(_) => None,
            })
    }

    /// Replaces the values of `predicate`; no values removes it.
    pub fn put(&mut self, predicate: &str, values: Vec<Value>) {
        if values.is_empty() {
            self.0.remove(predicate);
        } else {
            self.0.insert(predicate.to_string(), Value::Array(values));
        }
    }

    pub fn push(&mut self, predicate: &str, value: Value) {
        let mut values = self.values(predicate).to_vec();
        if !values.contains(&value) {
            values.push(value);
        }
        self.put(predicate, values);
    }

    pub fn remove(&mut self, predicate: &str) {
        self.0.remove(predicate);
    }

    /// An IRI object in expanded form.
    pub fn iri(iri: &str) -> Value {
        json!({ "@id": iri })
    }

    /// A plain or language-tagged string literal in expanded form.
    pub fn literal(value: &str, language: Option<&str>) -> Value {
        match language {
            Some(lang) => json!({ "@value": value, "@language": lang }),
            None => json!({ "@value": value }),
        }
    }

    /// A typed literal in expanded form.
    pub fn typed(value: &str, datatype: &str) -> Value {
        json!({ "@value": value, "@type": datatype })
    }

    /// Inline descriptions of other catalog resources become plain references to them: each
    /// resource is described once, in its own record. A node typed as something else (a vCard
    /// contact, a period) stays inline, whatever its `@id`.
    pub fn reduce_embedded_resources(&mut self) {
        let predicates: Vec<String> = self.predicates().map(str::to_string).collect();
        for predicate in predicates {
            let reduced = self
                .values(&predicate)
                .iter()
                .map(|value| match value.get("@id").and_then(Value::as_str) {
                    Some(id) if id.starts_with("urn:") && Self::is_resource_reference(value) => {
                        Self::iri(id)
                    }
                    _ => value.clone(),
                })
                .collect();
            self.put(&predicate, reduced);
        }
    }

    /// A node with no type, or typed as one of the catalog's own resources.
    fn is_resource_reference(value: &Value) -> bool {
        let Some(node) = value.as_object() else {
            return false;
        };
        if node.contains_key("@value") {
            return false;
        }
        let types: Vec<&str> = match node.get("@type") {
            Some(Value::Array(types)) => types.iter().filter_map(Value::as_str).collect(),
            Some(Value::String(t)) => vec![t.as_str()],
            _ => vec![],
        };
        types.is_empty() || ResourceKind::from_classes(types).is_some()
    }

    /// The links to other local resources (`urn:` objects of the link predicates).
    pub fn local_links(&self) -> Vec<(&'static str, String)> {
        Vocab::RESOURCE_LINKS
            .into_iter()
            .flat_map(|predicate| {
                self.iris(predicate)
                    .into_iter()
                    .filter(|iri| iri.starts_with("urn:"))
                    .map(move |iri| (predicate, iri.to_string()))
            })
            .collect()
    }

    pub fn as_map(&self) -> &Map<String, Value> {
        &self.0
    }

    pub fn into_value(self) -> Value {
        Value::Object(self.0)
    }

    pub fn to_value(&self) -> Value {
        Value::Object(self.0.clone())
    }

    fn term_of(value: &Value) -> Option<Term<'_>> {
        let object = value.as_object()?;
        if let Some(literal) = object.get("@value") {
            return Some(Term::Literal(Literal {
                value: literal,
                language: object.get("@language").and_then(Value::as_str),
                datatype: object.get("@type").and_then(Value::as_str),
            }));
        }
        match object.get("@id").and_then(Value::as_str) {
            Some(id) if object.len() == 1 => Some(Term::Iri(id)),
            _ => Some(Term::Node(object)),
        }
    }
}
