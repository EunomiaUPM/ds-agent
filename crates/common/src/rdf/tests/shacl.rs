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

//! `ShapeSet`: compiling groups, choosing them and reading rudof's results back.

use std::borrow::Cow;

use crate::eunomia::EunomiaProfile;
use crate::rdf::shacl::{ShaclSeverity, ShapeSet, ShapeSource};
use crate::rdf::{RdfGraph, RdfSyntax};

const PREFIXES: [(&str, &str); 2] = [
    ("dct", "http://purl.org/dc/terms/"),
    ("dcat", "http://www.w3.org/ns/dcat#"),
];

const REQUIRED: &str = r#"
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix dcat: <http://www.w3.org/ns/dcat#> .
@prefix dct: <http://purl.org/dc/terms/> .
<urn:shape:dataset> a sh:NodeShape ; sh:targetClass dcat:Dataset ;
    sh:property [ sh:path dct:title ; sh:minCount 1 ] ;
    sh:property [ sh:path dct:issued ; sh:maxCount 1 ] .
"#;

const ADVISED: &str = r#"
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix dcat: <http://www.w3.org/ns/dcat#> .
<urn:shape:keywords> a sh:NodeShape ; sh:targetClass dcat:Dataset ;
    sh:property [ sh:path dcat:keyword ; sh:minCount 1 ; sh:severity sh:Warning ] .
"#;

fn shapes() -> ShapeSet {
    ShapeSet::compile(
        vec![
            ShapeSource {
                name: "base",
                description: "required",
                default: true,
                turtle: Cow::Borrowed(REQUIRED),
            },
            ShapeSource {
                name: "advice",
                description: "advised",
                default: false,
                turtle: Cow::Borrowed(ADVISED),
            },
        ],
        &PREFIXES,
    )
    .unwrap()
}

async fn graph(turtle: &str) -> RdfGraph {
    RdfGraph::parse(
        RdfSyntax::Turtle,
        turtle.as_bytes(),
        EunomiaProfile::shared(),
    )
    .await
    .unwrap()
}

/// A missing mandatory value is a violation with a readable, compacted message.
#[tokio::test]
async fn missing_value_is_a_violation() {
    let data = graph("<urn:d:1> a <http://www.w3.org/ns/dcat#Dataset> .").await;
    let report = shapes().validate(&data, &[]).unwrap();

    assert!(!report.conforms);
    assert_eq!(report.violations, 1);
    let result = &report.results[0];
    assert_eq!(result.focus_node, "urn:d:1");
    assert_eq!(result.constraint, "MinCount");
    assert_eq!(
        result.path.as_deref(),
        Some("http://purl.org/dc/terms/title")
    );
    assert_eq!(result.message, "dct:title is mandatory");
    assert_eq!(report.checks, vec!["base"]);
}

/// Too many values break a maximum.
#[tokio::test]
async fn extra_value_breaks_the_maximum() {
    let data = graph(
        r#"<urn:d:1> a <http://www.w3.org/ns/dcat#Dataset> ; <http://purl.org/dc/terms/title> "T" ;
           <http://purl.org/dc/terms/issued> "2026" , "2027" ."#,
    )
    .await;
    let report = shapes().validate(&data, &[]).unwrap();

    assert_eq!(report.results.len(), 1);
    assert_eq!(
        report.results[0].message,
        "dct:issued takes at most 1 value(s)"
    );
}

/// Groups not run by default run when named; their warnings leave the graph conforming.
#[tokio::test]
async fn named_groups_run_and_warnings_conform() {
    let data = graph(r#"<urn:d:1> a <http://www.w3.org/ns/dcat#Dataset> ; <http://purl.org/dc/terms/title> "T" ."#).await;
    let report = shapes().validate(&data, &["advice".to_string()]).unwrap();

    assert!(report.conforms);
    assert_eq!(report.warnings, 1);
    assert_eq!(report.results[0].severity, ShaclSeverity::Warning);
    assert_eq!(report.results[0].message, "dcat:keyword is recommended");
}

/// An unknown group is a format error listing the available ones.
#[tokio::test]
async fn unknown_group_is_refused() {
    let data = graph("<urn:d:1> a <http://www.w3.org/ns/dcat#Dataset> .").await;
    let error = shapes().validate(&data, &["nope".to_string()]).unwrap_err();

    assert!(format!("{error:?}").contains("base, advice"));
}

/// Results can be narrowed to the nodes of interest.
#[tokio::test]
async fn report_narrows_to_nodes() {
    let data = graph(
        "<urn:d:1> a <http://www.w3.org/ns/dcat#Dataset> . <urn:d:2> a <http://www.w3.org/ns/dcat#Dataset> .",
    )
    .await;
    let report = shapes()
        .validate(&data, &[])
        .unwrap()
        .about(&["urn:d:2".to_string()]);

    assert_eq!(report.results.len(), 1);
    assert_eq!(report.results[0].focus_node, "urn:d:2");
}

/// A result on a blank node is attributed to the named subject holding it.
#[tokio::test]
async fn nested_results_name_their_holder() {
    let shapes = ShapeSet::compile(
        vec![ShapeSource {
            name: "base",
            description: "periods",
            default: true,
            turtle: Cow::Borrowed(
                r#"@prefix sh: <http://www.w3.org/ns/shacl#> .
                   <urn:shape:period> a sh:NodeShape ; sh:targetClass <http://purl.org/dc/terms/PeriodOfTime> ;
                       sh:property [ sh:path <http://www.w3.org/ns/dcat#startDate> ; sh:minCount 1 ] ."#,
            ),
        }],
        &PREFIXES,
    )
    .unwrap();
    let data = graph(
        "<urn:d:1> <http://purl.org/dc/terms/temporal> [ a <http://purl.org/dc/terms/PeriodOfTime> ] .",
    )
    .await;
    let report = shapes.validate(&data, &[]).unwrap().attributed(&data);

    assert_eq!(report.results[0].focus_node, "urn:d:1");
    assert_eq!(
        report.results[0].message,
        "in a nested node, dcat:startDate is mandatory"
    );
}
