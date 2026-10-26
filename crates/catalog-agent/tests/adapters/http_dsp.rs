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

//! The DSP catalog routes behind PeerAuth, over a mocked provider service, grants and mates.

use std::sync::Arc;

use axum::body::Body;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::Router;
use catalog_agent::protocols::dsp::http::dsp::DspRouter;
use catalog_agent::protocols::dsp::services::dsp_catalog::MockDspCatalogServiceTrait;
use catalog_agent::protocols::dsp::services::rpc_catalog::MockRpcCatalogServiceTrait;
use catalog_agent::protocols::dsp::setup::context::DspContext;
use common::facades::grants_facade::{MockGrantsFacadeTrait, VerifiedPeer};
use common::facades::mates_facade::MockMatesFacadeTrait;
use serde_json::{json, Value};
use tower::ServiceExt;
use ymir::errors::Errors;
use ymir::types::oauth::RolePath;
use ymir::types::participants::Visibility;

use crate::support::context::TestContext;

const DATASET: &str = "urn:dataset:9f8c7a2e-0000-0000-0000-000000000001";

/// A router whose peer is always verified, whose catalog is fixed and whose datasets are unknown.
fn router() -> Router {
    let mut grants = MockGrantsFacadeTrait::new();
    grants.expect_verify_token().returning(|_| {
        Ok(VerifiedPeer {
            participant_id: "did:example:consumer".into(),
            role: RolePath::root(),
            visibility: Visibility::Public,
        })
    });
    let mut mates = MockMatesFacadeTrait::new();
    mates.expect_get_mate_by_id().returning(|_, id| Err(Errors::missing_resource(id, "not a mate", None)));
    let app = TestContext::new().grants(grants).mates(mates).build();
    let mut catalog = MockDspCatalogServiceTrait::new();
    catalog.expect_catalog().returning(|_, _| Ok(json!({"@type": "Catalog"})));
    catalog
        .expect_dataset()
        .returning(|_, _| Err(Errors::missing_resource(DATASET, "dataset not found", None)));
    let ctx = Arc::new(DspContext {
        app,
        catalog: Arc::new(catalog),
        rpc: Arc::new(MockRpcCatalogServiceTrait::new()),
    });
    DspRouter::new(ctx).router()
}

async fn send(method: &str, path: &str, body: Option<Value>) -> (StatusCode, Value) {
    let request = Request::builder()
        .method(method)
        .uri(path)
        .header("authorization", "Bearer tok")
        .header("content-type", "application/json")
        .body(body.map(|b| Body::from(b.to_string())).unwrap_or_else(Body::empty))
        .unwrap();
    let response = router().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 64 * 1024).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

/// A catalog request read through RDF is answered with the catalog.
#[tokio::test]
async fn catalog_request_is_answered() {
    let (status, body) = send(
        "POST",
        "/request",
        Some(json!({"@context": ["https://w3id.org/dspace/2025/1/context.jsonld"], "@type": "CatalogRequestMessage"})),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["@type"], "Catalog");
}

/// A body that declares no catalog request is a 400 `CatalogError` (DSP 6.2.3).
#[tokio::test]
async fn body_without_message_type_is_a_catalog_error() {
    let (status, body) = send("POST", "/request", Some(json!({"filter": []}))).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["@type"], "CatalogError");
    assert_eq!(body["code"], "400");
}

/// An unknown dataset is a 404 with a `CatalogError` body.
#[tokio::test]
async fn unknown_dataset_is_a_404_catalog_error() {
    let (status, body) = send("GET", &format!("/datasets/{DATASET}"), None).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["@type"], "CatalogError");
    assert!(body["reason"].as_array().is_some_and(|r| !r.is_empty()));
}

/// A dataset id that is not a URN is a 400 `CatalogError`.
#[tokio::test]
async fn malformed_dataset_id_is_a_400_catalog_error() {
    let (status, body) = send("GET", "/datasets/not-a-urn", None).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["@type"], "CatalogError");
}
