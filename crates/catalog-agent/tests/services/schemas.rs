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

//! SchemaService over mocked schema and resource repositories: versions, checks, usage and the
//! public view.

use catalog_agent::data::repo::resource::MockResourceRepoTrait;
use catalog_agent::data::repo::schema::MockSchemaRepoTrait;
use catalog_agent::entities::resource_kind::ResourceKind;
use catalog_agent::entities::schemas::{
    CheckSchemaUrlDto, DataSchema, InferSchemaDto, NewSchemaDto, NewSchemaVersionDto,
    ValidateSchemaDto,
};
use common::test_utils::scopes::TestUsers;
use serde_json::{json, Value};
use ymir::errors::Errors;
use ymir::oauth::OwnerScope;
use ymir::types::oauth::RolePath;
use ymir::types::participants::Visibility;

use crate::support::builders::dataset;
use crate::support::context::TestContext;

const ID: &str = "urn:schema:readings";

fn status(error: &Errors) -> u16 {
    error.info().status_code.as_u16()
}

fn stored(version: i32, visibility: Visibility) -> DataSchema {
    DataSchema {
        id: ID.to_string(),
        version,
        user_id: "ana".to_string(),
        user_role: "/ana".parse::<RolePath>().unwrap(),
        visibility,
        created_at: chrono::Utc::now().into(),
        title: "Readings".to_string(),
        description: Some("Hourly readings".to_string()),
        content: json!({ "type": "object", "properties": { "t": { "type": "integer" } } }),
        sha256: "00".to_string(),
    }
}

fn readings() -> Value {
    json!({ "$id": "https://elsewhere.example/x", "type": "object" })
}

