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

//! ResourceService over a mocked resource repository: placement, links, versions, ownership.

use catalog_agent::data::repo::resource::MockResourceRepoTrait;
use catalog_agent::entities::commands::{NewResourceCommand, ReplaceResourceCommand};
use catalog_agent::entities::graph::ResourceGraph;
use catalog_agent::entities::resource_kind::ResourceKind;
use catalog_agent::entities::vocab::Vocab;
use common::eunomia::EUNOMIA_CATALOG_CONTEXT_URL;
use common::test_utils::scopes::TestUsers;
use serde_json::json;
use ymir::errors::Errors;
use ymir::types::participants::Visibility;

use crate::support::builders::{catalog, data_service, dataset, main_catalog, titled, urn};
use crate::support::context::TestContext;

fn command(kind: ResourceKind, parent: Option<urn::Urn>, description: ResourceGraph) -> NewResourceCommand {
    NewResourceCommand {
        kind,
        id: None,
        owner: None,
        visibility: None,
        parent,
        description,
    }
}

fn status(error: &Errors) -> u16 {
    error.info().status_code.as_u16()
}

/// A dataset goes into a catalog the user sees, owned by the user, private by default.
#[tokio::test]
async fn dataset_is_created_in_a_seen_catalog() {
    let mut repo = MockResourceRepoTrait::new();
    repo.expect_get_by_id().returning(|_, _| Ok(Some(catalog(1))));
    repo.expect_insert()
        .withf(|r| r.catalog_id() == Some(&urn(ResourceKind::Catalog, 1)) && r.owner().user_id == "ana")
        .returning(|r| Ok(r.clone()));
    let ctx = TestContext::new().resource_repo(repo).build();

    let created = ctx
        .resources()
        .create(&TestUsers::alone("ana"), &command(ResourceKind::Dataset, Some(urn(ResourceKind::Catalog, 1)), titled("d")))
        .await
        .unwrap();

    assert_eq!(created.owner().visibility, Visibility::Private);
    assert!(created.description().has(Vocab::DCT_ISSUED));
    assert!(created.id().to_string().starts_with("urn:dataset:"));
}

/// A dataset with no catalog, or under something that is not one, is refused before storing.
#[tokio::test]
async fn dataset_needs_a_catalog() {
    let mut repo = MockResourceRepoTrait::new();
    repo.expect_get_by_id().returning(|_, _| Ok(Some(dataset(2, 1))));
    let ctx = TestContext::new().resource_repo(repo).build();
    let user = TestUsers::alone("ana");

    let orphan = ctx.resources().create(&user, &command(ResourceKind::Dataset, None, titled("d"))).await;
    let misplaced = ctx
        .resources()
        .create(&user, &command(ResourceKind::Dataset, Some(urn(ResourceKind::Dataset, 2)), titled("d")))
        .await;

    assert_eq!(status(&orphan.unwrap_err()), 400);
    assert_eq!(status(&misplaced.unwrap_err()), 400);
}

/// Without the connector's main catalog no other catalog can be created.
#[tokio::test]
async fn catalogs_wait_for_the_main_one() {
    let mut repo = MockResourceRepoTrait::new();
    repo.expect_get_main().returning(|_| Ok(None));
    let ctx = TestContext::new().resource_repo(repo).build();

    let refused = ctx
        .resources()
        .create(&TestUsers::alone("ana"), &command(ResourceKind::Catalog, None, titled("c")))
        .await;

    assert_eq!(status(&refused.unwrap_err()), 409);
}

/// An access service must exist, be seen and be a data service.
#[tokio::test]
async fn links_must_reach_resources_of_the_right_kind() {
    let mut repo = MockResourceRepoTrait::new();
    repo.expect_get_by_id().returning(|_, _| Ok(Some(dataset(3, 1))));
    repo.expect_get_batch().returning(|_, _| Ok(vec![dataset(9, 1)]));
    let ctx = TestContext::new().resource_repo(repo).build();
    let mut description = titled("dist");
    description.put(Vocab::DCAT_ACCESS_SERVICE, vec![ResourceGraph::iri(urn(ResourceKind::Dataset, 9).as_ref())]);
    description.put(Vocab::DCT_PUBLISHER, vec![ResourceGraph::iri("urn:agent:missing")]);

    let refused = ctx
        .resources()
        .create(&TestUsers::alone("ana"), &command(ResourceKind::Distribution, Some(urn(ResourceKind::Dataset, 3)), description))
        .await
        .unwrap_err();

    assert_eq!(status(&refused), 400);
    let paths: Vec<_> = refused.info().reasons.iter().map(|r| r.path.as_str()).collect();
    assert!(paths.contains(&Vocab::DCAT_ACCESS_SERVICE));
    assert!(paths.contains(&Vocab::DCT_PUBLISHER));
}

/// A data service with no endpoint is refused before the repository is touched.
#[tokio::test]
async fn data_service_needs_an_endpoint() {
    let ctx = TestContext::new().build();

    let refused = ctx
        .resources()
        .create(&TestUsers::alone("ana"), &command(ResourceKind::DataService, Some(urn(ResourceKind::Catalog, 1)), titled("s")))
        .await;

    assert_eq!(status(&refused.unwrap_err()), 400);
}

