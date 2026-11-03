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

//! The public references router over mocked resolvers, and the remote resolver against that
//! router served on a local port.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::Router;
use serde_json::{json, Value};
use tower::ServiceExt;
use urn::Urn;

use crate::public_refs::{
    MockPublicResolverTrait, PublicDocument, PublicQuery, PublicRefsRouter, PublicResolverTrait,
    RemotePublicResolver,
};
use crate::routes;

const BASE: &str = "http://agent.example:1200";

fn resolver(nid: &'static str, body: Value) -> Arc<dyn PublicResolverTrait> {
    let mut mock = MockPublicResolverTrait::new();
    mock.expect_resolve().returning(move |urn, _| {
        Ok((urn.nid() == nid).then(|| {
            PublicDocument::json_ld(body.clone())
                .with_profile("http://www.w3.org/ns/dcat3")
                .with_location("/owner/path")
        }))
    });
    Arc::new(mock)
}

fn app_at(base: &str, resolvers: Vec<Arc<dyn PublicResolverTrait>>) -> Router {
    let router = PublicRefsRouter::new(base, resolvers).router();
    Router::new().nest(routes::public_refs::PREFIX, router)
}

fn app(resolvers: Vec<Arc<dyn PublicResolverTrait>>) -> Router {
    app_at(BASE, resolvers)
}

/// `app` served on a free local port; returns its base URL.
async fn serve(resolvers: Vec<Arc<dyn PublicResolverTrait>>) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let app = app_at(&base, resolvers);
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    base
}

fn urn(s: &str) -> Urn {
    s.parse().unwrap()
}

async fn get(
    app: Router,
    uri: &str,
    headers: &[(header::HeaderName, &str)],
) -> axum::response::Response {
    let mut request = Request::builder().uri(uri);
    for (name, value) in headers {
        request = request.header(name, *value);
    }
    app.oneshot(request.body(Body::empty()).unwrap())
        .await
        .unwrap()
}

async fn body(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), 4096)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

/// The resolver that holds the URN answers, after those that do not.
#[tokio::test]
async fn first_resolver_holding_the_urn_answers() {
    let app = app(vec![
        resolver("dataset", json!({ "@id": "wrong" })),
        resolver("agreement", json!({ "@id": "urn:agreement:1" })),
    ]);
    let response = get(app, "/open/urn:agreement:1", &[]).await;
    assert_eq!(response.status(), StatusCode::OK);
    let headers = response.headers();
    assert_eq!(headers[header::CONTENT_TYPE], "application/ld+json");
    let links: Vec<_> = headers.get_all(header::LINK).iter().collect();
    assert_eq!(links[0], "<http://www.w3.org/ns/dcat3>; rel=\"profile\"");
    assert_eq!(
        links[1],
        "<http://agent.example:1200/open/urn:agreement:1>; rel=\"canonical\""
    );
    assert_eq!(
        headers[header::CONTENT_LOCATION],
        "http://agent.example:1200/owner/path"
    );
    assert_eq!(headers[header::ACCESS_CONTROL_ALLOW_ORIGIN], "*");
    assert_eq!(body(response).await, json!({ "@id": "urn:agreement:1" }));
}

