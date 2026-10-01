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

//! Reading expanded JSON-LD documents, node lookups, and typed extraction getters.

use std::collections::HashMap;

use serde_json::Value;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::rdf::traits::FromRdf;

/// An expanded JSON-LD document indexed by `@id` for resolving bare references.
pub struct ExpandedDoc<'a> {
    nodes: &'a [Value],
    by_id: HashMap<&'a str, &'a Value>,
}

impl<'a> ExpandedDoc<'a> {
    /// Constructs an index from an expanded JSON-LD top-level array.
    pub fn new(expanded: &'a Value) -> Option<Self> {
        let nodes = expanded.as_array()?;
        let mut by_id = HashMap::new();
        for node in nodes {
            Self::index(node, &mut by_id);
        }
        Some(Self { nodes, by_id })
    }

    /// Indexes nodes by `@id`, recursing into inlined sub-objects.
    fn index(node: &'a Value, by_id: &mut HashMap<&'a str, &'a Value>) {
        let Some(object) = node.as_object() else {
            return;
        };
        if let Some(id) = object.get("@id").and_then(Value::as_str) {
            if object.len() > 1 || !by_id.contains_key(id) {
                by_id.insert(id, node);
            }
        }
        for (key, value) in object {
            if key.starts_with('@') {
                continue;
            }
            for entry in value.as_array().map(|a| a.as_slice()).unwrap_or_default() {
                Self::index(entry, by_id);
            }
        }
    }

    /// Iterates over all top-level nodes in the document.
    pub fn nodes(&self) -> impl Iterator<Item = RdfNode<'_, 'a>> {
        self.nodes
            .iter()
            .map(move |value| RdfNode { doc: self, value })
    }

    /// Filters top-level nodes matching a specific `@type` IRI.
    pub fn nodes_of_type<'s>(
        &'s self,
        type_iri: &'s str,
    ) -> impl Iterator<Item = RdfNode<'s, 'a>> + 's {
        self.nodes().filter(move |n| n.has_type(type_iri))
    }

    /// Resolves the root node of the document for target type `T`.
    pub fn root_node<T: FromRdf>(&self) -> Outcome<RdfNode<'_, 'a>> {
        match T::TYPE_IRI {
            Some(type_iri) => {
                let mut matching = self.nodes_of_type(type_iri);
                let node = matching.next().ok_or_else(|| {
                    Errors::format(
                        BadFormat::Received,
                        format!("expanded document contains no node with type <{type_iri}>"),
                        None,
                    )
                })?;
                if matching.next().is_some() {
                    return Err(Errors::format(
                        BadFormat::Received,
                        format!("expanded document contains multiple nodes with type <{type_iri}>"),
                        None,
                    ));
                }
                Ok(node)
            }
            None => {
                if self.nodes.len() == 1 {
                    Ok(RdfNode {
                        doc: self,
                        value: &self.nodes[0],
                    })
                } else if self.nodes.is_empty() {
                    Err(Errors::format(
                        BadFormat::Received,
                        "expanded document contains no nodes",
                        None,
                    ))
                } else {
                    Err(Errors::format(
                        BadFormat::Received,
                        "expanded document contains multiple nodes but no TYPE_IRI was declared",
                        None,
                    ))
                }
            }
        }
    }

    /// Legacy helper: resolves a single root node matching `type_iri`.
    pub fn root_message_node<'s>(
        &'s self,
        type_iri: &'s str,
        message: &impl std::fmt::Display,
    ) -> Outcome<RdfNode<'s, 'a>> {
        let mut matching = self.nodes_of_type(type_iri);
        let node = matching.next().ok_or_else(|| {
            Errors::format(
                BadFormat::Received,
                format!("expanded message has no {message} node"),
                None,
            )
        })?;
        if matching.next().is_some() {
            return Err(Errors::format(
                BadFormat::Received,
                format!("expanded message has more than one {message} node"),
                None,
            ));
        }
        Ok(node)
    }

    /// Resolves a bare `{"@id": ...}` reference to its full node definition.
    fn resolve(&self, value: &'a Value) -> &'a Value {
        let is_reference = value
            .as_object()
            .is_some_and(|o| o.len() == 1 && o.contains_key("@id"));
        if !is_reference {
            return value;
        }
        value
            .get("@id")
            .and_then(Value::as_str)
            .and_then(|id| self.by_id.get(id).copied())
            .unwrap_or(value)
    }
}

/// A node in an expanded document with rich typed getters for properties.
#[derive(Clone, Copy)]
pub struct RdfNode<'d, 'a> {
    doc: &'d ExpandedDoc<'a>,
    value: &'a Value,
}

