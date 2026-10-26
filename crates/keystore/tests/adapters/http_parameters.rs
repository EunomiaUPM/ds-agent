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

//! Parameter router: bodies travel as JSON-LD in the keystore context; a mocked repository.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use common::eunomia::EUNOMIA_KEYSTORE_CONTEXT_URL;
use keystore::data::repo::parameters::MockParameterRepoTrait;
use keystore::entities::entry::Entry;
use keystore::http::parameters::ParameterRouter;
use keystore::services::parameters::service::ParameterStoreImpl;
use serde_json::{json, Value};
use tower::ServiceExt;
use ymir::services::token_validator::{FixedUserValidator, OauthTokenValidatorTrait};
use ymir::types::oauth::UserInfo;

use crate::support::fixtures::metadata;

fn app(repo: MockParameterRepoTrait) -> Router {
    let validator: Arc<dyn OauthTokenValidatorTrait> =
        Arc::new(FixedUserValidator::new(UserInfo::system()));
    ParameterRouter::new(Arc::new(ParameterStoreImpl::new(Arc::new(repo))))
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

fn parameter(context: &str) -> Value {
    json!({
        "@context": context,
        "@type": "Parameter",
        "key": "/config/timeout",
        "value": {"seconds": 30},
        "description": "Request timeout"
    })
}

/// A parameter in the keystore context is created and answered with its context and type.
#[tokio::test]
async fn parameter_is_answered_as_json_ld() {
    let mut repo = MockParameterRepoTrait::new();
    repo.expect_create_parameter().returning(|tenant, cmd| {
        Ok(Entry {
            metadata: metadata(tenant, cmd.key.clone()),
            value: cmd.value.clone(),
        })
    });
    let (status, answer) = post(app(repo), parameter(EUNOMIA_KEYSTORE_CONTEXT_URL)).await;
    assert_eq!(status, StatusCode::CREATED, "{answer}");
    assert_eq!(answer["@context"], EUNOMIA_KEYSTORE_CONTEXT_URL);
    assert_eq!(answer["@type"], "Parameter");
    assert_eq!(answer["value"]["seconds"], 30);
}

/// A parameter of another type is refused before the repository is touched.
#[tokio::test]
async fn parameter_of_another_type_is_refused() {
    let mut body = parameter(EUNOMIA_KEYSTORE_CONTEXT_URL);
    body["@type"] = json!("Secret");
    let (status, _) = post(app(MockParameterRepoTrait::new()), body).await;
    assert!(status.is_client_error(), "{status}");
}