/// A URN no resolver holds is a 404.
#[tokio::test]
async fn unknown_urn_is_not_found() {
    let response = get(
        app(vec![resolver("dataset", json!({}))]),
        "/open/urn:agreement:1",
        &[],
    )
    .await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

/// Something that is not a URN is a bad request, never asked to the resolvers.
#[tokio::test]
async fn non_urn_is_bad_request() {
    let response = get(
        app(vec![Arc::new(MockPublicResolverTrait::new())]),
        "/open/not-a-urn",
        &[],
    )
    .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

/// A URN with slashes reaches the resolver whole.
#[tokio::test]
async fn urn_with_slashes_is_taken_whole() {
    let mut mock = MockPublicResolverTrait::new();
    mock.expect_resolve()
        .withf(|urn, _| urn.as_str() == "urn:schema:a/b")
        .returning(|_, _| Ok(Some(PublicDocument::json_ld(json!({})))));
    let response = get(app(vec![Arc::new(mock)]), "/open/urn:schema:a/b", &[]).await;
    assert_eq!(response.status(), StatusCode::OK);
}

/// The query reaches the resolver.
#[tokio::test]
async fn query_reaches_the_resolver() {
    let mut mock = MockPublicResolverTrait::new();
    mock.expect_resolve()
        .withf(|_, q| q.profile.as_deref() == Some("dcat-ap") && q.version == Some(2))
        .returning(|_, _| Ok(Some(PublicDocument::json_ld(json!({})))));
    let response = get(
        app(vec![Arc::new(mock)]),
        "/open/urn:dataset:1?profile=dcat-ap&version=2",
        &[],
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
}

/// A matching `If-None-Match` is a 304.
#[tokio::test]
async fn matching_etag_is_not_modified() {
    let tag = PublicDocument::json_ld(json!({ "@id": "urn:agreement:1" })).etag();
    let app = app(vec![resolver(
        "agreement",
        json!({ "@id": "urn:agreement:1" }),
    )]);
    let response = get(
        app,
        "/open/urn:agreement:1",
        &[(header::IF_NONE_MATCH, &tag)],
    )
    .await;
    assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
}

/// A client accepting only JSON gets the same body as `application/json`.
#[tokio::test]
async fn json_only_client_gets_plain_json() {
    let app = app(vec![resolver("agreement", json!({}))]);
    let response = get(
        app,
        "/open/urn:agreement:1",
        &[(header::ACCEPT, "application/json")],
    )
    .await;
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");
}

/// A location the resolver gives as an absolute URL is kept as it is.
#[tokio::test]
async fn absolute_location_is_kept() {
    let mut mock = MockPublicResolverTrait::new();
    mock.expect_resolve().returning(|_, _| {
        Ok(Some(
            PublicDocument::json_ld(json!({})).with_location("http://catalog:1201/x"),
        ))
    });
    let response = get(app(vec![Arc::new(mock)]), "/open/urn:dataset:1", &[]).await;
    assert_eq!(
        response.headers()[header::CONTENT_LOCATION],
        "http://catalog:1201/x"
    );
}

/// The remote resolver reads the agent's document with its media type, profile and location.
#[tokio::test]
async fn remote_reads_the_agents_document() {
    let base = serve(vec![resolver(
        "agreement",
        json!({ "@id": "urn:agreement:1" }),
    )])
    .await;
    let document = RemotePublicResolver::new(&base)
        .resolve(&urn("urn:agreement:1"), &PublicQuery::default())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(document.body, json!({ "@id": "urn:agreement:1" }));
    assert_eq!(document.media_type, "application/ld+json");
    assert_eq!(
        document.profile.as_deref(),
        Some("http://www.w3.org/ns/dcat3")
    );
    assert_eq!(document.location, Some(format!("{base}/owner/path")));
}

/// The remote resolver passes the query on.
#[tokio::test]
async fn remote_passes_the_query() {
    let mut mock = MockPublicResolverTrait::new();
    mock.expect_resolve()
        .withf(|_, q| q.profile.as_deref() == Some("dcat-ap") && q.version == Some(2))
        .returning(|_, _| Ok(Some(PublicDocument::json_ld(json!({})))));
    let base = serve(vec![Arc::new(mock)]).await;
    let query = PublicQuery {
        profile: Some("dcat-ap".into()),
        version: Some(2),
    };

    let document = RemotePublicResolver::new(base)
        .resolve(&urn("urn:dataset:1"), &query)
        .await
        .unwrap();
    assert!(document.is_some());
}

/// A URN the agent does not hold is absent, and a bad request stays one.
#[tokio::test]
async fn remote_maps_not_found_and_bad_request() {
    let mut mock = MockPublicResolverTrait::new();
    mock.expect_resolve().returning(|urn, _| match urn.nid() {
        "dataset" => Ok(None),
        _ => Err(ymir::errors::Errors::format(
            ymir::errors::BadFormat::Received,
            "unknown profile",
            None,
        )),
    });
    let remote = RemotePublicResolver::new(serve(vec![Arc::new(mock)]).await);

    let missing = remote
        .resolve(&urn("urn:dataset:1"), &PublicQuery::default())
        .await
        .unwrap();
    let error = remote
        .resolve(&urn("urn:agreement:1"), &PublicQuery::default())
        .await
        .unwrap_err();

    assert!(missing.is_none());
    assert_eq!(error.info().status_code.as_u16(), 400);
}
