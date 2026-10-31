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

//! `ImportPlan` over the EDACCIT DCAT-AP fixtures and small documents: kinds, parents, ids and
//! descriptions; no doubles needed.

use std::str::FromStr;

use catalog_agent::entities::findings::Severity;
use catalog_agent::entities::import_plan::{ImportPlan, PlannedResource};
use catalog_agent::entities::resource::CatalogResource;
use catalog_agent::entities::resource_kind::ResourceKind;
use catalog_agent::entities::vocab::Vocab;
use serde_json::json;
use urn::Urn;
use ymir::oauth::Owner;

use crate::support::fixtures::{dcat_ap_graph, dcat_ap_names, graph_of};

fn main() -> Urn {
    Urn::from_str("urn:catalog:main").unwrap()
}

fn find<'a>(plan: &'a ImportPlan, source: &str) -> &'a PlannedResource {
    plan.resources
        .iter()
        .find(|r| r.source == source)
        .unwrap_or_else(|| panic!("{source} not planned"))
}

/// Palma's document is one catalog with its dataset, distribution, record and publisher; the
/// contact is no resource of its own.
#[tokio::test]
async fn palma_plans_its_resources() {
    let plan = ImportPlan::build(
        &dcat_ap_graph("fc-lastcycle-daily-palma.jsonld").await,
        None,
        &main(),
    );
    let kinds: Vec<ResourceKind> = plan.resources.iter().map(|r| r.kind).collect();

    assert!(plan.is_clean(), "{:#?}", plan.issues);
    assert_eq!(
        kinds,
        vec![
            ResourceKind::Catalog,
            ResourceKind::Agent,
            ResourceKind::Dataset,
            ResourceKind::CatalogRecord,
            ResourceKind::Distribution
        ]
    );
    let catalog = "urn:catalog:puertos-es-ocean-forecast";
    assert_eq!(
        find(&plan, "urn:dataset:fc-lastcycle-daily-palma")
            .parent
            .as_ref()
            .unwrap()
            .to_string(),
        catalog
    );
    assert_eq!(
        find(&plan, "urn:record:fc-lastcycle-daily-palma")
            .parent
            .as_ref()
            .unwrap()
            .to_string(),
        catalog
    );
    assert_eq!(
        find(&plan, "urn:distribution:fc-lastcycle-daily-palma:netcdf")
            .parent
            .as_ref()
            .unwrap()
            .to_string(),
        "urn:dataset:fc-lastcycle-daily-palma"
    );
    assert!(find(&plan, catalog).parent.is_none());
}

/// Memberships live in the parents, not in descriptions; a node that is no resource (the
/// vCard contact) is written inside the resource pointing at it.
#[tokio::test]
async fn memberships_leave_and_contacts_inline() {
    let plan = ImportPlan::build(
        &dcat_ap_graph("fc-lastcycle-daily-palma.jsonld").await,
        None,
        &main(),
    );
    let catalog = &find(&plan, "urn:catalog:puertos-es-ocean-forecast").description;
    let dataset = &find(&plan, "urn:dataset:fc-lastcycle-daily-palma").description;

    assert!(!catalog.has(Vocab::DCAT_DATASET_P));
    assert!(!catalog.has(Vocab::DCAT_RECORD));
    assert!(!dataset.has(Vocab::DCAT_DISTRIBUTION_P));
    let contact = &dataset.values(Vocab::DCAT_CONTACT_POINT)[0];
    assert_eq!(contact["@id"], "urn:contact:puertos-del-estado");
    assert!(contact.get("http://www.w3.org/2006/vcard/ns#fn").is_some());
    assert_eq!(
        dataset.first_iri(Vocab::DCT_PUBLISHER),
        Some("urn:org:puertos-del-estado")
    );
}

/// Every EDACCIT document plans without problems, a data service included.
#[tokio::test]
async fn every_fixture_plans_cleanly() {
    for name in dcat_ap_names() {
        let plan = ImportPlan::build(&dcat_ap_graph(&name).await, None, &main());

        assert!(plan.is_clean(), "{name}: {:#?}", plan.issues);
        assert!(
            plan.resources
                .iter()
                .any(|r| r.kind == ResourceKind::Dataset),
            "{name}"
        );
    }
    let adif = ImportPlan::build(
        &dcat_ap_graph("adif-railwaylink-wfs.jsonld").await,
        None,
        &main(),
    );
    assert_eq!(
        find(&adif, "urn:service:adif-tramificacion-comun-wfs")
            .parent
            .as_ref()
            .unwrap()
            .to_string(),
        "urn:catalog:adif-ideadif"
    );
}