/// A new schema is version 1 of a fresh URN, private, hashed and without the `$id` it came with.
#[tokio::test]
async fn create_stores_version_one() {
    let mut repo = MockSchemaRepoTrait::new();
    repo.expect_insert()
        .withf(|c| {
            c.version == 1
                && c.id.starts_with("urn:schema:")
                && c.content.get("$id").is_none()
                && c.sha256.len() == 64
                && c.owner.visibility == Visibility::Private
        })
        .returning(|c| {
            let mut schema = stored(c.version, c.owner.visibility.clone());
            schema.id = c.id.clone();
            Ok(schema)
        });
    let ctx = TestContext::new().schema_repo(repo).build();

    let view = ctx
        .schemas()
        .create(
            &TestUsers::alone("ana"),
            &NewSchemaDto {
                title: "Readings".into(),
                description: None,
                schema: readings(),
                visibility: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(view.version, 1);
    assert!(view
        .url
        .ends_with(&format!("/catalog-agent/public/schemas/{}/1", view.id)));
}

/// An invalid schema or a blank title is refused before anything is stored.
#[tokio::test]
async fn create_refuses_invalid_input() {
    let ctx = TestContext::new().build();
    let user = TestUsers::alone("ana");
    let invalid = NewSchemaDto {
        title: "Bad".into(),
        description: None,
        schema: json!({ "type": "nonsense" }),
        visibility: None,
    };
    let untitled = NewSchemaDto {
        title: " ".into(),
        description: None,
        schema: readings(),
        visibility: None,
    };

    assert_eq!(
        status(&ctx.schemas().create(&user, &invalid).await.unwrap_err()),
        400
    );
    assert_eq!(
        status(&ctx.schemas().create(&user, &untitled).await.unwrap_err()),
        400
    );
}

/// A new version follows the latest, keeps its owner and title, and leaves the rest untouched.
#[tokio::test]
async fn add_version_follows_the_latest() {
    let mut repo = MockSchemaRepoTrait::new();
    repo.expect_get_versions()
        .withf(|scope, id| matches!(scope, OwnerScope::Acting { .. }) && id == ID)
        .returning(|_, _| {
            Ok(vec![
                stored(2, Visibility::Public),
                stored(1, Visibility::Public),
            ])
        });
    repo.expect_insert()
        .withf(|c| {
            c.version == 3 && c.title == "Readings" && c.owner.visibility == Visibility::Public
        })
        .returning(|c| Ok(stored(c.version, c.owner.visibility.clone())));
    let ctx = TestContext::new().schema_repo(repo).build();

    let view = ctx
        .schemas()
        .add_version(
            &TestUsers::alone("ana"),
            ID,
            &NewSchemaVersionDto {
                schema: readings(),
                title: None,
                description: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(view.version, 3);
}

/// A version of a schema the user cannot act on is not found.
#[tokio::test]
async fn add_version_needs_the_schema() {
    let mut repo = MockSchemaRepoTrait::new();
    repo.expect_get_versions().returning(|_, _| Ok(vec![]));
    let ctx = TestContext::new().schema_repo(repo).build();

    let error = ctx
        .schemas()
        .add_version(
            &TestUsers::alone("bob"),
            ID,
            &NewSchemaVersionDto {
                schema: readings(),
                title: None,
                description: None,
            },
        )
        .await
        .unwrap_err();

    assert_eq!(status(&error), 404);
}

/// A version some resource points to is a 409 and stays stored.
#[tokio::test]
async fn delete_refuses_a_used_version() {
    let mut schemas = MockSchemaRepoTrait::new();
    schemas
        .expect_get_versions()
        .returning(|_, _| Ok(vec![stored(1, Visibility::Public)]));
    let mut resources = MockResourceRepoTrait::new();
    resources
        .expect_count()
        .withf(|scope, f| {
            matches!(scope, OwnerScope::All)
                && f.schema.as_deref().is_some_and(|u| u.ends_with("/1"))
        })
        .returning(|_, _| Ok(2));
    let ctx = TestContext::new()
        .schema_repo(schemas)
        .resource_repo(resources)
        .build();

    let error = ctx
        .schemas()
        .delete(&TestUsers::alone("ana"), ID, Some(1))
        .await
        .unwrap_err();

    assert_eq!(status(&error), 409);
}

/// An unused schema is deleted whole.
#[tokio::test]
async fn delete_removes_an_unused_schema() {
    let mut schemas = MockSchemaRepoTrait::new();
    schemas.expect_get_versions().returning(|_, _| {
        Ok(vec![
            stored(2, Visibility::Public),
            stored(1, Visibility::Public),
        ])
    });
    schemas
        .expect_delete()
        .withf(|_, id, version| id == ID && version.is_none())
        .returning(|_, _, _| Ok(()));
    let mut resources = MockResourceRepoTrait::new();
    resources.expect_count().times(2).returning(|_, _| Ok(0));
    let ctx = TestContext::new()
        .schema_repo(schemas)
        .resource_repo(resources)
        .build();

    ctx.schemas()
        .delete(&TestUsers::alone("ana"), ID, None)
        .await
        .unwrap();
}

/// A version comes with its published content and the resources using it.
#[tokio::test]
async fn version_shows_content_and_usage() {
    let mut schemas = MockSchemaRepoTrait::new();
    schemas
        .expect_get_version()
        .returning(|_, _, v| Ok(Some(stored(v, Visibility::Public))));
    let mut resources = MockResourceRepoTrait::new();
    resources
        .expect_get_page()
        .withf(|_, f, _, _| f.schema.is_some())
        .returning(|_, _, _, _| Ok(vec![dataset(3, 1)]));
    let ctx = TestContext::new()
        .schema_repo(schemas)
        .resource_repo(resources)
        .build();

    let view = ctx
        .schemas()
        .version(&TestUsers::alone("ana"), ID, 1)
        .await
        .unwrap();

    assert_eq!(view.schema["$id"], view.summary.url);
    assert_eq!(view.used_by.len(), 1);
    assert_eq!(view.used_by[0].kind, ResourceKind::Dataset.as_str());
}

/// The public view reads as nobody in particular and names its own URL.
#[tokio::test]
async fn published_is_read_as_the_public() {
    let mut repo = MockSchemaRepoTrait::new();
    repo.expect_get_version()
        .withf(|scope, _, _| matches!(scope, OwnerScope::Seeing { user_id, .. } if user_id == "urn:eunomia:public-reader"))
        .returning(|_, _, v| Ok(Some(stored(v, Visibility::Public))));
    let ctx = TestContext::new().schema_repo(repo).build();

    let schema = ctx.schemas().published(ID, 1).await.unwrap();

    assert_eq!(schema["$id"], ctx.schemas().url(ID, 1));
    assert_eq!(
        schema["$schema"],
        "https://json-schema.org/draft/2020-12/schema"
    );
    assert_eq!(schema["title"], "Readings");
    assert_eq!(schema["description"], "Hourly readings");
}

/// A version the public does not see is not found.
#[tokio::test]
async fn published_hides_what_the_public_cannot_see() {
    let mut repo = MockSchemaRepoTrait::new();
    repo.expect_get_version().returning(|_, _, _| Ok(None));
    let ctx = TestContext::new().schema_repo(repo).build();

    assert_eq!(
        status(&ctx.schemas().published(ID, 1).await.unwrap_err()),
        404
    );
}

/// Inference needs samples and returns a schema they satisfy.
#[tokio::test]
async fn infer_and_validate_samples() {
    let ctx = TestContext::new().build();
    let samples = vec![json!({ "t": 1 }), json!({ "t": 2, "q": "ok" })];

    assert_eq!(
        status(
            &ctx.schemas()
                .infer(&InferSchemaDto { samples: vec![] })
                .unwrap_err()
        ),
        400
    );
    let schema = ctx
        .schemas()
        .infer(&InferSchemaDto {
            samples: samples.clone(),
        })
        .unwrap();
    let report = ctx
        .schemas()
        .validate(&ValidateSchemaDto {
            schema,
            samples: vec![samples[0].clone(), json!({ "q": 1 })],
        })
        .unwrap();

    assert!(!report.valid);
    assert!(report.samples[0].valid);
    assert!(!report.samples[1].valid);
}

/// Only absolute http(s) URLs are fetched.
#[tokio::test]
async fn check_url_refuses_other_schemes() {
    let ctx = TestContext::new().build();

    for url in ["file:///etc/passwd", "not a url", "/relative"] {
        let error = ctx
            .schemas()
            .check_url(&CheckSchemaUrlDto { url: url.into() })
            .await
            .unwrap_err();
        assert_eq!(status(&error), 400, "{url}");
    }
}

/// A resource may point to an external schema, or to a hosted version the user sees.
#[tokio::test]
async fn references_to_hosted_versions_must_exist() {
    let mut repo = MockSchemaRepoTrait::new();
    repo.expect_get_version()
        .returning(|_, _, v| Ok((v == 1).then(|| stored(1, Visibility::Public))));
    let ctx = TestContext::new().schema_repo(repo).build();
    let user = TestUsers::alone("ana");
    let schemas = ctx.schemas();

    schemas
        .check_reference(&user, "https://example.org/s.json")
        .await
        .unwrap();
    schemas
        .check_reference(&user, &schemas.url(ID, 1))
        .await
        .unwrap();
    let missing = schemas
        .check_reference(&user, &schemas.url(ID, 7))
        .await
        .unwrap_err();
    assert_eq!(status(&missing), 400);
}
