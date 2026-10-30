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

//! Template and offer routers over mocked template and composition services: usages, previews,
//! compositions and the 409 of a referenced version.

use axum::body::Body;
use axum::extract::Request;
use axum::http::{Method, StatusCode};
use catalog_agent::entities::errors::CatalogErrors;
use catalog_agent::entities::policy_templates::ControlPoint;
use catalog_agent::entities::resource_kind::ResourceKind;
use catalog_agent::http::odrl_policies::OdrlOfferEntityRouter;
use catalog_agent::http::policy_templates::PolicyTemplateEntityRouter;
use catalog_agent::services::policy_composition::MockPolicyCompositionServiceTrait;
use catalog_agent::services::policy_templates::views::{PolicyPreviewView, TemplateUsagesView};
use catalog_agent::services::policy_templates::MockPolicyTemplateServiceTrait;
use common::test_utils::scopes::TestUsers;
use serde_json::{json, Value};
use tower::ServiceExt;

use crate::support::builders::{id, offer};
use crate::support::context::TestContext;

async fn call(router: axum::Router, method: Method, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(body.map(|b| Body::from(b.to_string())).unwrap_or_else(Body::empty))
        .unwrap();
    request.extensions_mut().insert(TestUsers::alone("ana"));
    let response = router.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

/// Usages answer the versions and offers built from a version.
#[tokio::test]
async fn usages_route_answers_templates_and_offers() {
    let mut templates = MockPolicyTemplateServiceTrait::new();
    templates
        .expect_usages()
        .withf(|_, id, version| id == "expires-after" && version == "1.0")
        .returning(|_, _, _| Ok(TemplateUsagesView { templates: vec!["research-30-days@1.0".into()], offers: vec![] }));
    let ctx = TestContext::new().templates(templates).build();

    let (status, answer) =
        call(PolicyTemplateEntityRouter::new(ctx).router(), Method::GET, "/expires-after/1.0/usages", None).await;
    assert_eq!(status, StatusCode::OK, "{answer}");
    assert_eq!(answer, json!({"templates": ["research-30-days@1.0"], "offers": []}));
}

/// A preview answers the plain ODRL and its control points.
#[tokio::test]
async fn preview_route_answers_odrl_and_control_points() {
    let mut templates = MockPolicyTemplateServiceTrait::new();
    templates.expect_preview().returning(|_, _| {
        Ok(PolicyPreviewView { odrl: json!({"permission": [{"action": "use"}]}), control_points: vec![ControlPoint::Transfer] })
    });
    let ctx = TestContext::new().templates(templates).build();

    let body = json!({"id": "use", "version": "1.0"});
    let (status, answer) = call(PolicyTemplateEntityRouter::new(ctx).router(), Method::POST, "/preview", Some(body)).await;
    assert_eq!(status, StatusCode::OK, "{answer}");
    assert_eq!(answer["controlPoints"], json!(["transfer"]));
}

/// Deleting a referenced version answers 409.
#[tokio::test]
async fn deleting_a_referenced_version_is_a_conflict() {
    let mut templates = MockPolicyTemplateServiceTrait::new();
    templates
        .expect_delete_policy_template_by_version_and_id()
        .returning(|_, _, _| Err(CatalogErrors::conflict("referenced")));
    let ctx = TestContext::new().templates(templates).build();

    let (status, _) = call(PolicyTemplateEntityRouter::new(ctx).router(), Method::DELETE, "/use/1.0", None).await;
    assert_eq!(status, StatusCode::CONFLICT);
}

/// A composition is created and answered as an offer with its parts.
#[tokio::test]
async fn compose_route_creates_the_offer() {
    let mut composition = MockPolicyCompositionServiceTrait::new();
    composition
        .expect_compose()
        .withf(|_, request| request.parts.len() == 2)
        .returning(|_, request| {
            let mut created = offer(1, &request.entity_id);
            created.composition = Some(json!({"parts": request.parts, "controlPoints": ["validity"]}));
            Ok(created)
        });
    let ctx = TestContext::new().composition(composition).build();

    let body = json!({
        "entityId": id(ResourceKind::Dataset, 3),
        "entityType": "Dataset",
        "parts": [{"@ref": "use@1.0"}, {"@ref": "expires-after@1.0", "into": "permission"}]
    });
    let (status, answer) = call(OdrlOfferEntityRouter::new(ctx).router(), Method::POST, "/compose", Some(body)).await;
    assert_eq!(status, StatusCode::CREATED, "{answer}");
    assert_eq!(answer["composition"][1]["into"], "permission");
    assert_eq!(answer["controlPoints"], json!(["validity"]));
}
