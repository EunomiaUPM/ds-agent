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

//! CatalogPublicResolver over a mocked closure service and offer and schema repositories, with
//! the real projection, offer and schema services.

use std::str::FromStr;

use catalog_agent::data::repo::odrl_offer::MockOdrlOfferRepoTrait;
use catalog_agent::data::repo::schema::MockSchemaRepoTrait;
use catalog_agent::entities::resource::CatalogResource;
use catalog_agent::entities::resource_kind::ResourceKind;
use catalog_agent::entities::resource_set::ResourceSet;
use catalog_agent::entities::schemas::DataSchema;
use catalog_agent::facades::CatalogPublicResolver;
use catalog_agent::services::closure::MockClosureServiceTrait;
use common::public_refs::{PublicQuery, PublicResolverTrait};
use serde_json::json;
use urn::Urn;
use ymir::oauth::OwnerScope;
use ymir::types::oauth::RolePath;
use ymir::types::participants::Visibility;

use crate::support::builders::{offer, provider_set, provider_set_with_draft, urn};
use crate::support::context::TestContext;

const SCHEMA: &str = "urn:schema:readings";

fn closures(set: fn() -> (CatalogResource, ResourceSet)) -> MockClosureServiceTrait {
    let mut closures = MockClosureServiceTrait::new();
    closures
        .expect_resource()
        .withf(|scope, _| matches!(scope, OwnerScope::Seeing { user_id, .. } if user_id == "urn:eunomia:public-reader"))
        .returning(move |_, id| {
            let (_, set) = set();
            Ok(set.get(id.as_ref()).cloned().map(|root| (root, set)))
        });
    closures
}

fn resolver(closures: MockClosureServiceTrait) -> CatalogPublicResolver {
    let ctx = TestContext::new()
        .closures(closures)
        .offer_repo(offers())
        .build();
    CatalogPublicResolver::new(ctx)
}

fn schema(version: i32) -> DataSchema {
    DataSchema {
        id: SCHEMA.to_string(),
        version,
        user_id: "ana".to_string(),
        user_role: "/ana".parse::<RolePath>().unwrap(),
        visibility: Visibility::Public,
        created_at: chrono::Utc::now().into(),
        title: "Readings".to_string(),
        description: None,
        content: json!({ "type": "object" }),
        sha256: "00".to_string(),
    }
}

fn query(profile: Option<&str>, version: Option<i32>) -> PublicQuery {
    PublicQuery {
        profile: profile.map(str::to_string),
        version,
    }
}

/// A dataset comes in DCAT 3 by default, with its profile and the agent's own route.
#[tokio::test]
async fn dataset_defaults_to_dcat3() {
    let id = urn(ResourceKind::Dataset, 3);
    let document = resolver(closures(provider_set))
        .resolve(&id, &PublicQuery::default())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(document.media_type, "application/ld+json");
    assert!(document.profile.unwrap().contains("dcat"));
    assert!(document
        .location
        .unwrap()
        .ends_with(&format!("/public/dcat3/resources/{id}")));
}

/// `?profile=` picks another registered profile.
#[tokio::test]
async fn profile_query_picks_the_profile() {
    let id = urn(ResourceKind::Dataset, 3);
    let document = resolver(closures(provider_set))
        .resolve(&id, &query(Some("dcat-ap"), None))
        .await
        .unwrap()
        .unwrap();

    assert!(document
        .location
        .unwrap()
        .contains("/public/dcat-ap/resources/"));
}

/// A draft and a URN the catalog does not hold are both absent.
#[tokio::test]
async fn drafts_and_unknown_urns_are_absent() {
    let draft = resolver(closures(provider_set_with_draft))
        .resolve(&urn(ResourceKind::Dataset, 3), &PublicQuery::default())
        .await
        .unwrap();
    let unknown = resolver(closures(provider_set))
        .resolve(
            &Urn::from_str("urn:agreement:1").unwrap(),
            &PublicQuery::default(),
        )
        .await
        .unwrap();

    assert!(draft.is_none());
    assert!(unknown.is_none());
}

/// An unknown profile is a bad request.
#[tokio::test]
async fn unknown_profile_is_rejected() {
    let error = resolver(MockClosureServiceTrait::new())
        .resolve(&urn(ResourceKind::Dataset, 3), &query(Some("nope"), None))
        .await
        .unwrap_err();

    assert_eq!(error.info().status_code.as_u16(), 400);
}

/// A schema URN is its newest public version as JSON Schema, located at its versioned URL.
#[tokio::test]
async fn schema_is_its_latest_version() {
    let mut repo = MockSchemaRepoTrait::new();
    repo.expect_get_latest()
        .withf(|scope, id| id == SCHEMA && matches!(scope, OwnerScope::Seeing { user_id, .. } if user_id == "urn:eunomia:public-reader"))
        .returning(|_, _| Ok(Some(schema(3))));
    let ctx = TestContext::new().schema_repo(repo).build();

    let document = CatalogPublicResolver::new(ctx)
        .resolve(&Urn::from_str(SCHEMA).unwrap(), &PublicQuery::default())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(document.media_type, "application/schema+json");
    assert!(document
        .location
        .as_deref()
        .unwrap()
        .ends_with(&format!("/schemas/{SCHEMA}/3")));
    assert_eq!(document.body["title"], "Readings");
}

/// `?version=` asks for that version.
#[tokio::test]
async fn schema_version_query_is_honoured() {
    let mut repo = MockSchemaRepoTrait::new();
    repo.expect_get_version()
        .withf(|_, _, v| *v == 1)
        .returning(|_, _, v| Ok(Some(schema(v))));
    let ctx = TestContext::new().schema_repo(repo).build();

    let document = CatalogPublicResolver::new(ctx)
        .resolve(&Urn::from_str(SCHEMA).unwrap(), &query(None, Some(1)))
        .await
        .unwrap()
        .unwrap();

    assert!(document.location.unwrap().ends_with("/1"));
}

/// The offers repository holding offer 1 on dataset 3, read as the public.
fn offers() -> MockOdrlOfferRepoTrait {
    let mut repo = MockOdrlOfferRepoTrait::new();
    repo.expect_get_by_id()
        .withf(|scope, _| matches!(scope, OwnerScope::Seeing { user_id, .. } if user_id == "urn:eunomia:public-reader"))
        .returning(|_, id| {
            let stored = offer(1, &urn(ResourceKind::Dataset, 3));
            Ok((&stored.id == id).then_some(stored))
        });
    repo
}

/// An offer on a public dataset is its ODRL document with its target and the DSP context.
#[tokio::test]
async fn offer_on_a_public_dataset_is_dereferenced() {
    let id = offer(1, &urn(ResourceKind::Dataset, 3)).id;
    let document = resolver(closures(provider_set))
        .resolve(&id, &PublicQuery::default())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(document.media_type, "application/ld+json");
    assert_eq!(document.body["@id"], id.to_string());
    assert_eq!(
        document.body["target"],
        urn(ResourceKind::Dataset, 3).to_string()
    );
    assert!(document.body["@context"]
        .to_string()
        .contains("dspace/2025/1"));
    assert!(document.body.get("permission").is_some());
}

/// An offer whose dataset is a draft is absent, like the dataset.
#[tokio::test]
async fn offer_on_a_draft_is_absent() {
    let id = offer(1, &urn(ResourceKind::Dataset, 3)).id;
    let document = resolver(closures(provider_set_with_draft))
        .resolve(&id, &PublicQuery::default())
        .await
        .unwrap();

    assert!(document.is_none());
}