/// The main catalog is the root's; an existing one is returned, never doubled.
#[tokio::test]
async fn main_catalog_is_the_roots_and_unique() {
    let mut repo = MockResourceRepoTrait::new();
    repo.expect_get_main().returning(|_| Ok(Some(main_catalog(1))));
    let ctx = TestContext::new().resource_repo(repo).build();
    let request = command(ResourceKind::Catalog, None, ResourceGraph::default());

    let denied = ctx.resources().create_main(&TestUsers::alone("ana"), &request).await;
    let existing = ctx.resources().create_main(&TestUsers::root(), &request).await.unwrap();

    assert!(denied.is_err());
    assert_eq!(existing.id(), &urn(ResourceKind::Catalog, 1));
}

/// A stale `If-Match` is a 412 and nothing is written.
#[tokio::test]
async fn stale_version_is_refused() {
    let mut repo = MockResourceRepoTrait::new();
    repo.expect_get_by_id().returning(|_, _| Ok(Some(dataset(3, 1))));
    let ctx = TestContext::new().resource_repo(repo).build();
    let replace = ReplaceResourceCommand {
        description: titled("new"),
        parent: None,
        visibility: None,
        expected_version: Some(7),
    };

    let refused = ctx
        .resources()
        .replace(&TestUsers::root(), ResourceKind::Dataset, &urn(ResourceKind::Dataset, 3), &replace)
        .await;

    assert_eq!(status(&refused.unwrap_err()), 412);
}

/// A replacement keeps the parent, moves the version on and can change the visibility.
#[tokio::test]
async fn replacement_moves_the_version_and_visibility() {
    let mut repo = MockResourceRepoTrait::new();
    repo.expect_get_by_id().returning(|_, _| Ok(Some(dataset(3, 1))));
    repo.expect_replace()
        .withf(|_, r, expected| *expected == 1 && r.version() == 2)
        .returning(|_, r, _| Ok(r.clone()));
    let ctx = TestContext::new().resource_repo(repo).build();
    let replace = ReplaceResourceCommand {
        description: titled("new"),
        parent: Some(urn(ResourceKind::Catalog, 1)),
        visibility: Some(Visibility::Private),
        expected_version: Some(1),
    };

    let replaced = ctx
        .resources()
        .replace(&TestUsers::root(), ResourceKind::Dataset, &urn(ResourceKind::Dataset, 3), &replace)
        .await
        .unwrap();

    assert_eq!(replaced.title().as_deref(), Some("new"));
    assert_eq!(replaced.owner().visibility, Visibility::Private);
}

/// A resource cannot move to another parent.
#[tokio::test]
async fn parent_cannot_change() {
    let mut repo = MockResourceRepoTrait::new();
    repo.expect_get_by_id().returning(|_, _| Ok(Some(dataset(3, 1))));
    let ctx = TestContext::new().resource_repo(repo).build();
    let replace = ReplaceResourceCommand {
        description: titled("new"),
        parent: Some(urn(ResourceKind::Catalog, 5)),
        visibility: None,
        expected_version: None,
    };

    let refused = ctx
        .resources()
        .replace(&TestUsers::root(), ResourceKind::Dataset, &urn(ResourceKind::Dataset, 3), &replace)
        .await;

    assert_eq!(status(&refused.unwrap_err()), 400);
}

/// A lookup for the wrong kind is a missing resource.
#[tokio::test]
async fn wrong_kind_is_not_found() {
    let mut repo = MockResourceRepoTrait::new();
    repo.expect_get_by_id().returning(|_, _| Ok(Some(data_service(2, 1))));
    let ctx = TestContext::new().resource_repo(repo).build();

    let missing = ctx
        .resources()
        .get(&TestUsers::root(), ResourceKind::Dataset, &urn(ResourceKind::DataService, 2))
        .await;

    assert_eq!(status(&missing.unwrap_err()), 404);
}

/// A linked resource cannot go: the conflict names who links to it.
#[tokio::test]
async fn linked_resource_cannot_be_deleted() {
    let mut repo = MockResourceRepoTrait::new();
    repo.expect_get_by_id().returning(|_, _| Ok(Some(data_service(2, 1))));
    repo.expect_delete()
        .returning(|_, _| Err(catalog_agent::entities::errors::CatalogErrors::conflict("fk")));
    repo.expect_referrers()
        .returning(|_| Ok(vec![("urn:distribution:4".to_string(), Vocab::DCAT_ACCESS_SERVICE.to_string())]));
    let ctx = TestContext::new().resource_repo(repo).build();

    let refused = ctx
        .resources()
        .delete(&TestUsers::root(), ResourceKind::DataService, &urn(ResourceKind::DataService, 2), None)
        .await
        .unwrap_err();

    assert_eq!(status(&refused), 409);
    assert!(refused.reason().contains("urn:distribution:4"));
}

/// A document with no `@type` is taken as the route's kind; one of another kind is refused.
#[tokio::test]
async fn documents_are_read_as_the_routes_kind() {
    let mut repo = MockResourceRepoTrait::new();
    repo.expect_insert().returning(|r| Ok(r.clone()));
    let ctx = TestContext::new().resource_repo(repo).build();
    let user = TestUsers::alone("ana");

    let agent = ctx
        .resources()
        .create_document(&user, ResourceKind::Agent, json!({"name": {"en": "ACME"}}), None, false)
        .await
        .unwrap();
    let refused = ctx
        .resources()
        .create_document(&user, ResourceKind::Agent, json!({"@context": EUNOMIA_CATALOG_CONTEXT_URL, "@type": "Dataset"}), None, false)
        .await;

    assert_eq!(agent.kind(), ResourceKind::Agent);
    assert_eq!(status(&refused.unwrap_err()), 400);
}
