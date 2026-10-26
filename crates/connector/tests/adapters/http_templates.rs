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

//! Template router: bodies travel as JSON-LD in the connector context; a mocked service.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use common::eunomia::EUNOMIA_CONNECTOR_CONTEXT_URL;
use connector::http::connector_template::ConnectorTemplateRouter;
use connector::MockConnectorTemplateServiceTrait;
use serde_json::{json, Value};
use tower::ServiceExt;
use ymir::services::token_validator::{FixedUserValidator, OauthTokenValidatorTrait};
use ymir::types::oauth::UserInfo;

fn template(context: &str) -> Value {
    json!({
        "@context": context,
        "@type": "ConnectorTemplate",
        "name": "weather-pull",
        "version": "1.0",
        "authentication": {"type": "NO_AUTH"},
        "interaction": {
            "mode": "PULL",
            "dataAccess": {"protocol": "HTTP", "urlTemplate": "https://example.com", "method": ["GET"]}
        },
        "parameters": []
    })
}

fn app(service: MockConnectorTemplateServiceTrait) -> Router {
    let validator: Arc<dyn OauthTokenValidatorTrait> =
        Arc::new(FixedUserValidator::new(UserInfo::system()));
    ConnectorTemplateRouter::new(Arc::new(service))
        .router()
        .route_layer(axum::middleware::from_fn_with_state(
            validator,
            ymir::http::OauthHttpMiddleware::run,
        ))
}

async fn post(app: Router, body: Value) -> (StatusCode, Value) {
    let request = Request::post("/")
        .header("content-type", "application/json")
        .header("authorization", "Bearer any")
        .body(Body::from(body.to_string()))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

/// A template in the connector context is stored and answered with its context and type.
#[tokio::test]
async fn template_is_answered_as_json_ld() {
    let mut service = MockConnectorTemplateServiceTrait::new();
    service
        .expect_create_template()
        .returning(|_, template| Ok(template.clone()));
    let (status, answer) = post(app(service), template(EUNOMIA_CONNECTOR_CONTEXT_URL)).await;
    assert_eq!(status, StatusCode::OK, "{answer}");
    assert_eq!(answer["@context"], EUNOMIA_CONNECTOR_CONTEXT_URL);
    assert_eq!(answer["@type"], "ConnectorTemplate");
    assert_eq!(answer["name"], "weather-pull");
}

/// A template in another context is refused before the service is called.
#[tokio::test]
async fn template_in_another_context_is_refused() {
    let (status, _) = post(
        app(MockConnectorTemplateServiceTrait::new()),
        template("https://w3id.org/eunomia/v1/context.jsonld"),
    )
    .await;
    assert!(status.is_client_error(), "{status}");
}
