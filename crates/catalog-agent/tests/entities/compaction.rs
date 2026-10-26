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

//! Compactor and TermTable: every profile's compact output expands back to the same triples.

use catalog_agent::entities::admin_document::AdminDocument;
use catalog_agent::entities::jsonld::{Compactor, TermTable};
use catalog_agent::profiles::dcat3::Dcat3;
use catalog_agent::profiles::dcat_ap::DcatAp;
use catalog_agent::profiles::shared::dcat::DcatProfileSpec;
use common::eunomia::{EunomiaProfile, EUNOMIA_CATALOG_CONTEXT_URL};
use serde_json::{json, Value};

/// An expanded dataset touching every coercion: languages, IRIs, typed and plain literals,
/// an inline blank node and a property no table defines.
fn expanded_dataset() -> Value {
    json!({
        "@id": "urn:dataset:1",
        "@type": ["http://www.w3.org/ns/dcat#Dataset"],
        "http://purl.org/dc/terms/title": [
            {"@value": "Readings", "@language": "en"},
            {"@value": "Lecturas", "@language": "es"}
        ],
        "http://purl.org/dc/terms/description": [{"@value": "Untagged text"}],
        "http://www.w3.org/ns/dcat#keyword": [
            {"@value": "energy", "@language": "en"},
            {"@value": "meter", "@language": "en"}
        ],
        "http://www.w3.org/ns/dcat#theme": [
            {"@id": "http://publications.europa.eu/resource/authority/data-theme/ENER"}
        ],
        "http://purl.org/dc/terms/issued": [
            {"@value": "2026-01-01T00:00:00.000Z", "@type": "http://www.w3.org/2001/XMLSchema#dateTime"}
        ],
        "http://purl.org/dc/terms/temporal": [{
            "@type": ["http://purl.org/dc/terms/PeriodOfTime"],
            "http://www.w3.org/ns/dcat#startDate": [
                {"@value": "2019", "@type": "http://www.w3.org/2001/XMLSchema#gYear"}
            ]
        }],
        "http://purl.org/dc/terms/format": [{"@value": "http+pull"}],
        "http://purl.org/dc/terms/publisher": [{"@id": "urn:agent:1"}],
        "https://example.org/ns#extension": [{"@value": 3}]
    })
}

async fn n_quads(document: &Value) -> String {
    EunomiaProfile::shared().expand(document).await.unwrap().canonical_n_quads
}

async fn round_trips(table: &TermTable, context: Value) {
    let node = expanded_dataset();
    let mut compact = Compactor::new(table).node(node.as_object().unwrap());
    compact.insert("@context".into(), context);
    assert_eq!(n_quads(&Value::Object(compact)).await, n_quads(&node).await);
}

/// The catalog context asset and the admin term table agree.
#[tokio::test]
async fn admin_terms_round_trip_through_the_asset_context() {
    round_trips(&AdminDocument::terms(), json!(EUNOMIA_CATALOG_CONTEXT_URL)).await;
}

/// DCAT 3 output with its own context expands to the stored triples.
#[tokio::test]
async fn dcat3_terms_round_trip() {
    let table = Dcat3::terms();
    round_trips(&table, table.context()["@context"].clone()).await;
}

/// DCAT-AP output with its own context expands to the stored triples.
#[tokio::test]
async fn dcat_ap_terms_round_trip() {
    let table = DcatAp::terms();
    round_trips(&table, table.context()["@context"].clone()).await;
}

/// A term the table knows is written by its term, a property it does not by compact IRI.
#[tokio::test]
async fn known_terms_and_compact_iris() {
    let table = AdminDocument::terms();
    let compact = Compactor::new(&table).node(expanded_dataset().as_object().unwrap());

    assert_eq!(compact["@type"], "Dataset");
    assert_eq!(compact["title"], json!({"en": "Readings", "es": "Lecturas"}));
    assert_eq!(compact["description"], "Untagged text");
    assert_eq!(compact["theme"], json!(["http://publications.europa.eu/resource/authority/data-theme/ENER"]));
    assert_eq!(compact["issued"], "2026-01-01T00:00:00.000Z");
    assert_eq!(compact["format"], "http+pull");
    assert_eq!(compact["https://example.org/ns#extension"], json!({"@value": 3}));
}

/// A context protects its terms, so a peer cannot redefine them under our documents.
#[test]
fn generated_contexts_are_protected() {
    let context = Dcat3::terms().context();

    assert_eq!(context["@context"]["@protected"], true);
    assert_eq!(context["@context"]["title"]["@container"], "@language");
}
