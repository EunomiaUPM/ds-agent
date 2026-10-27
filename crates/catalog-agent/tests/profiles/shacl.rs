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

//! Import validation with each profile's shapes, over the EDACCIT DCAT-AP fixtures: they pass
//! SEMIC's shapes as sent, broken copies do not, and what the agent projects back passes too.

use std::str::FromStr;

use catalog_agent::entities::import_plan::ImportPlan;
use catalog_agent::entities::resource::CatalogResource;
use catalog_agent::entities::resource_kind::{Parent, ResourceKind};
use catalog_agent::entities::resource_set::ResourceSet;
use catalog_agent::profiles::dcat3::Dcat3;
use catalog_agent::profiles::dcat_ap::DcatAp;
use catalog_agent::profiles::shared::dcat::DcatAdapter;
use catalog_agent::profiles::ProfileAdapterTrait;
use common::oauth::Owner;
use common::rdf::RdfGraph;
use serde_json::Value;
use urn::Urn;

use crate::support::fixtures::{dcat_ap_graph, dcat_ap_json, dcat_ap_names, graph_of};

fn dcat_ap() -> DcatAdapter<DcatAp> {
    DcatAdapter::default()
}

/// Every EDACCIT document conforms to SEMIC's default shape groups.
#[tokio::test]
async fn fixtures_conform_to_dcat_ap() {
    let adapter = dcat_ap();
    for name in dcat_ap_names() {
        let report = adapter
            .shapes()
            .unwrap()
            .validate(&dcat_ap_graph(&name).await, &[])
            .unwrap();

        assert!(report.conforms, "{name}: {:#?}", report.results);
        assert_eq!(report.checks, vec!["base", "recommended", "deprecated"]);
    }
}

/// A copy without the dataset's description or the catalog's title breaks SEMIC's Base shapes.
#[tokio::test]
async fn broken_fixture_is_refused() {
    let mut document = dcat_ap_json("fc-lastcycle-daily-palma.jsonld");
    for node in document["@graph"].as_array_mut().unwrap() {
        match node["@type"].as_str() {
            Some("dcat:Dataset") => {
                node.as_object_mut().unwrap().remove("dct:description");
            }
            Some("dcat:Catalog") => {
                node.as_object_mut().unwrap().remove("dct:title");
            }
            _ => {}
        }
    }
    let report = dcat_ap()
        .shapes()
        .unwrap()
        .validate(&graph_of(&document).await, &[])
        .unwrap();
    let missing: Vec<(&str, Option<&str>)> = report
        .results
        .iter()
        .filter(|r| r.check == "base")
        .map(|r| (r.focus_node.as_str(), r.path.as_deref()))
        .collect();

    assert!(!report.conforms);
    assert!(missing.contains(&(
        "urn:dataset:fc-lastcycle-daily-palma",
        Some("http://purl.org/dc/terms/description")
    )));
    assert!(missing.contains(&(
        "urn:catalog:puertos-es-ocean-forecast",
        Some("http://purl.org/dc/terms/title")
    )));
}

/// The ranges group needs every object typed, so it fails documents that rely on the EU
/// vocabularies; it only runs when asked for.
#[tokio::test]
async fn ranges_need_background_knowledge() {
    let report = dcat_ap()
        .shapes()
        .unwrap()
        .validate(
            &dcat_ap_graph("fc-lastcycle-daily-palma.jsonld").await,
            &["range".to_string()],
        )
        .unwrap();

    assert!(!report.conforms);
    assert!(report.results.iter().all(|r| r.constraint == "Class"));
}

/// DCAT 3's shapes are its doctor rules: the fixtures conform, a literal media type does not.
#[tokio::test]
async fn dcat3_shapes_follow_its_rules() {
    let adapter: DcatAdapter<Dcat3> = DcatAdapter::default();
    let shapes = adapter.shapes().unwrap();
    for name in dcat_ap_names() {
        let report = shapes.validate(&dcat_ap_graph(&name).await, &[]).unwrap();
        assert!(report.conforms, "{name}: {:#?}", report.results);
    }
    let mut document = dcat_ap_json("fc-lastcycle-daily-palma.jsonld");
    for node in document["@graph"].as_array_mut().unwrap() {
        if node["@type"] == "dcat:Distribution" {
            node["dcat:mediaType"] = Value::String("application/x-netcdf".into());
        }
    }
    let report = shapes.validate(&graph_of(&document).await, &[]).unwrap();
    assert!(!report.conforms);
    assert!(report
        .results
        .iter()
        .any(|r| r.path.as_deref() == Some("http://www.w3.org/ns/dcat#mediaType")));
}

/// The stored resources of an imported document, as the import service would write them.
fn stored(plan: &ImportPlan) -> ResourceSet {
    let mut set = ResourceSet::new();
    for planned in &plan.resources {
        let (catalog, dataset) = match planned.kind.parent() {
            Parent::Catalog => (planned.parent.clone(), None),
            Parent::Dataset => (None, planned.parent.clone()),
            Parent::None => (None, None),
        };
        set.insert(CatalogResource::imported(
            planned.id.clone(),
            planned.kind,
            Owner::connector(),
            catalog,
            dataset,
            planned.description.clone(),
        ));
    }
    set
}

/// What an import stores, projected back in DCAT-AP, still conforms to SEMIC's shapes.
#[tokio::test]
async fn imported_catalogs_project_back_conforming() {
    let adapter = dcat_ap();
    let main = Urn::from_str("urn:catalog:main").unwrap();
    for name in dcat_ap_names() {
        let plan = ImportPlan::build(&dcat_ap_graph(&name).await, None, &main);
        let set = stored(&plan);
        let root = set
            .iter()
            .find(|r| r.kind() == ResourceKind::Catalog)
            .unwrap()
            .clone();
        let projected = adapter.project(&set, &root).unwrap();
        let expanded = adapter.engine().expand(&projected).await.unwrap().expanded;
        let graph = RdfGraph::from_expanded(&expanded).unwrap();
        let report = adapter
            .shapes()
            .unwrap()
            .validate(&graph, &["base".to_string()])
            .unwrap();

        assert!(
            graph.len() > 30,
            "{name}: only {} triples projected",
            graph.len()
        );

        assert!(report.conforms, "{name}: {:#?}", report.results);
        let diagnosis = adapter.diagnose(&set, &root, true).unwrap();
        assert!(diagnosis.compliant, "{name}: {:#?}", diagnosis.findings);
    }
}
