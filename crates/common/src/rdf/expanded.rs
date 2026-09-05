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

//! Reading an expanded JSON-LD document: node lookup by type, reference
//! resolution, value access. Protocol-agnostic — term IRIs come from the caller.

use std::collections::HashMap;

use serde_json::Value;
use ymir::errors::{BadFormat, Errors, Outcome};

/// An expanded JSON-LD document, indexed by `@id` so bare references resolve.
pub struct ExpandedDoc<'a> {
    nodes: &'a [Value],
    by_id: HashMap<&'a str, &'a Value>,
}

impl<'a> ExpandedDoc<'a> {
    /// `None` if the value is not the top-level array expansion produces.
    pub fn new(expanded: &'a Value) -> Option<Self> {
        let nodes = expanded.as_array()?;
        let mut by_id = HashMap::new();
        for node in nodes {
            Self::index(node, &mut by_id);
        }
        Some(Self { nodes, by_id })
    }

    /// Record every node carrying an `@id`, recursing into inlined ones. A bare
    /// reference never shadows the node that holds the data.
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

    pub fn nodes(&self) -> impl Iterator<Item = Node<'_, 'a>> {
        self.nodes
            .iter()
            .map(move |value| Node { doc: self, value })
    }

    /// Top-level nodes carrying `type_iri`. A message expands to exactly one;
    /// anything else is the caller's error to report.
    pub fn nodes_of_type<'s>(
        &'s self,
        type_iri: &'s str,
    ) -> impl Iterator<Item = Node<'s, 'a>> + 's {
        self.nodes().filter(move |n| n.has_type(type_iri))
    }

    /// Bind the one node carrying `type_iri`. A nested message and an `@graph` of
    /// mutually-referencing nodes expand alike, so type is the only reliable handle.
    pub fn root_message_node<'s>(
        &'s self,
        type_iri: &'s str,
        message: &impl std::fmt::Display,
    ) -> Outcome<Node<'s, 'a>> {
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

    /// Follow a bare `{"@id": …}` to the node holding the data, when it is in
    /// this document.
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

/// A node object, bound to the document so its references can be followed.
#[derive(Clone, Copy)]
pub struct Node<'d, 'a> {
    doc: &'d ExpandedDoc<'a>,
    value: &'a Value,
}

impl<'d, 'a> Node<'d, 'a> {
    pub fn id(&self) -> Option<&'a str> {
        self.value.get("@id").and_then(Value::as_str)
    }

    pub fn types(&self) -> impl Iterator<Item = &'a str> {
        self.value
            .get("@type")
            .and_then(Value::as_array)
            .map(|a| a.as_slice())
            .unwrap_or_default()
            .iter()
            .filter_map(Value::as_str)
    }

    pub fn has_type(&self, type_iri: &str) -> bool {
        self.types().any(|t| t == type_iri)
    }

    /// Every value of a predicate. Expansion always yields an array, so cardinality
    /// is only visible here — extraction narrows it to one.
    pub fn values(&self, predicate: &str) -> impl Iterator<Item = &'a Value> {
        self.value
            .get(predicate)
            .and_then(Value::as_array)
            .map(|a| a.as_slice())
            .unwrap_or_default()
            .iter()
    }

    pub fn count(&self, predicate: &str) -> usize {
        self.values(predicate).count()
    }

    /// Read a single-valued predicate, `@id` or `@value` alike. Which one a term
    /// must be is a validation rule, not a read.
    pub fn iri_or_literal(&self, predicate: &str) -> Option<&'a str> {
        let entry = self.values(predicate).next()?;
        entry
            .get("@id")
            .or_else(|| entry.get("@value"))
            .and_then(Value::as_str)
    }

    /// The nodes a predicate points at, references followed.
    pub fn objects(&self, predicate: &str) -> impl Iterator<Item = Node<'d, 'a>> {
        let doc = self.doc;
        self.values(predicate).map(move |v| Node {
            doc,
            value: doc.resolve(v),
        })
    }

    pub fn object(&self, predicate: &str) -> Option<Node<'d, 'a>> {
        self.objects(predicate).next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const MSG: &str = "https://example.org/Message";
    const ADDR: &str = "https://example.org/address";
    const PORT: &str = "https://example.org/port";
    const PID: &str = "https://example.org/pid";

    fn graph_form() -> Value {
        json!([
            {"@id": "_:msg", "@type": [MSG],
             PID: [{"@id": "urn:uuid:cc"}],
             ADDR: [{"@id": "_:a"}]},
            {"@id": "_:a", PORT: [{"@value": "8080"}]}
        ])
    }

    #[test]
    fn finds_the_node_by_type() {
        let g = graph_form();
        let doc = ExpandedDoc::new(&g).unwrap();
        assert_eq!(doc.nodes_of_type(MSG).count(), 1);
        assert_eq!(doc.nodes_of_type("https://example.org/Other").count(), 0);
    }

    #[test]
    fn follows_a_bare_reference_to_the_node_holding_the_data() {
        let g = graph_form();
        let doc = ExpandedDoc::new(&g).unwrap();
        let msg = doc.nodes_of_type(MSG).next().unwrap();
        let addr = msg.object(ADDR).expect("address resolves");
        assert_eq!(addr.iri_or_literal(PORT), Some("8080"));
    }

    #[test]
    fn reads_a_pid_whether_it_is_an_id_or_a_literal() {
        let g = graph_form();
        let doc = ExpandedDoc::new(&g).unwrap();
        let msg = doc.nodes_of_type(MSG).next().unwrap();
        assert_eq!(msg.iri_or_literal(PID), Some("urn:uuid:cc"));

        let literal = json!([{"@type": [MSG], PID: [{"@value": "urn:uuid:cc"}]}]);
        let doc = ExpandedDoc::new(&literal).unwrap();
        let msg = doc.nodes_of_type(MSG).next().unwrap();
        assert_eq!(msg.iri_or_literal(PID), Some("urn:uuid:cc"));
    }

    /// Cardinality is only visible here: extraction narrows to the first value.
    #[test]
    fn counts_the_values_of_a_term() {
        let doubled = json!([{
            "@type": [MSG],
            PID: [{"@value": "urn:uuid:a"}, {"@value": "urn:uuid:b"}]
        }]);
        let doc = ExpandedDoc::new(&doubled).unwrap();
        let msg = doc.nodes_of_type(MSG).next().unwrap();
        assert_eq!(msg.count(PID), 2);
        assert_eq!(msg.iri_or_literal(PID), Some("urn:uuid:a"));
    }

    #[test]
    fn a_non_array_expansion_is_rejected() {
        assert!(ExpandedDoc::new(&json!({"@type": [MSG]})).is_none());
    }
}