fn http_document() -> serde_json::Value {
    json!({
        "@context": {"dcat": "http://www.w3.org/ns/dcat#", "dct": "http://purl.org/dc/terms/"},
        "@graph": [
            {"@id": "https://data.example/catalog", "@type": "dcat:Catalog", "dct:title": "C",
             "dcat:dataset": {"@id": "https://data.example/ds/1"}},
            {"@id": "https://data.example/ds/1", "@type": "dcat:Dataset", "dct:title": "D",
             "dcat:distribution": {"@id": "https://data.example/ds/1/csv"}},
            {"@id": "https://data.example/ds/1/csv", "@type": "dcat:Distribution",
             "dcat:accessURL": {"@id": "https://data.example/files/1.csv"}}
        ]
    })
}

/// IRIs that are not URNs get the same URN on every import, keep the IRI as identifier and are
/// referenced by their new id.
#[tokio::test]
async fn http_iris_get_stable_urns() {
    let graph = graph_of(&http_document()).await;
    let first = ImportPlan::build(&graph, None, &main());
    let second = ImportPlan::build(&graph, None, &main());
    let dataset = find(&first, "https://data.example/ds/1");
    let distribution = find(&first, "https://data.example/ds/1/csv");

    assert_eq!(dataset.id, find(&second, "https://data.example/ds/1").id);
    assert!(dataset.id.to_string().starts_with("urn:dataset:"));
    assert_eq!(distribution.parent.as_ref(), Some(&dataset.id));
    assert_eq!(
        dataset
            .description
            .first_value(Vocab::DCT_IDENTIFIER)
            .as_deref(),
        Some("https://data.example/ds/1")
    );
    assert_eq!(
        dataset.parent.as_ref(),
        Some(&find(&first, "https://data.example/catalog").id)
    );
}

/// With a target the document's catalogs are not stored and what they list goes there.
#[tokio::test]
async fn target_takes_the_members() {
    let target = Urn::from_str("urn:catalog:target").unwrap();
    let plan = ImportPlan::build(&graph_of(&http_document()).await, Some(&target), &main());

    assert!(plan
        .resources
        .iter()
        .all(|r| r.kind != ResourceKind::Catalog));
    assert_eq!(
        find(&plan, "https://data.example/ds/1").parent.as_ref(),
        Some(&target)
    );
}

/// A dataset no catalog lists goes to the main catalog when the document has none.
#[tokio::test]
async fn orphan_dataset_goes_to_main() {
    let document =
        json!({"@id": "urn:dataset:lonely", "@type": "http://www.w3.org/ns/dcat#Dataset"});
    let plan = ImportPlan::build(&graph_of(&document).await, None, &main());

    assert_eq!(
        find(&plan, "urn:dataset:lonely").parent.as_ref(),
        Some(&main())
    );
}

/// A distribution no dataset lists cannot be placed: a violation, and it is left out.
#[tokio::test]
async fn orphan_distribution_is_a_violation() {
    let document = json!({"@id": "urn:distribution:lonely", "@type": "http://www.w3.org/ns/dcat#Distribution"});
    let plan = ImportPlan::build(&graph_of(&document).await, None, &main());

    assert!(!plan.is_clean());
    assert_eq!(plan.issues[0].severity, Severity::Violation);
    assert!(plan.resources.is_empty());
}

/// A data service without endpoint is refused as the management API refuses it.
#[tokio::test]
async fn data_service_needs_its_endpoint() {
    let document =
        json!({"@id": "urn:service:bare", "@type": "http://www.w3.org/ns/dcat#DataService"});
    let plan = ImportPlan::build(&graph_of(&document).await, None, &main());

    assert!(plan
        .issues
        .iter()
        .any(|i| i.property.as_deref() == Some("endpointURL")));
}

/// An imported resource keeps the dates its document gives.
#[tokio::test]
async fn imported_resource_keeps_document_dates() {
    let plan = ImportPlan::build(
        &dcat_ap_graph("fc-lastcycle-daily-palma.jsonld").await,
        None,
        &main(),
    );
    let planned = find(&plan, "urn:dataset:fc-lastcycle-daily-palma");
    let resource = CatalogResource::imported(
        planned.id.clone(),
        planned.kind,
        Owner::connector(),
        planned.parent.clone(),
        None,
        planned.description.clone(),
    );

    assert_eq!(resource.issued().to_rfc3339(), "2026-04-12T00:00:00+00:00");
    assert_eq!(
        resource.description().values(Vocab::DCT_ISSUED)[0],
        json!({"@value": "2026-04-12", "@type": Vocab::XSD_DATE})
    );
    let reimported = resource.reimported(planned.description.clone());
    assert_eq!(reimported.version(), 2);
}
