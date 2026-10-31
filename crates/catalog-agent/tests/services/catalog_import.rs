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

//! CatalogImportService over a mocked resource repository and the registered profiles, with the
//! EDACCIT Palma document; a repository write that is not expected fails the test.

use std::str::FromStr;

use catalog_agent::data::repo::resource::MockResourceRepoTrait;
use catalog_agent::entities::commands::{ImportCatalogCommand, ImportMode};
use catalog_agent::entities::graph::ResourceGraph;
use catalog_agent::entities::resource::CatalogResource;
use catalog_agent::entities::resource_kind::ResourceKind;
use catalog_agent::services::catalog_import::views::ImportAction;
use common::rdf::RdfSyntax;
use common::test_utils::scopes::TestUsers;
use serde_json::json;
use urn::Urn;
use ymir::oauth::Owner;

use crate::support::builders::main_catalog;
use crate::support::context::TestContext;
use crate::support::fixtures::{dcat_ap_bytes, dcat_ap_json};

const PALMA: &str = "fc-lastcycle-daily-palma.jsonld";
const PALMA_CATALOG: &str = "urn:catalog:puertos-es-ocean-forecast";
const PALMA_DATASET: &str = "urn:dataset:fc-lastcycle-daily-palma";

fn command(dry_run: bool) -> ImportCatalogCommand {
    ImportCatalogCommand {
        profile: "dcat-ap".into(),
        mode: ImportMode::Upsert,
        dry_run,
        target: None,
        checks: vec![],
        visibility: None,
    }
}

/// A repository with the main catalog and `stored` as all it holds.
fn repo(stored: Vec<CatalogResource>) -> MockResourceRepoTrait {
    let mut repo = MockResourceRepoTrait::new();
    repo.expect_get_main()
        .returning(|_| Ok(Some(main_catalog(1))));
    repo.expect_get_batch().returning(move |_, ids| {
        Ok(stored
            .iter()
            .filter(|r| ids.contains(r.id()))
            .cloned()
            .collect())
    });
    repo
}

/// The Palma dataset as an earlier import stored it.
fn stored_dataset() -> CatalogResource {
    CatalogResource::imported(
        Urn::from_str(PALMA_DATASET).unwrap(),
        ResourceKind::Dataset,
        Owner::of(&TestUsers::root(), None),
        Some(Urn::from_str(PALMA_CATALOG).unwrap()),
        None,
        ResourceGraph::default(),
    )
}

/// A dry run validates and plans the whole document and writes nothing.
#[tokio::test]
async fn dry_run_writes_nothing() {
    let ctx = TestContext::new().resource_repo(repo(vec![])).build();

    let report = ctx
        .imports()
        .import(
            &TestUsers::root(),
            RdfSyntax::JsonLd,
            &dcat_ap_bytes(PALMA),
            &command(true),
        )
        .await
        .unwrap();

    assert!(
        report.conforms,
        "{:#?} {:#?}",
        report.validation.results, report.issues
    );
    assert!(!report.imported);
    assert_eq!((report.created, report.updated), (5, 0));
    assert!(report.validation.conforms);
}

/// A conforming document is written in one call, parents first, and the cache is invalidated.
#[tokio::test]
async fn import_writes_everything_at_once() {
    let mut repo = repo(vec![]);
    repo.expect_import().times(1).returning(|_, writes| {
        assert_eq!(writes.len(), 5);
        assert_eq!(writes[0].resource.kind(), ResourceKind::Catalog);
        assert!(writes.iter().all(|w| w.expected.is_none()));
        Ok(())
    });
    let ctx = TestContext::new().resource_repo(repo).build();

    let report = ctx
        .imports()
        .import(
            &TestUsers::root(),
            RdfSyntax::JsonLd,
            &dcat_ap_bytes(PALMA),
            &command(false),
        )
        .await
        .unwrap();

    assert!(report.imported);
}

/// Importing again replaces what is stored, over the version read.
#[tokio::test]
async fn reimport_updates_stored_resources() {
    let mut repo = repo(vec![stored_dataset()]);
    repo.expect_import().times(1).returning(|_, writes| {
        let dataset = writes
            .iter()
            .find(|w| w.resource.id().to_string() == PALMA_DATASET)
            .unwrap();
        assert_eq!(dataset.expected, Some(1));
        assert_eq!(dataset.resource.version(), 2);
        Ok(())
    });
    let ctx = TestContext::new().resource_repo(repo).build();

    let report = ctx
        .imports()
        .import(
            &TestUsers::root(),
            RdfSyntax::JsonLd,
            &dcat_ap_bytes(PALMA),
            &command(false),
        )
        .await
        .unwrap();

    assert_eq!((report.created, report.updated), (4, 1));
    let dataset = report
        .resources
        .iter()
        .find(|r| r.id == PALMA_DATASET)
        .unwrap();
    assert_eq!(dataset.action, ImportAction::Update);
}

