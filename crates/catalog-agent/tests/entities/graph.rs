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

//! ResourceGraph accessors over expanded nodes; no doubles.

use catalog_agent::entities::graph::{ResourceGraph, Term};
use catalog_agent::entities::vocab::Vocab;
use serde_json::json;

fn graph() -> ResourceGraph {
    ResourceGraph::from_node(json!({
        "@id": "urn:dataset:1",
        "@type": ["http://www.w3.org/ns/dcat#Dataset"],
        "http://purl.org/dc/terms/title": [
            {"@value": "Lecturas", "@language": "es"},
            {"@value": "Readings", "@language": "en"}
        ],
        "http://purl.org/dc/terms/publisher": [{"@id": "urn:agent:1"}],
        "http://purl.org/dc/terms/creator": [{"@id": "https://example.org/people/ana"}],
        "http://www.w3.org/ns/dcat#contactPoint": [{"http://www.w3.org/2006/vcard/ns#fn": [{"@value": "Ana"}]}]
    }))
    .unwrap()
}

/// A text comes in the preferred language, else untagged, else any.
#[test]
fn text_prefers_the_asked_language() {
    let graph = graph();

    assert_eq!(graph.text(Vocab::DCT_TITLE, &["en"]).as_deref(), Some("Readings"));
    assert_eq!(graph.text(Vocab::DCT_TITLE, &["fr"]).as_deref(), Some("Lecturas"));
}

/// IRIs, literals and inline nodes are told apart.
#[test]
fn terms_are_told_apart() {
    let graph = graph();

    assert_eq!(graph.first_iri(Vocab::DCT_PUBLISHER), Some("urn:agent:1"));
    assert_eq!(graph.literals(Vocab::DCT_TITLE).len(), 2);
    assert!(matches!(graph.terms(Vocab::DCAT_CONTACT_POINT).first(), Some(Term::Node(_))));
}

/// Only `urn:` objects of link predicates are local links; outside IRIs are not.
#[test]
fn local_links_are_urn_objects_of_link_predicates() {
    assert_eq!(graph().local_links(), vec![(Vocab::DCT_PUBLISHER, "urn:agent:1".to_string())]);
}

/// Pushing a value already there keeps one; putting none removes the predicate.
#[test]
fn push_deduplicates_and_empty_put_removes() {
    let mut graph = graph();
    graph.push(Vocab::DCT_PUBLISHER, ResourceGraph::iri("urn:agent:1"));
    assert_eq!(graph.count(Vocab::DCT_PUBLISHER), 1);

    graph.put(Vocab::DCT_PUBLISHER, vec![]);
    assert!(!graph.has(Vocab::DCT_PUBLISHER));
}

/// A class is made the first type, other types are kept.
#[test]
fn ensure_type_keeps_other_types() {
    let mut graph = ResourceGraph::from_node(json!({"@type": ["http://xmlns.com/foaf/0.1/Organization"]})).unwrap();
    graph.ensure_type(Vocab::FOAF_AGENT);

    assert_eq!(graph.types(), vec![Vocab::FOAF_AGENT, Vocab::FOAF_ORGANIZATION]);
}

/// A description must be a node object.
#[test]
fn a_description_is_a_node_object() {
    assert!(ResourceGraph::from_node(json!(["x"])).is_err());
}
