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

//! SchemaRouter and PublicSchemaRouter over a mocked schema service.

use axum::body::Body;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::Response;
use catalog_agent::http::schemas::{PublicSchemaRouter, SchemaRouter};
use catalog_agent::services::schemas::MockSchemaServiceTrait;
use common::test_utils::scopes::TestUsers;
use serde_json::{json, Value};
use tower::ServiceExt;
use ymir::errors::Errors;

use crate::support::context::TestContext;

async fn body(response: Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

/// A version is served as an immutable `application/schema+json` anyone may fetch.
#[tokio::test]
async fn public_version_is_a_cacheable_json_schema() {
    let mut service = MockSchemaServiceTrait::new();
    service
        .expect_published()
        .withf(|id, version| id == "urn:schema:a" && *version == 2)
        .returning(|_, _| Ok(json!({ "$id": "https://x/2", "type": "object" })));
    let router = PublicSchemaRouter::new(TestContext::new().schemas(service).build()).router();

    let response = router
        .oneshot(
            Request::builder()
                .uri("/schemas/urn:schema:a/v2")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()["content-type"],
        "application/schema+json"
    );
    assert!(response.headers()["cache-control"]
        .to_str()
        .unwrap()
        .contains("immutable"));
    assert_eq!(response.headers()["access-control-allow-origin"], "*");
    assert_eq!(body(response).await["type"], "object");
}

/// A version that is not a number, or not public, is not served.
#[tokio::test]
async fn public_version_errors() {
    let mut service = MockSchemaServiceTrait::new();
    service
        .expect_published()
        .returning(|id, _| Err(Errors::missing_resource(id, "schema not found", None)));
    let router = PublicSchemaRouter::new(TestContext::new().schemas(service).build()).router();

    let bad = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/schemas/urn:schema:a/latest")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let missing = router
        .oneshot(
            Request::builder()
                .uri("/schemas/urn:schema:a/1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(bad.status(), StatusCode::BAD_REQUEST);
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
}

/// Inference answers the inferred schema to an authenticated caller.
#[tokio::test]
async fn infer_answers_the_schema() {
    let mut service = MockSchemaServiceTrait::new();
    service
        .expect_infer()
        .withf(|input| input.samples.len() == 1)
        .returning(|_| Ok(json!({ "type": "object" })));
    let router = SchemaRouter::new(TestContext::new().schemas(service).build()).router();

    let mut request = Request::post("/infer")
        .header("content-type", "application/json")
        .body(Body::from(json!({ "samples": [{ "a": 1 }] }).to_string()))
        .unwrap();
    request.extensions_mut().insert(TestUsers::alone("ana"));
    let response = router.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body(response).await["type"], "object");
}