/// In create mode a stored resource keeps the document out.
#[tokio::test]
async fn create_mode_refuses_stored_resources() {
    let ctx = TestContext::new()
        .resource_repo(repo(vec![stored_dataset()]))
        .build();
    let mut create = command(false);
    create.mode = ImportMode::Create;

    let report = ctx
        .imports()
        .import(
            &TestUsers::root(),
            RdfSyntax::JsonLd,
            &dcat_ap_bytes(PALMA),
            &create,
        )
        .await
        .unwrap();

    assert!(!report.conforms);
    assert!(!report.imported);
    assert!(report.issues.iter().any(|i| i.resource == PALMA_DATASET));
}

/// A document breaking SEMIC's shapes is reported and not written.
#[tokio::test]
async fn invalid_document_is_not_written() {
    let mut document = dcat_ap_json(PALMA);
    for node in document["@graph"].as_array_mut().unwrap() {
        if node["@type"] == "dcat:Dataset" {
            node.as_object_mut().unwrap().remove("dct:title");
        }
    }
    let ctx = TestContext::new().resource_repo(repo(vec![])).build();

    let report = ctx
        .imports()
        .import(
            &TestUsers::root(),
            RdfSyntax::JsonLd,
            &serde_json::to_vec(&document).unwrap(),
            &command(false),
        )
        .await
        .unwrap();

    assert!(!report.conforms);
    assert_eq!(report.validation.violations, 1);
    assert_eq!(report.validation.results[0].focus_node, PALMA_DATASET);
}

/// A link to a resource neither in the document nor in the catalog keeps it out.
#[tokio::test]
async fn dangling_link_is_an_issue() {
    let document = json!({
        "@context": {"dcat": "http://www.w3.org/ns/dcat#", "dct": "http://purl.org/dc/terms/"},
        "@id": "urn:dataset:linked", "@type": "dcat:Dataset",
        "dct:title": "T", "dct:description": "D",
        "dct:publisher": {"@id": "urn:agent:nobody"}
    });
    let ctx = TestContext::new().resource_repo(repo(vec![])).build();

    let report = ctx
        .imports()
        .import(
            &TestUsers::root(),
            RdfSyntax::JsonLd,
            &serde_json::to_vec(&document).unwrap(),
            &command(true),
        )
        .await
        .unwrap();

    assert!(!report.conforms);
    assert!(report.issues[0].message.contains("urn:agent:nobody"));
}

/// Turtle is imported like JSON-LD.
#[tokio::test]
async fn turtle_is_accepted() {
    let turtle = r#"
        @prefix dcat: <http://www.w3.org/ns/dcat#> . @prefix dct: <http://purl.org/dc/terms/> .
        <urn:dataset:t> a dcat:Dataset ; dct:title "T"@en ; dct:description "D"@en .
    "#;
    let ctx = TestContext::new().resource_repo(repo(vec![])).build();

    let report = ctx
        .imports()
        .import(
            &TestUsers::root(),
            RdfSyntax::Turtle,
            turtle.as_bytes(),
            &command(true),
        )
        .await
        .unwrap();

    assert!(report.conforms, "{:#?}", report.issues);
    assert_eq!(
        report.resources[0].parent.as_deref(),
        Some(main_catalog(1).id().to_string().as_str())
    );
}

/// Only profiles with shapes import; DSP is not one.
#[tokio::test]
async fn dsp_is_no_import_profile() {
    let ctx = TestContext::new().build();
    let mut dsp = command(true);
    dsp.profile = "dsp".into();

    let refused = ctx
        .imports()
        .import(
            &TestUsers::root(),
            RdfSyntax::JsonLd,
            &dcat_ap_bytes(PALMA),
            &dsp,
        )
        .await
        .unwrap_err();

    assert_eq!(refused.info().status_code.as_u16(), 400);
    assert!(refused.reason().contains("dcat-ap"));
}

/// Without the main catalog there is nothing to import into.
#[tokio::test]
async fn main_catalog_comes_first() {
    let mut repo = MockResourceRepoTrait::new();
    repo.expect_get_main().returning(|_| Ok(None));
    let ctx = TestContext::new().resource_repo(repo).build();

    let refused = ctx
        .imports()
        .import(
            &TestUsers::root(),
            RdfSyntax::JsonLd,
            &dcat_ap_bytes(PALMA),
            &command(true),
        )
        .await
        .unwrap_err();

    assert_eq!(refused.info().status_code.as_u16(), 409);
}
