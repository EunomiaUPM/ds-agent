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

//! Writing an expanded node with a term table. Values a term cannot carry plainly go out as
//! explicit value or node objects, under the term or under the predicate's compact IRI, so the
//! output always expands back to the same triples.

use serde_json::{json, Map, Value};

use crate::entities::jsonld::terms::{Coercion, TermDef, TermTable};

/// Compaction of expanded nodes with one term table.
pub struct Compactor<'t> {
    table: &'t TermTable,
}

impl<'t> Compactor<'t> {
    pub fn new(table: &'t TermTable) -> Self {
        Self { table }
    }

    /// The node in compact form, without `@context`.
    pub fn node(&self, node: &Map<String, Value>) -> Map<String, Value> {
        let mut out = Map::new();
        if let Some(id) = node.get("@id") {
            out.insert("@id".into(), id.clone());
        }
        if let Some(types) = node.get("@type") {
            out.insert("@type".into(), self.types(types));
        }
        for (predicate, values) in node.iter().filter(|(k, _)| !k.starts_with('@')) {
            let values = values.as_array().map(Vec::as_slice).unwrap_or(&[]);
            match self.table.term_for(predicate) {
                Some(term) => {
                    let (fits, rest) = self.with_term(term, values);
                    if let Some(fits) = fits {
                        out.insert(term.term.to_string(), fits);
                    }
                    if !rest.is_empty() {
                        out.insert(self.table.compact_iri(predicate), self.generic(&rest));
                    }
                }
                None => {
                    out.insert(self.table.compact_iri(predicate), self.generic(values));
                }
            }
        }
        out
    }

    fn types(&self, types: &Value) -> Value {
        let terms: Vec<Value> = match types {
            Value::Array(types) => types
                .iter()
                .filter_map(Value::as_str)
                .map(|t| json!(self.table.class_term(t)))
                .collect(),
            Value::String(t) => vec![json!(self.table.class_term(t))],
            _ => vec![],
        };
        match terms.len() {
            1 => terms.into_iter().next().unwrap_or(Value::Null),
            _ => Value::Array(terms),
        }
    }

    /// The values `term` carries, and those left for the predicate's IRI.
    fn with_term(&self, term: &TermDef, values: &[Value]) -> (Option<Value>, Vec<Value>) {
        if values.is_empty() {
            return (None, vec![]);
        }
        match term.coercion {
            Coercion::Language => self.language_map(values),
            _ => {
                let compacted = values.iter().map(|v| self.coerced(term, v)).collect();
                (Some(self.shape(term.set, compacted)), vec![])
            }
        }
    }

    /// A language map, or plain strings when none is tagged; anything else goes to the IRI.
    fn language_map(&self, values: &[Value]) -> (Option<Value>, Vec<Value>) {
        let plain = |v: &Value| {
            v.get("@value").is_some_and(Value::is_string) && v.get("@type").is_none()
        };
        if !values.iter().all(plain) {
            return (None, values.to_vec());
        }
        let tagged = values.iter().any(|v| v.get("@language").is_some());
        if !tagged {
            let strings: Vec<Value> = values.iter().map(|v| v["@value"].clone()).collect();
            return (Some(self.shape(false, strings)), vec![]);
        }
        let mut map: Map<String, Value> = Map::new();
        for value in values {
            let language = value
                .get("@language")
                .and_then(Value::as_str)
                .unwrap_or("@none")
                .to_string();
            let text = value["@value"].clone();
            match map.get_mut(&language) {
                Some(Value::Array(texts)) => texts.push(text),
                Some(existing) => *existing = json!([existing.clone(), text]),
                None => {
                    map.insert(language, text);
                }
            }
        }
        (Some(Value::Object(map)), vec![])
    }

    /// One value under a coerced term: plain when the coercion reads it, explicit otherwise.
    fn coerced(&self, term: &TermDef, value: &Value) -> Value {
        let iri_only = value.as_object().is_some_and(|o| o.len() == 1 && o.contains_key("@id"));
        match term.coercion {
            Coercion::Id | Coercion::Vocab if iri_only => value["@id"].clone(),
            Coercion::Typed(datatype)
                if value.get("@type").and_then(Value::as_str) == Some(datatype) =>
            {
                value["@value"].clone()
            }
            Coercion::Json if value.get("@type").and_then(Value::as_str) == Some("@json") => {
                value["@value"].clone()
            }
            Coercion::None if Self::plain_string(value) => value["@value"].clone(),
            _ => self.explicit(value),
        }
    }

    /// Values under an uncoerced key: strings when plain, objects otherwise.
    fn generic(&self, values: &[Value]) -> Value {
        let compacted = values
            .iter()
            .map(|value| {
                if Self::plain_string(value) {
                    value["@value"].clone()
                } else {
                    self.explicit(value)
                }
            })
            .collect();
        self.shape(false, compacted)
    }

    /// A value or node object spelled out, its datatype compacted.
    fn explicit(&self, value: &Value) -> Value {
        let Some(object) = value.as_object() else {
            return value.clone();
        };
        if object.contains_key("@value") {
            let mut literal = object.clone();
            if let Some(datatype) = object.get("@type").and_then(Value::as_str) {
                if datatype != "@json" {
                    literal.insert("@type".into(), json!(self.table.compact_iri(datatype)));
                }
            }
            return Value::Object(literal);
        }
        Value::Object(self.node(object))
    }

    fn shape(&self, set: bool, values: Vec<Value>) -> Value {
        if !set && values.len() == 1 {
            values.into_iter().next().unwrap_or(Value::Null)
        } else {
            Value::Array(values)
        }
    }

    fn plain_string(value: &Value) -> bool {
        value.get("@value").is_some_and(Value::is_string)
            && value.get("@type").is_none()
            && value.get("@language").is_none()
    }
}
