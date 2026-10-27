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

//! An RDF graph as a set of triples, with the subjects in the order they first appear so that
//! whatever is built from it is stable. It reads JSON-LD through an [`RdfEngine`], so contexts
//! resolve as in every other expansion, and Turtle, RDF/XML and N-Triples directly.

use std::collections::{HashMap, HashSet};

use oxrdf::vocab::{rdf, xsd};
use oxrdf::{
    BlankNode, Graph, Literal, NamedNode, NamedNodeRef, NamedOrBlankNode, NamedOrBlankNodeRef,
    Term, TermRef, Triple, TripleRef,
};
use serde_json::{json, Map, Value};
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::rdf::engine::RdfEngine;
use crate::rdf::syntax::RdfSyntax;

/// Triples plus the order their subjects appeared in.
#[derive(Debug, Clone, Default)]
pub struct RdfGraph {
    graph: Graph,
    subjects: Vec<NamedOrBlankNode>,
    seen: HashSet<NamedOrBlankNode>,
}

impl RdfGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reads `body` in `syntax`, JSON-LD expanded by `engine`; anything malformed is a 400
    /// naming what failed.
    pub async fn parse(syntax: RdfSyntax, body: &[u8], engine: &RdfEngine) -> Outcome<Self> {
        let mut graph = Self::new();
        match syntax {
            RdfSyntax::JsonLd => {
                let document: Value = serde_json::from_slice(body)
                    .map_err(|e| Self::malformed(format!("the body is not JSON: {e}")))?;
                let expansion = engine.expand(&document).await?;
                graph.add_expanded(&expansion.expanded)?;
            }
            RdfSyntax::Turtle => {
                for triple in oxttl::TurtleParser::new().for_slice(body) {
                    graph.insert(
                        triple
                            .map_err(|e| Self::malformed(format!("Turtle: {e}")))?
                            .as_ref(),
                    );
                }
            }
            RdfSyntax::NTriples => {
                for triple in oxttl::NTriplesParser::new().for_slice(body) {
                    graph.insert(
                        triple
                            .map_err(|e| Self::malformed(format!("N-Triples: {e}")))?
                            .as_ref(),
                    );
                }
            }
            RdfSyntax::RdfXml => {
                for triple in oxrdfxml::RdfXmlParser::new().for_slice(body) {
                    graph.insert(
                        triple
                            .map_err(|e| Self::malformed(format!("RDF/XML: {e}")))?
                            .as_ref(),
                    );
                }
            }
        }
        Ok(graph)
    }

    /// The triples of an expanded JSON-LD document (one node or an array of them).
    pub fn from_expanded(expanded: &Value) -> Outcome<Self> {
        let mut graph = Self::new();
        graph.add_expanded(expanded)?;
        Ok(graph)
    }

    /// The triples of expanded node objects; `_:` labels are shared across them.
    pub fn from_nodes<'a>(
        nodes: impl IntoIterator<Item = &'a Map<String, Value>>,
    ) -> Outcome<Self> {
        let mut graph = Self::new();
        let mut blanks = HashMap::new();
        for node in nodes {
            graph.add_node(node, &mut blanks)?;
        }
        Ok(graph)
    }

    pub fn len(&self) -> usize {
        self.graph.len()
    }

    pub fn is_empty(&self) -> bool {
        self.graph.is_empty()
    }

    pub fn insert(&mut self, triple: TripleRef<'_>) {
        let subject = triple.subject.into_owned();
        if self.seen.insert(subject.clone()) {
            self.subjects.push(subject);
        }
        self.graph.insert(triple);
    }

    pub fn triples(&self) -> impl Iterator<Item = TripleRef<'_>> {
        self.graph.iter()
    }

    /// Every subject, in the order it first appeared.
    pub fn subjects(&self) -> &[NamedOrBlankNode] {
        &self.subjects
    }

    /// Whether `subject` has any triple of its own.
    pub fn describes(&self, subject: NamedOrBlankNodeRef<'_>) -> bool {
        self.graph.triples_for_subject(subject).next().is_some()
    }

    /// The `rdf:type`s of `subject`.
    pub fn types(&self, subject: NamedOrBlankNodeRef<'_>) -> Vec<String> {
        self.graph
            .objects_for_subject_predicate(subject, rdf::TYPE)
            .filter_map(|t| match t {
                TermRef::NamedNode(n) => Some(n.as_str().to_string()),
                _ => None,
            })
            .collect()
    }

    /// The subjects that point at `object` through `predicate`.
    pub fn subjects_linking(
        &self,
        predicate: &str,
        object: NamedOrBlankNodeRef<'_>,
    ) -> Vec<NamedOrBlankNode> {
        let Ok(predicate) = NamedNodeRef::new(predicate) else {
            return vec![];
        };
        let object: TermRef<'_> = match object {
            NamedOrBlankNodeRef::NamedNode(n) => n.into(),
            NamedOrBlankNodeRef::BlankNode(b) => b.into(),
        };
        self.graph
            .subjects_for_predicate_object(predicate, object)
            .map(|s| s.into_owned())
            .collect()
    }

    /// The named subject holding the blank node `label` (`_:b0`), however deep it is nested.
    pub fn named_owner(&self, label: &str) -> Option<String> {
        let mut current = BlankNode::new(label.strip_prefix("_:")?).ok()?;
        for _ in 0..16 {
            let holder = self
                .graph
                .triples_for_object(current.as_ref())
                .next()?
                .subject;
            match holder {
                NamedOrBlankNodeRef::NamedNode(n) => return Some(n.as_str().to_string()),
                NamedOrBlankNodeRef::BlankNode(b) => current = b.into_owned(),
            }
        }
        None
    }

    /// The description of `subject` as an expanded node; blank nodes, and the subjects `inline`
    /// admits, are written inside it, everything else is a reference by `@id`.
    pub fn node(
        &self,
        subject: NamedOrBlankNodeRef<'_>,
        inline: &dyn Fn(NamedOrBlankNodeRef<'_>) -> bool,
    ) -> Map<String, Value> {
        let mut path = HashSet::new();
        self.node_within(subject, inline, &mut path)
    }

    /// The graph in `syntax`; JSON-LD is not written from triples, a profile projects it.
    pub fn write(&self, syntax: RdfSyntax, prefixes: &[(&str, &str)]) -> Outcome<String> {
        match syntax {
            RdfSyntax::Turtle => self.to_turtle(prefixes),
            RdfSyntax::NTriples => Ok(self.to_ntriples()),
            RdfSyntax::RdfXml => self.to_rdf_xml(prefixes),
            RdfSyntax::JsonLd => Err(Errors::crazy(
                "JSON-LD is projected, not written from triples",
                None,
            )),
        }
    }

    /// The graph as RDF/XML, with `prefixes` declared.
    pub fn to_rdf_xml(&self, prefixes: &[(&str, &str)]) -> Outcome<String> {
        let mut serializer = oxrdfxml::RdfXmlSerializer::new();
        for (name, iri) in prefixes {
            serializer = serializer
                .with_prefix(*name, *iri)
                .map_err(|e| Errors::crazy(format!("prefix {name}: {e}"), None))?;
        }
        let mut writer = serializer.for_writer(Vec::new());
        for subject in &self.subjects {
            for triple in self.graph.triples_for_subject(subject.as_ref()) {
                writer
                    .serialize_triple(triple)
                    .map_err(|e| Errors::crazy(format!("writing RDF/XML: {e}"), None))?;
            }
        }
        let bytes = writer
            .finish()
            .map_err(|e| Errors::crazy(format!("writing RDF/XML: {e}"), None))?;
        String::from_utf8(bytes).map_err(|e| Errors::crazy(format!("writing RDF/XML: {e}"), None))
    }

    /// The graph as Turtle, with `prefixes` declared.
    pub fn to_turtle(&self, prefixes: &[(&str, &str)]) -> Outcome<String> {
        let mut serializer = oxttl::TurtleSerializer::new();
        for (name, iri) in prefixes {
            serializer = serializer
                .with_prefix(*name, *iri)
                .map_err(|e| Errors::crazy(format!("prefix {name}: {e}"), None))?;
        }
        let mut writer = serializer.for_writer(Vec::new());
        for subject in &self.subjects {
            for triple in self.graph.triples_for_subject(subject.as_ref()) {
                writer
                    .serialize_triple(triple)
                    .map_err(|e| Errors::crazy(format!("writing Turtle: {e}"), None))?;
            }
        }
        let bytes = writer
            .finish()
            .map_err(|e| Errors::crazy(format!("writing Turtle: {e}"), None))?;
        String::from_utf8(bytes).map_err(|e| Errors::crazy(format!("writing Turtle: {e}"), None))
    }

    /// The graph as N-Triples, one triple per line.
    pub fn to_ntriples(&self) -> String {
        self.subjects
            .iter()
            .flat_map(|s| self.graph.triples_for_subject(s.as_ref()))
            .map(|t| format!("{t} .\n"))
            .collect()
    }

    fn node_within(
        &self,
        subject: NamedOrBlankNodeRef<'_>,
        inline: &dyn Fn(NamedOrBlankNodeRef<'_>) -> bool,
        path: &mut HashSet<NamedOrBlankNode>,
    ) -> Map<String, Value> {
        path.insert(subject.into_owned());
        let mut node = Map::new();
        if let NamedOrBlankNodeRef::NamedNode(n) = subject {
            node.insert("@id".into(), json!(n.as_str()));
        }
        let mut types = Vec::new();
        let mut properties: Vec<(String, Vec<Value>)> = Vec::new();
        for triple in self.graph.triples_for_subject(subject) {
            if triple.predicate == rdf::TYPE {
                if let TermRef::NamedNode(class) = triple.object {
                    types.push(json!(class.as_str()));
                }
                continue;
            }
            let value = self.value_of(triple.object, inline, path);
            let predicate = triple.predicate.as_str();
            match properties.iter_mut().find(|(p, _)| p == predicate) {
                Some((_, values)) => values.push(value),
                None => properties.push((predicate.to_string(), vec![value])),
            }
        }
        if !types.is_empty() {
            node.insert("@type".into(), Value::Array(types));
        }
        for (predicate, values) in properties {
            node.insert(predicate, Value::Array(values));
        }
        path.remove(&subject.into_owned());
        node
    }

    fn value_of(
        &self,
        object: TermRef<'_>,
        inline: &dyn Fn(NamedOrBlankNodeRef<'_>) -> bool,
        path: &mut HashSet<NamedOrBlankNode>,
    ) -> Value {
        let nested: NamedOrBlankNodeRef<'_> = match object {
            TermRef::Literal(literal) => return Self::literal_value(literal),
            TermRef::NamedNode(n) => n.into(),
            TermRef::BlankNode(b) => b.into(),
            #[allow(unreachable_patterns)]
            _ => return Value::Null,
        };
        let embed = match nested {
            NamedOrBlankNodeRef::BlankNode(_) => true,
            named => inline(named) && self.describes(named),
        };
        if embed && !path.contains(&nested.into_owned()) {
            return Value::Object(self.node_within(nested, inline, path));
        }
        match nested {
            NamedOrBlankNodeRef::NamedNode(n) => json!({ "@id": n.as_str() }),
            // A blank node already on the path is a cycle: it is cut, not repeated.
            NamedOrBlankNodeRef::BlankNode(b) => json!({ "@id": format!("_:{}", b.as_str()) }),
        }
    }

    fn literal_value(literal: oxrdf::LiteralRef<'_>) -> Value {
        if let Some(language) = literal.language() {
            return json!({ "@value": literal.value(), "@language": language });
        }
        if literal.datatype() == xsd::STRING {
            return json!({ "@value": literal.value() });
        }
        json!({ "@value": literal.value(), "@type": literal.datatype().as_str() })
    }

    fn add_expanded(&mut self, expanded: &Value) -> Outcome<()> {
        let mut blanks = HashMap::new();
        match expanded {
            Value::Array(nodes) => {
                for node in nodes {
                    if let Value::Object(node) = node {
                        self.add_node(node, &mut blanks)?;
                    }
                }
            }
            Value::Object(node) => {
                self.add_node(node, &mut blanks)?;
            }
            _ => {
                return Err(Self::malformed(
                    "an expanded JSON-LD document is a node or an array of nodes",
                ))
            }
        }
        Ok(())
    }

    /// Writes one expanded node, its nested nodes, named graphs and reverse links.
    fn add_node(
        &mut self,
        node: &Map<String, Value>,
        blanks: &mut HashMap<String, BlankNode>,
    ) -> Outcome<NamedOrBlankNode> {
        let subject = match node.get("@id").and_then(Value::as_str) {
            Some(id) => Self::node_id(id, blanks)?,
            None => BlankNode::default().into(),
        };
        if let Some(Value::Array(graph)) = node.get("@graph") {
            for inner in graph {
                if let Value::Object(inner) = inner {
                    self.add_node(inner, blanks)?;
                }
            }
        }
        let types = match node.get("@type") {
            Some(Value::Array(types)) => types.iter().filter_map(Value::as_str).collect(),
            Some(Value::String(t)) => vec![t.as_str()],
            _ => vec![],
        };
        for class in types {
            if let NamedOrBlankNode::NamedNode(class) = Self::node_id(class, blanks)? {
                self.insert(TripleRef::new(subject.as_ref(), rdf::TYPE, class.as_ref()));
            }
        }
        for (predicate, values) in node.iter().filter(|(k, _)| !k.starts_with('@')) {
            // Blank-node predicates are generalized RDF: there is no triple to write.
            let Ok(predicate) = NamedNode::new(predicate.as_str()) else {
                continue;
            };
            for value in Self::array(values) {
                if let Some(object) = self.object(value, blanks)? {
                    self.insert(TripleRef::new(
                        subject.as_ref(),
                        predicate.as_ref(),
                        object.as_ref(),
                    ));
                }
            }
        }
        if let Some(Value::Object(reverse)) = node.get("@reverse") {
            for (predicate, values) in reverse {
                let Ok(predicate) = NamedNode::new(predicate.as_str()) else {
                    continue;
                };
                for value in Self::array(values) {
                    if let Value::Object(other) = value {
                        let other = self.add_node(other, blanks)?;
                        let object: Term = match &subject {
                            NamedOrBlankNode::NamedNode(n) => n.clone().into(),
                            NamedOrBlankNode::BlankNode(b) => b.clone().into(),
                        };
                        self.insert(TripleRef::new(
                            other.as_ref(),
                            predicate.as_ref(),
                            object.as_ref(),
                        ));
                    }
                }
            }
        }
        Ok(subject)
    }

    /// The RDF term of one expanded value; `None` for what has no RDF form (a null).
    fn object(
        &mut self,
        value: &Value,
        blanks: &mut HashMap<String, BlankNode>,
    ) -> Outcome<Option<Term>> {
        let Value::Object(object) = value else {
            return Ok(None);
        };
        if let Some(literal) = object.get("@value") {
            return Self::literal(literal, object).map(|l| l.map(Term::from));
        }
        if let Some(Value::Array(items)) = object.get("@list") {
            return self.list(items, blanks).map(Some);
        }
        let is_reference = object.keys().all(|k| k == "@id");
        let node = if is_reference {
            match object.get("@id").and_then(Value::as_str) {
                Some(id) => Self::node_id(id, blanks)?,
                None => return Ok(None),
            }
        } else {
            self.add_node(object, blanks)?
        };
        Ok(Some(match node {
            NamedOrBlankNode::NamedNode(n) => n.into(),
            NamedOrBlankNode::BlankNode(b) => b.into(),
        }))
    }

    fn literal(value: &Value, object: &Map<String, Value>) -> Outcome<Option<Literal>> {
        let datatype = object.get("@type").and_then(Value::as_str);
        if datatype == Some("@json") {
            return Ok(Some(Literal::new_typed_literal(
                value.to_string(),
                rdf::JSON,
            )));
        }
        let (lexical, native) = match value {
            Value::String(s) => (s.clone(), None),
            Value::Bool(b) => (b.to_string(), Some(xsd::BOOLEAN)),
            Value::Number(n) if n.is_i64() || n.is_u64() => (n.to_string(), Some(xsd::INTEGER)),
            Value::Number(n) => (
                format!("{:E}", n.as_f64().unwrap_or_default()),
                Some(xsd::DOUBLE),
            ),
            _ => return Ok(None),
        };
        if let Some(language) = object.get("@language").and_then(Value::as_str) {
            return Literal::new_language_tagged_literal(lexical, language)
                .map(Some)
                .map_err(|e| Self::malformed(format!("language tag {language}: {e}")));
        }
        Ok(Some(match (datatype, native) {
            (Some(datatype), _) => Literal::new_typed_literal(
                lexical,
                NamedNode::new(datatype)
                    .map_err(|e| Self::malformed(format!("datatype {datatype}: {e}")))?,
            ),
            (None, Some(native)) => Literal::new_typed_literal(lexical, native),
            (None, None) => Literal::new_simple_literal(lexical),
        }))
    }

    /// An `@list` as an RDF collection.
    fn list(&mut self, items: &[Value], blanks: &mut HashMap<String, BlankNode>) -> Outcome<Term> {
        let mut rest: Term = rdf::NIL.into_owned().into();
        for item in items.iter().rev() {
            let Some(first) = self.object(item, blanks)? else {
                continue;
            };
            let cell = BlankNode::default();
            self.insert(TripleRef::new(cell.as_ref(), rdf::FIRST, first.as_ref()));
            self.insert(TripleRef::new(cell.as_ref(), rdf::REST, rest.as_ref()));
            rest = cell.into();
        }
        Ok(rest)
    }

    /// A node identifier: `_:` labels map to one blank node each, the rest must be IRIs.
    fn node_id(id: &str, blanks: &mut HashMap<String, BlankNode>) -> Outcome<NamedOrBlankNode> {
        if let Some(label) = id.strip_prefix("_:") {
            return Ok(blanks.entry(label.to_string()).or_default().clone().into());
        }
        NamedNode::new(id)
            .map(Into::into)
            .map_err(|e| Self::malformed(format!("{id} is not an absolute IRI: {e}")))
    }

    fn array(values: &Value) -> Vec<&Value> {
        match values {
            Value::Array(values) => values.iter().collect(),
            other => vec![other],
        }
    }

    fn malformed(reason: impl Into<String>) -> Errors {
        Errors::format(BadFormat::Received, reason.into(), None)
    }

    /// Every triple, owned.
    pub fn to_triples(&self) -> Vec<Triple> {
        self.graph.iter().map(TripleRef::into_owned).collect()
    }
}