impl<'d, 'a> RdfNode<'d, 'a> {
    /// Returns the node's `@id` identifier, if declared.
    pub fn id(&self) -> Option<&'a str> {
        self.value.get("@id").and_then(Value::as_str)
    }

    /// Iterates over all `@type` IRIs associated with this node.
    pub fn types(&self) -> impl Iterator<Item = &'a str> {
        self.value
            .get("@type")
            .and_then(Value::as_array)
            .map(|a| a.as_slice())
            .unwrap_or_default()
            .iter()
            .filter_map(Value::as_str)
    }

    /// Checks if this node carries the given `@type` IRI.
    pub fn has_type(&self, type_iri: &str) -> bool {
        self.types().any(|t| t == type_iri)
    }

    /// Iterates over expanded JSON values for a predicate.
    pub fn values(&self, predicate: &str) -> impl Iterator<Item = &'a Value> {
        self.value
            .get(predicate)
            .and_then(Value::as_array)
            .map(|a| a.as_slice())
            .unwrap_or_default()
            .iter()
    }

    /// Returns the number of values associated with a predicate.
    pub fn count(&self, predicate: &str) -> usize {
        self.values(predicate).count()
    }

    /// Returns the first expanded JSON value associated with a predicate.
    pub fn value(&self, predicate: &str) -> Option<&'a Value> {
        self.values(predicate).next()
    }

    /// Reads a single-valued predicate, resolving `@id` or `@value`.
    pub fn iri_or_literal(&self, predicate: &str) -> Option<&'a str> {
        let entry = self.value(predicate)?;
        entry
            .get("@id")
            .or_else(|| entry.get("@value"))
            .and_then(Value::as_str)
    }

    /// Iterates over object nodes pointed to by a predicate, with references followed.
    pub fn objects(&self, predicate: &str) -> impl Iterator<Item = RdfNode<'d, 'a>> {
        let doc = self.doc;
        self.values(predicate).map(move |v| RdfNode {
            doc,
            value: doc.resolve(v),
        })
    }

    /// Returns the first object node pointed to by a predicate.
    pub fn object(&self, predicate: &str) -> Option<RdfNode<'d, 'a>> {
        self.objects(predicate).next()
    }

    /// Builds a formatted error for a missing mandatory predicate.
    pub fn missing_field(&self, predicate: &str) -> Errors {
        let id = self.id().unwrap_or("_:blank");
        Errors::format(
            BadFormat::Received,
            format!("missing required RDF predicate <{predicate}> on node '{id}'"),
            None,
        )
    }

    /// Builds a formatted error for an invalid predicate value.
    pub fn invalid_field(&self, predicate: &str, reason: &str) -> Errors {
        let id = self.id().unwrap_or("_:blank");
        Errors::format(
            BadFormat::Received,
            format!("invalid value for predicate <{predicate}> on node '{id}': {reason}"),
            None,
        )
    }

    /// Extracts a required string slice (`@value` or `@id`).
    pub fn get_str(&self, predicate: &str) -> Outcome<&'a str> {
        self.iri_or_literal(predicate)
            .ok_or_else(|| self.missing_field(predicate))
    }

    /// Extracts an owned `String` from a required predicate.
    pub fn get_string(&self, predicate: &str) -> Outcome<String> {
        self.get_str(predicate).map(str::to_string)
    }

    /// Extracts an optional string slice from a predicate.
    pub fn get_opt_str(&self, predicate: &str) -> Outcome<Option<&'a str>> {
        Ok(self.iri_or_literal(predicate))
    }

    /// Extracts an optional owned `String` from a predicate.
    pub fn get_opt_string(&self, predicate: &str) -> Outcome<Option<String>> {
        Ok(self.iri_or_literal(predicate).map(str::to_string))
    }

    /// Extracts a required `@id` IRI string slice.
    pub fn get_iri(&self, predicate: &str) -> Outcome<&'a str> {
        let entry = self
            .value(predicate)
            .ok_or_else(|| self.missing_field(predicate))?;
        entry
            .get("@id")
            .and_then(Value::as_str)
            .ok_or_else(|| self.invalid_field(predicate, "expected @id IRI"))
    }

    /// Extracts an optional `@id` IRI string slice.
    pub fn get_opt_iri(&self, predicate: &str) -> Outcome<Option<&'a str>> {
        match self.value(predicate) {
            Some(entry) => {
                let iri = entry
                    .get("@id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| self.invalid_field(predicate, "expected @id IRI"))?;
                Ok(Some(iri))
            }
            None => Ok(None),
        }
    }

    /// Extracts a required `@id` IRI as an owned `String`.
    pub fn get_iri_string(&self, predicate: &str) -> Outcome<String> {
        self.get_iri(predicate).map(str::to_string)
    }

    /// Extracts an optional `@id` IRI as an owned `String`.
    pub fn get_opt_iri_string(&self, predicate: &str) -> Outcome<Option<String>> {
        self.get_opt_iri(predicate).map(|o| o.map(str::to_string))
    }

    /// Extracts a required `u64` numeric literal.
    pub fn get_u64(&self, predicate: &str) -> Outcome<u64> {
        let entry = self
            .value(predicate)
            .ok_or_else(|| self.missing_field(predicate))?;
        if let Some(val) = entry.get("@value") {
            if let Some(n) = val.as_u64() {
                return Ok(n);
            }
            if let Some(s) = val.as_str() {
                return s
                    .parse::<u64>()
                    .map_err(|e| self.invalid_field(predicate, &format!("cannot parse u64: {e}")));
            }
        }
        Err(self.invalid_field(predicate, "expected u64 numeric value"))
    }

    /// Extracts an optional `u64` numeric literal.
    pub fn get_opt_u64(&self, predicate: &str) -> Outcome<Option<u64>> {
        match self.value(predicate) {
            Some(_) => Ok(Some(self.get_u64(predicate)?)),
            None => Ok(None),
        }
    }

    /// Extracts a required `i64` numeric literal.
    pub fn get_i64(&self, predicate: &str) -> Outcome<i64> {
        let entry = self
            .value(predicate)
            .ok_or_else(|| self.missing_field(predicate))?;
        if let Some(val) = entry.get("@value") {
            if let Some(n) = val.as_i64() {
                return Ok(n);
            }
            if let Some(s) = val.as_str() {
                return s
                    .parse::<i64>()
                    .map_err(|e| self.invalid_field(predicate, &format!("cannot parse i64: {e}")));
            }
        }
        Err(self.invalid_field(predicate, "expected i64 numeric value"))
    }

    /// Extracts an optional `i64` numeric literal.
    pub fn get_opt_i64(&self, predicate: &str) -> Outcome<Option<i64>> {
        match self.value(predicate) {
            Some(_) => Ok(Some(self.get_i64(predicate)?)),
            None => Ok(None),
        }
    }

    /// Extracts a required boolean literal.
    pub fn get_bool(&self, predicate: &str) -> Outcome<bool> {
        let entry = self
            .value(predicate)
            .ok_or_else(|| self.missing_field(predicate))?;
        if let Some(val) = entry.get("@value") {
            if let Some(b) = val.as_bool() {
                return Ok(b);
            }
            if let Some(s) = val.as_str() {
                return s.parse::<bool>().map_err(|e| {
                    self.invalid_field(predicate, &format!("cannot parse bool: {e}"))
                });
            }
        }
        Err(self.invalid_field(predicate, "expected boolean value"))
    }

    /// Extracts an optional boolean literal.
    pub fn get_opt_bool(&self, predicate: &str) -> Outcome<Option<bool>> {
        match self.value(predicate) {
            Some(_) => Ok(Some(self.get_bool(predicate)?)),
            None => Ok(None),
        }
    }

    /// Extracts all literal or IRI string values for a predicate.
    pub fn get_strings(&self, predicate: &str) -> Vec<String> {
        self.values(predicate)
            .filter_map(|entry| {
                entry
                    .get("@value")
                    .or_else(|| entry.get("@id"))
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
            .collect()
    }

    /// Extracts all `@id` IRI values for a predicate.
    pub fn get_iris(&self, predicate: &str) -> Vec<String> {
        self.values(predicate)
            .filter_map(|entry| entry.get("@id").and_then(Value::as_str).map(str::to_string))
            .collect()
    }

    /// Extracts a mandatory child object deserialized via `FromRdf`.
    pub fn get_object<T: FromRdf>(&self, predicate: &str) -> Outcome<T> {
        let node = self
            .object(predicate)
            .ok_or_else(|| self.missing_field(predicate))?;
        T::from_rdf(&node)
    }

    /// Extracts an optional child object deserialized via `FromRdf`.
    pub fn get_opt_object<T: FromRdf>(&self, predicate: &str) -> Outcome<Option<T>> {
        match self.object(predicate) {
            Some(node) => Ok(Some(T::from_rdf(&node)?)),
            None => Ok(None),
        }
    }

    /// Extracts a list of child objects deserialized via `FromRdf`.
    pub fn get_list<T: FromRdf>(&self, predicate: &str) -> Outcome<Vec<T>> {
        self.objects(predicate)
            .map(|node| T::from_rdf(&node))
            .collect::<Outcome<Vec<T>>>()
    }
}
