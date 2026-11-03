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
//! CatalogPublicResolver over mocked schema and resource repositories, with the real schema
//! service.

use std::str::FromStr;

use catalog_agent::data::repo::resource::MockResourceRepoTrait;
use catalog_agent::data::repo::schema::MockSchemaRepoTrait;
use catalog_agent::entities::resource_kind::ResourceKind;
use catalog_agent::entities::schemas::DataSchema;
use catalog_agent::facades::CatalogPublicResolver;
use common::public_refs::{PublicQuery, PublicResolverTrait};
use serde_json::json;
use urn::Urn;
use ymir::oauth::OwnerScope;
use ymir::types::oauth::RolePath;
use ymir::types::participants::Visibility;

use crate::support::builders::{provider_set, urn};
use crate::support::context::TestContext;

const SCHEMA: &str = "urn:schema:readings";

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

fn query(version: Option<i32>) -> PublicQuery {
    PublicQuery {
        profile: None,
        version,
    }
}

/// Versions 1 to 3 of the schema.
fn schemas() -> MockSchemaRepoTrait {
    let mut repo = MockSchemaRepoTrait::new();
    repo.expect_get_versions()
        .withf(|scope, id| id == SCHEMA && matches!(scope, OwnerScope::All))
        .returning(|_, _| Ok(vec![schema(1), schema(2), schema(3)]));
    repo
}

/// A published dataset uses every version.
fn used() -> MockResourceRepoTrait {
    let mut repo = MockResourceRepoTrait::new();
    repo.expect_get_all()
        .withf(|scope, filter| matches!(scope, OwnerScope::All) && filter.schema.is_some())
        .returning(|_, _| {
            let (_, set) = provider_set();
            Ok(set.get(urn(ResourceKind::Dataset, 3).as_ref()).cloned().into_iter().collect())
        });
    repo
}

/// Catalog resources and their offers go through DSP; neither they nor unknown URNs resolve here.
#[tokio::test]
async fn only_schemas_are_resolved() {
    let resolver = CatalogPublicResolver::new(TestContext::new().build());
    let dataset = resolver
        .resolve(&urn(ResourceKind::Dataset, 3), &PublicQuery::default())
        .await
        .unwrap();
    let unknown = resolver
        .resolve(&Urn::from_str("urn:agreement:1").unwrap(), &PublicQuery::default())
        .await
        .unwrap();

    assert!(dataset.is_none());
    assert!(unknown.is_none());
}

/// A schema URN is its newest version a published resource uses, located at its versioned URL.
#[tokio::test]
async fn schema_is_its_latest_version() {
    let ctx = TestContext::new()
        .schema_repo(schemas())
        .resource_repo(used())
        .build();

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
    let ctx = TestContext::new()
        .schema_repo(schemas())
        .resource_repo(used())
        .build();

    let document = CatalogPublicResolver::new(ctx)
        .resolve(&Urn::from_str(SCHEMA).unwrap(), &query(Some(1)))
        .await
        .unwrap()
        .unwrap();

    assert!(document.location.unwrap().ends_with("/1"));
}
