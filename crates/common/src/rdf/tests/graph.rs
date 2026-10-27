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

//! `RdfGraph`: reading each syntax, expanded nodes both ways and writing Turtle.

use oxrdf::NamedNodeRef;
use serde_json::json;

use crate::eunomia::EunomiaProfile;
use crate::rdf::{RdfGraph, RdfSyntax};

const TURTLE: &str = r#"
@prefix dcat: <http://www.w3.org/ns/dcat#> .
@prefix dct: <http://purl.org/dc/terms/> .
<urn:dataset:1> a dcat:Dataset ;
    dct:title "Readings"@en ;
    dct:temporal [ a dct:PeriodOfTime ; dcat:startDate "2026-01-01" ] .
"#;

/// Turtle is read with its subjects in order and blank nodes kept.
#[tokio::test]
async fn turtle_is_read() {
    let graph = RdfGraph::parse(
        RdfSyntax::Turtle,
        TURTLE.as_bytes(),
        EunomiaProfile::shared(),
    )
    .await
    .unwrap();

    assert_eq!(graph.len(), 5);
    assert_eq!(graph.subjects()[0].to_string(), "<urn:dataset:1>");
    assert_eq!(
        graph.types(NamedNodeRef::new("urn:dataset:1").unwrap().into()),
        vec!["http://www.w3.org/ns/dcat#Dataset"]
    );
}

/// JSON-LD with an inline context gives the same triples as the Turtle it mirrors.
#[tokio::test]
async fn json_ld_matches_turtle() {
    let document = json!({
        "@context": {"dcat": "http://www.w3.org/ns/dcat#", "dct": "http://purl.org/dc/terms/"},
        "@id": "urn:dataset:1",
        "@type": "dcat:Dataset",
        "dct:title": {"@value": "Readings", "@language": "en"},
        "dct:temporal": {"@type": "dct:PeriodOfTime", "dcat:startDate": "2026-01-01"}
    });
    let body = serde_json::to_vec(&document).unwrap();
    let from_json = RdfGraph::parse(RdfSyntax::JsonLd, &body, EunomiaProfile::shared())
        .await
        .unwrap();
    let from_turtle = RdfGraph::parse(
        RdfSyntax::Turtle,
        TURTLE.as_bytes(),
        EunomiaProfile::shared(),
    )
    .await
    .unwrap();

    assert_eq!(from_json.len(), from_turtle.len());
    assert!(from_json
        .to_ntriples()
        .contains("<urn:dataset:1> <http://purl.org/dc/terms/title> \"Readings\"@en"));
}

/// A node read back embeds its blank nodes and references everything else by `@id`.
#[tokio::test]
async fn node_embeds_blank_nodes() {
    let graph = RdfGraph::parse(
        RdfSyntax::Turtle,
        TURTLE.as_bytes(),
        EunomiaProfile::shared(),
    )
    .await
    .unwrap();
    let node = graph.node(NamedNodeRef::new("urn:dataset:1").unwrap().into(), &|_| {
        false
    });

    assert_eq!(node["@id"], "urn:dataset:1");
    assert_eq!(
        node["http://purl.org/dc/terms/title"],
        json!([{"@value": "Readings", "@language": "en"}])
    );
    let period = &node["http://purl.org/dc/terms/temporal"][0];
    assert_eq!(
        period["@type"],
        json!(["http://purl.org/dc/terms/PeriodOfTime"])
    );
    assert!(period.get("@id").is_none());
}

/// Expanded nodes become triples, typed literals and nested nodes included, and Turtle is
/// written with the given prefixes.
#[test]
fn expanded_nodes_round_trip_to_turtle() {
    let node = json!({
        "@id": "urn:dataset:1",
        "@type": ["http://www.w3.org/ns/dcat#Dataset"],
        "http://purl.org/dc/terms/issued": [{"@value": "2026-04-12", "@type": "http://www.w3.org/2001/XMLSchema#date"}],
        "http://www.w3.org/ns/dcat#contactPoint": [{"@id": "urn:contact:1", "http://www.w3.org/2006/vcard/ns#fn": [{"@value": "Desk"}]}]
    });
    let graph = RdfGraph::from_nodes([node.as_object().unwrap()]).unwrap();
    let turtle = graph
        .to_turtle(&[("dcat", "http://www.w3.org/ns/dcat#")])
        .unwrap();

    assert_eq!(graph.len(), 4);
    assert!(turtle.contains("@prefix dcat: <http://www.w3.org/ns/dcat#>"));
    assert!(turtle.contains("dcat:Dataset"));
    assert!(graph
        .to_ntriples()
        .contains("\"2026-04-12\"^^<http://www.w3.org/2001/XMLSchema#date>"));
}

/// A malformed document is a format error naming the syntax.
#[tokio::test]
async fn malformed_turtle_is_refused() {
    let error = RdfGraph::parse(RdfSyntax::Turtle, b"<a> <b> .", EunomiaProfile::shared())
        .await
        .unwrap_err();

    assert!(error.to_string().contains("Turtle") || format!("{error:?}").contains("Turtle"));
}

/// Media types and format names map to syntaxes; plain JSON reads as JSON-LD.
#[test]
fn syntaxes_are_named() {
    assert_eq!(
        RdfSyntax::from_media_type("application/json; charset=utf-8").unwrap(),
        RdfSyntax::JsonLd
    );
    assert_eq!(
        RdfSyntax::from_media_type("text/turtle").unwrap(),
        RdfSyntax::Turtle
    );
    assert!(RdfSyntax::from_media_type("text/csv").is_err());
    assert_eq!(RdfSyntax::from_name("ttl"), Some(RdfSyntax::Turtle));
    assert_eq!(RdfSyntax::Turtle.extension(), "ttl");
}

/// The named holder of a nested blank node is found however deep it sits.
#[tokio::test]
async fn blank_node_owner_is_found() {
    let graph = RdfGraph::parse(
        RdfSyntax::Turtle,
        TURTLE.as_bytes(),
        EunomiaProfile::shared(),
    )
    .await
    .unwrap();
    let blank = graph.subjects()[1].to_string();

    assert_eq!(graph.named_owner(&blank).as_deref(), Some("urn:dataset:1"));
}
