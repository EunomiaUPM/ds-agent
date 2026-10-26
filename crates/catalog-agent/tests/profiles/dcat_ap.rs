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

//! The DCAT-AP adapter over built resource sets: its rewrites and its requirements.

use catalog_agent::entities::graph::ResourceGraph;
use catalog_agent::entities::resource_kind::ResourceKind;
use catalog_agent::entities::vocab::Vocab;
use catalog_agent::profiles::dcat_ap::DcatAp;
use catalog_agent::profiles::shared::dcat::DcatAdapter;
use catalog_agent::profiles::{ProfileAdapterTrait, Severity};
use common::eunomia::EunomiaProfile;
use serde_json::json;

use crate::support::builders::{agent, distribution, id, provider_set, resource};

fn adapter() -> DcatAdapter<DcatAp> {
    DcatAdapter::default()
}

/// A distribution reached through a service gets the endpoint as access URL; a DSP transfer
/// format is no file type, so it stays out of DCAT-AP.
#[test]
fn distribution_is_rewritten_into_the_profile() {
    let (_, set) = provider_set();
    let distribution = set.get(&id(ResourceKind::Distribution, 4)).unwrap();
    let graph = adapter().graph(distribution, &set);

    assert_eq!(graph.first_iri(Vocab::DCAT_ACCESS_URL), Some("https://provider.example/dsp"));
    assert!(!graph.has(Vocab::DCT_FORMAT));
}

/// Known format and language literals become the EU vocabulary concepts.
#[test]
fn literals_become_nal_concepts() {
    let (_, mut set) = provider_set();
    let mut csv = distribution(7, 3, 2, "text/csv");
    let mut description = csv.description().clone();
    description.put(Vocab::DCT_LANGUAGE, vec![ResourceGraph::literal("en", None)]);
    csv = csv.revise(description);
    set.insert(csv.clone());
    let graph = adapter().graph(&csv, &set);

    assert_eq!(graph.first_iri(Vocab::DCT_FORMAT), Some("http://publications.europa.eu/resource/authority/file-type/CSV"));
    assert_eq!(graph.first_iri(Vocab::DCT_LANGUAGE), Some("http://publications.europa.eu/resource/authority/language/ENG"));
}

/// A catalog without description or publisher breaks DCAT-AP.
#[test]
fn catalog_needs_description_and_publisher() {
    let (main, set) = provider_set();
    let diagnosis = adapter().diagnose(&set, &main, false).unwrap();
    let missing: Vec<_> = diagnosis
        .findings
        .iter()
        .filter(|f| f.severity == Severity::Violation)
        .filter_map(|f| f.property.as_deref())
        .collect();

    assert!(!diagnosis.compliant);
    assert!(missing.contains(&Vocab::DCT_DESCRIPTION));
    assert!(missing.contains(&Vocab::DCT_PUBLISHER));
}

/// A described dataset conforms; an untagged title and a theme from elsewhere are warnings.
#[test]
fn described_dataset_conforms_with_warnings() {
    let (_, mut set) = provider_set();
    let mut description = ResourceGraph::default();
    description.put(Vocab::DCT_TITLE, vec![ResourceGraph::literal("Readings", None)]);
    description.put(Vocab::DCT_DESCRIPTION, vec![ResourceGraph::literal("Hourly readings", Some("en"))]);
    description.put(Vocab::DCAT_THEME, vec![ResourceGraph::iri("https://example.org/themes/energy")]);
    let dataset = resource(ResourceKind::Dataset, 8, Some(1), None, description);
    set.insert(dataset.clone());
    let diagnosis = adapter().diagnose(&set, &dataset, false).unwrap();

    assert!(diagnosis.compliant, "{:#?}", diagnosis.findings);
    let warned: Vec<_> = diagnosis
        .findings
        .iter()
        .filter(|f| f.severity == Severity::Warning)
        .filter_map(|f| f.property.as_deref())
        .collect();
    assert!(warned.contains(&Vocab::DCT_TITLE));
    assert!(warned.contains(&Vocab::DCAT_THEME));
}

/// A publisher agent and a full catalog description make the catalog itself conform.
#[test]
fn catalog_with_publisher_conforms() {
    let (main, mut set) = provider_set();
    let publisher = agent(9, "ACME");
    let mut description = main.description().clone();
    description.put(Vocab::DCT_DESCRIPTION, vec![ResourceGraph::literal("Provider catalog", Some("en"))]);
    description.put(Vocab::DCT_PUBLISHER, vec![ResourceGraph::iri(publisher.id().as_ref())]);
    let main = main.revise(description);
    set.insert(publisher);
    let diagnosis = adapter().diagnose(&set, &main, false).unwrap();

    assert!(diagnosis.compliant, "{:#?}", diagnosis.findings);
}

/// The projection is one `@graph` with the profile's context and expands to DCAT triples.
#[tokio::test]
async fn projection_is_a_graph_that_expands() {
    let (main, set) = provider_set();
    let document = adapter().project(&set, &main).unwrap();

    assert_eq!(document["@graph"][0]["@id"], id(ResourceKind::Catalog, 1));
    assert_eq!(document["@graph"].as_array().unwrap().len(), set.len());
    let quads = EunomiaProfile::shared().expand(&document).await.unwrap().canonical_n_quads;
    assert!(quads.contains(&format!("<{}> <{}> <{}>", id(ResourceKind::Catalog, 1), Vocab::DCAT_DATASET_P, id(ResourceKind::Dataset, 3))));
    assert!(quads.contains(&format!("<{}> <{}> <https://provider.example/dsp>", id(ResourceKind::Distribution, 4), Vocab::DCAT_ACCESS_URL)));
}

/// The served context is the one the projection uses.
#[test]
fn context_is_the_projection_context() {
    let (main, set) = provider_set();
    let adapter = adapter();

    assert_eq!(adapter.context()["@context"], adapter.project(&set, &main).unwrap()["@context"]);
    assert_eq!(adapter.context()["@context"]["title"], json!({"@id": Vocab::DCT_TITLE, "@container": "@language"}));
    assert_eq!(adapter.key(), "dcat-ap");
}
