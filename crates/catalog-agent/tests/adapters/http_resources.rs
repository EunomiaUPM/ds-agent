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

//! ResourceRouter over a mocked resource service and doctor: JSON-LD bodies, metadata headers.

use axum::body::Body;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::Response;
use catalog_agent::entities::resource_kind::ResourceKind;
use catalog_agent::http::resources::ResourceRouter;
use catalog_agent::services::doctor::views::DoctorReport;
use catalog_agent::services::doctor::MockDoctorServiceTrait;
use catalog_agent::services::resources::MockResourceServiceTrait;
use common::eunomia::EUNOMIA_CATALOG_CONTEXT_URL;
use common::paginated_spec::Paginated;
use common::test_utils::scopes::TestUsers;
use serde_json::{json, Value};
use tower::ServiceExt;
use ymir::types::participants::Visibility;

use crate::support::builders::{dataset, id};
use crate::support::context::TestContext;

async fn send(router: axum::Router, method: &str, uri: &str, headers: &[(&str, &str)], body: Option<Value>) -> Response {
    let mut request = Request::builder().method(method).uri(uri);
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    let body = match body {
        Some(body) => {
            request = request.header("content-type", "application/json");
            Body::from(body.to_string())
        }
        None => Body::empty(),
    };
    let mut request = request.body(body).unwrap();
    request.extensions_mut().insert(TestUsers::alone("ana"));
    router.oneshot(request).await.unwrap()
}

async fn json(response: Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

/// A created dataset answers 201 with its JSON-LD and its version, visibility and owner.
#[tokio::test]
async fn create_answers_the_document_and_its_metadata() {
    let mut service = MockResourceServiceTrait::new();
    service
        .expect_create_document()
        .withf(|_, kind, body, visibility, main| {
            *kind == ResourceKind::Dataset && body["title"]["en"] == "Readings" && *visibility == Some(Visibility::Public) && !main
        })
        .returning(|_, _, _, _, _| Ok(dataset(3, 1)));
    let ctx = TestContext::new().resources(service).build();
    let router = ResourceRouter::new(ctx, ResourceKind::Dataset).router();

    let response = send(
        router,
        "POST",
        "/",
        &[("eunomia-visibility", "Public")],
        Some(json!({"@context": EUNOMIA_CATALOG_CONTEXT_URL, "@type": "Dataset", "title": {"en": "Readings"}})),
    )
    .await;

    assert_eq!(response.status(), StatusCode::CREATED);
    let headers = response.headers().clone();
    assert_eq!(headers["content-type"], "application/ld+json");
    assert_eq!(headers["etag"], "W/\"1\"");
    assert_eq!(headers["eunomia-visibility"], "Public");
    let body = json(response).await;
    assert_eq!(body["@id"], id(ResourceKind::Dataset, 3));
    assert_eq!(body["inCatalog"], id(ResourceKind::Catalog, 1));
}

/// A list is one `@graph`, its total and next cursor in headers.
#[tokio::test]
async fn list_is_a_graph_with_paging_headers() {
    let mut service = MockResourceServiceTrait::new();
    service
        .expect_list()
        .withf(|_, filter, _, _| filter.kind == Some(ResourceKind::Dataset) && filter.title.as_deref() == Some("read"))
        .returning(|_, _, _, _| Ok(Paginated::new(vec![dataset(3, 1)], Some("next".into()), Some(9))));
    let ctx = TestContext::new().resources(service).build();
    let router = ResourceRouter::new(ctx, ResourceKind::Dataset).router();

    let response = send(router, "GET", "/?title=read", &[], None).await;

    assert_eq!(response.headers()["x-total-count"], "9");
    assert_eq!(response.headers()["x-next-cursor"], "next");
    let body = json(response).await;
    assert_eq!(body["@graph"][0]["@id"], id(ResourceKind::Dataset, 3));
}

/// `If-Match` reaches the service as the expected version.
#[tokio::test]
async fn if_match_is_the_expected_version() {
    let mut service = MockResourceServiceTrait::new();
    service
        .expect_replace_document()
        .withf(|_, _, _, _, _, expected| *expected == Some(4))
        .returning(|_, _, _, _, _, _| Ok(dataset(3, 1)));
    let ctx = TestContext::new().resources(service).build();
    let router = ResourceRouter::new(ctx, ResourceKind::Dataset).router();

    let response = send(
        router,
        "PUT",
        &format!("/{}", id(ResourceKind::Dataset, 3)),
        &[("if-match", "W/\"4\"")],
        Some(json!({"title": {"en": "renamed"}})),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
}

/// A visibility header that names no visibility is a bad request.
#[tokio::test]
async fn unknown_visibility_is_refused() {
    let ctx = TestContext::new().resources(MockResourceServiceTrait::new()).build();
    let router = ResourceRouter::new(ctx, ResourceKind::Dataset).router();

    let response = send(router, "POST", "/", &[("eunomia-visibility", "Secret")], Some(json!({}))).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

/// The doctor route answers the report of the resource it names.
#[tokio::test]
async fn doctor_route_answers_the_report() {
    let mut doctor = MockDoctorServiceTrait::new();
    doctor
        .expect_diagnose()
        .withf(|_, kind, _, profile, deep| *kind == ResourceKind::Dataset && profile.as_deref() == Some("dcat-ap") && *deep)
        .returning(|_, _, id, _, _| {
            Ok(DoctorReport {
                resource: id.to_string(),
                kind: "Dataset".into(),
                checked: vec![id.to_string()],
                profiles: vec![],
            })
        });
    let ctx = TestContext::new().doctor(doctor).build();
    let router = ResourceRouter::new(ctx, ResourceKind::Dataset).router();

    let response = send(router, "GET", &format!("/{}/doctor?profile=dcat-ap&deep=true", id(ResourceKind::Dataset, 3)), &[], None).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(json(response).await["resource"], id(ResourceKind::Dataset, 3));
}

/// Only catalogs and data services have a main route; distributions answer by dataset and format.
#[tokio::test]
async fn kind_specific_routes() {
    let mut service = MockResourceServiceTrait::new();
    service
        .expect_distribution_by_format()
        .withf(|_, _, format| format == "http+pull")
        .returning(|_, _, _| Ok(crate::support::builders::distribution(4, 3, 2, "http+pull")));
    let ctx = TestContext::new().resources(service).build();

    let main = send(ResourceRouter::new(ctx.clone(), ResourceKind::Dataset).router(), "GET", "/main", &[], None).await;
    let by_format = send(
        ResourceRouter::new(ctx, ResourceKind::Distribution).router(),
        "GET",
        &format!("/dataset/{}/format/http+pull", id(ResourceKind::Dataset, 3)),
        &[],
        None,
    )
    .await;

    assert_ne!(main.status(), StatusCode::OK);
    assert_eq!(by_format.status(), StatusCode::OK);
}
