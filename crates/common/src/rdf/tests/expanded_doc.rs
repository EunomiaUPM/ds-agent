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

//! ExpandedDoc navigation over hand-written expanded JSON-LD, without the engine.

use serde_json::{json, Value};

use crate::rdf::ExpandedDoc;

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

/// Nodes are found by their type.
#[test]
fn finds_the_node_by_type() {
    let g = graph_form();
    let doc = ExpandedDoc::new(&g).unwrap();
    assert_eq!(doc.nodes_of_type(MSG).count(), 1);
    assert_eq!(doc.nodes_of_type("https://example.org/Other").count(), 0);
}

/// A bare `@id` reference resolves to the node that holds the data.
#[test]
fn follows_a_bare_reference_to_the_node_holding_the_data() {
    let g = graph_form();
    let doc = ExpandedDoc::new(&g).unwrap();
    let msg = doc.nodes_of_type(MSG).next().unwrap();
    let addr = msg.object(ADDR).expect("address resolves");
    assert_eq!(addr.iri_or_literal(PORT), Some("8080"));
}

/// A pid reads the same whether it came as an `@id` or as a literal.
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

/// A repeated term is counted, and reading it returns the first value.
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

/// Expanded JSON-LD is always an array; anything else is rejected.
#[test]
fn a_non_array_expansion_is_rejected() {
    assert!(ExpandedDoc::new(&json!({"@type": [MSG]})).is_none());
}
