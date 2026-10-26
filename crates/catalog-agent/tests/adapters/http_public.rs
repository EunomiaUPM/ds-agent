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

//! PublicProfileRouter over a mocked closure service: JSON-LD, profile link, entity tags.

use std::sync::Arc;

use axum::body::Body;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::Response;
use catalog_agent::entities::resource_kind::ResourceKind;
use catalog_agent::profiles::dcat_ap::DcatAp;
use catalog_agent::profiles::shared::dcat::DcatAdapter;
use catalog_agent::profiles::shared::public::PublicProfileRouter;
use catalog_agent::services::closure::MockClosureServiceTrait;
use tower::ServiceExt;

use crate::support::builders::{id, provider_set, urn};
use crate::support::context::TestContext;

/// The same provider set on every call, so equal requests get equal bodies.
fn router() -> axum::Router {
    let fixed = provider_set();
    let main = fixed.clone();
    let mut closures = MockClosureServiceTrait::new();
    closures.expect_main_catalog().returning(move |_| Ok(main.clone()));
    closures.expect_resource().returning(move |_, id| {
        let set = fixed.1.clone();
        Ok(set.get(id.as_ref()).cloned().map(|root| (root, set)))
    });
    let ctx = TestContext::new().closures(closures).build();
    PublicProfileRouter::new(ctx, Arc::new(DcatAdapter::<DcatAp>::default())).router()
}

async fn get_from(router: axum::Router, path: &str, if_none_match: Option<&str>) -> Response {
    let mut request = Request::builder().uri(path);
    if let Some(tag) = if_none_match {
        request = request.header("if-none-match", tag);
    }
    router.oneshot(request.body(Body::empty()).unwrap()).await.unwrap()
}

async fn get(path: &str, if_none_match: Option<&str>) -> Response {
    get_from(router(), path, if_none_match).await
}

/// The catalog comes as JSON-LD with the profile it conforms to and a tag over its body.
#[tokio::test]
async fn catalog_is_json_ld_with_profile_and_tag() {
    let router = router();
    let response = get_from(router.clone(), "/catalog", None).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "application/ld+json");
    assert!(response.headers()["link"].to_str().unwrap().contains("rel=\"profile\""));
    let etag = response.headers()["etag"].to_str().unwrap().to_string();
    assert_eq!(get_from(router, "/catalog", Some(&etag)).await.status(), StatusCode::NOT_MODIFIED);
}

/// A resource is its closure in the profile; one that is not there is a 404.
#[tokio::test]
async fn resources_are_projected_or_missing() {
    let found = get(&format!("/resources/{}", id(ResourceKind::Dataset, 3)), None).await;
    let missing = get(&format!("/resources/{}", urn(ResourceKind::Dataset, 99)), None).await;

    assert_eq!(found.status(), StatusCode::OK);
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
}

/// The profile's context is served as a document of its own.
#[tokio::test]
async fn context_is_served() {
    let response = get("/context.jsonld", None).await;
    let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let context: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(context["@context"]["@protected"], true);
}
