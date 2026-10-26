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

//! AdminDocument read through the eunomia engine and written back with its terms; no doubles.

use catalog_agent::entities::admin_document::AdminDocument;
use catalog_agent::entities::graph::Term;
use catalog_agent::entities::resource_kind::ResourceKind;
use catalog_agent::entities::vocab::Vocab;
use common::eunomia::EUNOMIA_CATALOG_CONTEXT_URL;
use serde_json::json;

use crate::support::builders::{distribution, id, resource, titled};

/// A dataset body yields its kind, id, catalog and language-tagged, repeated values.
#[tokio::test]
async fn dataset_body_is_read_with_languages_and_catalog() {
    let read = AdminDocument::read(json!({
        "@context": EUNOMIA_CATALOG_CONTEXT_URL,
        "@id": "urn:dataset:1",
        "@type": "Dataset",
        "title": {"es": "Lecturas", "en": "Readings"},
        "keyword": {"en": ["energy", "meter"]},
        "inCatalog": "urn:catalog:1"
    }))
    .await
    .unwrap();

    assert_eq!(read.kind, ResourceKind::Dataset);
    assert_eq!(read.id.unwrap().to_string(), "urn:dataset:1");
    assert_eq!(read.parent.unwrap().to_string(), "urn:catalog:1");
    assert_eq!(read.description.text(Vocab::DCT_TITLE, &["es"]).as_deref(), Some("Lecturas"));
    assert_eq!(read.description.count(Vocab::DCAT_KEYWORD), 2);
}

/// A body without `@context` is read in the catalog context.
#[tokio::test]
async fn body_without_context_assumes_the_catalog_context() {
    let read = AdminDocument::read(json!({"@type": "Agent", "name": "ACME"})).await.unwrap();

    assert_eq!(read.kind, ResourceKind::Agent);
    assert!(read.description.has(Vocab::FOAF_NAME));
}

/// The managed dates a caller sends are dropped; the agent stamps its own.
#[tokio::test]
async fn managed_dates_are_dropped() {
    let read = AdminDocument::read(json!({
        "@context": EUNOMIA_CATALOG_CONTEXT_URL,
        "@type": "Catalog",
        "issued": "1999-01-01T00:00:00Z",
        "modified": "1999-01-01T00:00:00Z"
    }))
    .await
    .unwrap();

    assert!(!read.description.has(Vocab::DCT_ISSUED));
    assert!(!read.description.has(Vocab::DCT_MODIFIED));
}

/// A language code is a literal for `language`; elsewhere a relative IRI is refused.
#[tokio::test]
async fn bare_language_codes_are_kept_and_relative_iris_refused() {
    let read = AdminDocument::read(json!({"@context": EUNOMIA_CATALOG_CONTEXT_URL, "@type": "Dataset", "language": ["en"]})).await.unwrap();
    assert!(matches!(
        read.description.terms(Vocab::DCT_LANGUAGE).first(),
        Some(Term::Literal(l)) if l.lexical() == "en"
    ));

    let refused = AdminDocument::read(json!({"@context": EUNOMIA_CATALOG_CONTEXT_URL, "@type": "Dataset", "theme": ["ENER"]})).await;
    assert!(refused.is_err());
}

/// A body with no known class, two resources or a non-URN id is malformed.
#[tokio::test]
async fn bodies_that_name_no_single_resource_are_malformed() {
    assert!(AdminDocument::read(json!({"@context": EUNOMIA_CATALOG_CONTEXT_URL, "title": "x"})).await.is_err());
    assert!(AdminDocument::read(json!({"@context": EUNOMIA_CATALOG_CONTEXT_URL, "@type": "Dataset", "@id": "https://example.org/d"}))
        .await
        .is_err());
    assert!(AdminDocument::read(json!({"@context": EUNOMIA_CATALOG_CONTEXT_URL, "@graph": [{"@type": "Dataset"}, {"@type": "Catalog"}]}))
        .await
        .is_err());
}

/// An embedded description of another resource is kept as a plain reference.
#[tokio::test]
async fn embedded_resources_become_references() {
    let read = AdminDocument::read(json!({
        "@context": EUNOMIA_CATALOG_CONTEXT_URL,
        "@type": "Distribution",
        "accessService": [{"@id": "urn:data-service:1", "endpointURL": "https://x.example"}]
    }))
    .await
    .unwrap();

    let values = read.description.values(Vocab::DCAT_ACCESS_SERVICE);
    assert_eq!(values, &[json!({"@id": "urn:data-service:1"})]);
}

/// What `render` writes reads back as the same resource and description.
#[tokio::test]
async fn rendered_resource_reads_back_the_same() {
    let mut description = titled("Readings");
    description.push(Vocab::DCT_TITLE, json!({"@value": "Lecturas", "@language": "es"}));
    description.put(Vocab::DCAT_THEME, vec![json!({"@id": "http://publications.europa.eu/resource/authority/data-theme/ENER"})]);
    description.put("https://example.org/ns#extension", vec![json!({"@value": "kept"})]);
    let stored = resource(ResourceKind::Dataset, 7, Some(1), None, description);

    let rendered = AdminDocument::render(&stored);
    assert_eq!(rendered["inCatalog"], id(ResourceKind::Catalog, 1));
    let read = AdminDocument::read(rendered).await.unwrap();

    assert_eq!(read.id.as_ref(), Some(stored.id()));
    assert_eq!(read.parent.unwrap().to_string(), id(ResourceKind::Catalog, 1));
    let mut expected = stored.description().clone();
    for managed in Vocab::MANAGED {
        expected.remove(managed);
    }
    expected.remove("@id");
    let mut got = read.description.clone();
    got.remove("@id");
    assert_eq!(got.as_map().get(Vocab::DCT_TITLE), expected.as_map().get(Vocab::DCT_TITLE));
    assert_eq!(got.as_map().get(Vocab::DCAT_THEME), expected.as_map().get(Vocab::DCAT_THEME));
    assert_eq!(
        got.as_map().get("https://example.org/ns#extension"),
        expected.as_map().get("https://example.org/ns#extension")
    );
}

/// A distribution's `dct:format` stays the literal DSP uses.
#[tokio::test]
async fn distribution_format_renders_as_written() {
    let rendered = AdminDocument::render(&distribution(4, 3, 2, "http+pull"));

    assert_eq!(rendered["format"], "http+pull");
    assert_eq!(rendered["ofDataset"], id(ResourceKind::Dataset, 3));
    assert_eq!(rendered["accessService"], json!([id(ResourceKind::DataService, 2)]));
}
