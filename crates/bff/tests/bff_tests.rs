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

use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::extract::Request;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use bff::create_gateway_http_router;
use bff::events::feed_router::ListEventsQuery;
use bff::events::sse_handler::SseQuery;
use bff::events::{BffEventFeedRouter, SseStreamHandler};
use bff::proxy::HttpProxyDispatcher;
use bff::setup::context::AppContext;
use bff::setup::BffModule;
use bff::GatewayHttpRouter;
use common::auth::claims::{Claims, RbacRole};
use common::auth::http::AuthHttpMiddleware;
use common::auth::OauthTokenValidator;
use common::config::services::GatewayConfig;
use common::module_loader::service_module::ServiceModuleTrait;
use events::bus::envelope::{EventEnvelope, Topic};
use events::bus::{EventBus, EventBusTrait};
use futures_util::StreamExt;
use serde_json::json;
use tokio::net::TcpListener;
use ymir::errors::{Errors, Outcome};

struct MockTokenValidator;

#[async_trait::async_trait]
impl OauthTokenValidator for MockTokenValidator {
    async fn validate_token(&self, token: &str) -> Outcome<Claims> {
        if token == "valid-jwt-token" || token.starts_with("pat_") {
            Ok(Claims {
                sub: "user-admin-123".to_string(),
                role: RbacRole::Admin,
                iat: 1000,
                exp: 9999999999,
            })
        } else {
            Err(Errors::unauthorized("invalid token", None))
        }
    }
}

fn dummy_gateway_config(upstream_port: u16) -> GatewayConfig {
    let p_str = upstream_port.to_string();
    let raw = json!({
        "common": {
            "hosts": {
                "http": { "protocol": "http", "url": "127.0.0.1", "port": "8080", "internal_port": "8080" },
                "grpc": null,
                "graphql": null
            },
            "db": { "db_type": "Postgres", "url": "localhost", "port": "5432" },
            "api": { "version": "v1", "openapi_path": "/openapi.json" },
            "connection": { "is_local": true, "is_prod": false, "is_vault_real": false, "has_tls_proxy": false }
        },
        "is_production": false,
        "is_catalog_datahub": false,
        "catalog": {
            "hosts": {
                "http": { "protocol": "http", "url": "127.0.0.1", "port": p_str.clone(), "internal_port": p_str.clone() },
                "grpc": null,
                "graphql": null
            },
            "api_version": "v1"
        },
        "contracts": {
            "hosts": {
                "http": { "protocol": "http", "url": "127.0.0.1", "port": p_str.clone(), "internal_port": p_str.clone() },
                "grpc": null,
                "graphql": null
            },
            "api_version": "v1"
        },
        "transfer": {
            "hosts": {
                "http": { "protocol": "http", "url": "127.0.0.1", "port": p_str.clone(), "internal_port": p_str.clone() },
                "grpc": null,
                "graphql": null
            },
            "api_version": "v1"
        },
        "ssi_auth": {
            "hosts": {
                "http": { "protocol": "http", "url": "127.0.0.1", "port": p_str.clone(), "internal_port": p_str },
                "grpc": null,
                "graphql": null
            },
            "api_version": "v1"
        }
    });
    serde_json::from_value(raw).expect("valid config")
}

#[tokio::test]
async fn test_bff_auth_middleware_bearer_and_query() {
    let validator: Arc<dyn OauthTokenValidator> = Arc::new(MockTokenValidator);
    let auth = AuthHttpMiddleware::new(Some(validator), true);

    let app = Router::new()
        .route(
            "/protected",
            get(|req: Request| async move {
                let user_sub = req
                    .extensions()
                    .get::<Claims>()
                    .map(|c| c.sub.clone())
                    .unwrap_or_default();
                (StatusCode::OK, user_sub)
            }),
        )
        .layer(axum::middleware::from_fn(move |req, next| {
            let auth = auth.clone();
            async move { auth.handle(req, next).await }
        }));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = reqwest::Client::new();
    let base = format!("http://127.0.0.1:{port}");

    // 1. Missing token -> 401 Unauthorized
    let unauth_resp = client
        .get(format!("{base}/protected"))
        .send()
        .await
        .unwrap();
    assert_eq!(unauth_resp.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        unauth_resp.headers().get("x-content-type-options").unwrap(),
        "nosniff"
    );

    // 2. Valid Bearer token in header -> 200 OK
    let bearer_resp = client
        .get(format!("{base}/protected"))
        .header("Authorization", "Bearer valid-jwt-token")
        .send()
        .await
        .unwrap();
    assert_eq!(bearer_resp.status(), StatusCode::OK);
    assert_eq!(bearer_resp.text().await.unwrap(), "user-admin-123");

    // 3. Valid PAT in query param (used for browser WebSockets) -> 200 OK
    let query_resp = client
        .get(format!("{base}/protected?token=pat_testsecret123"))
        .send()
        .await
        .unwrap();
    assert_eq!(query_resp.status(), StatusCode::OK);
    assert_eq!(query_resp.text().await.unwrap(), "user-admin-123");
}

#[tokio::test]
async fn test_bff_reverse_proxy_dispatch() {
    let upstream = Router::new().route(
        "/api/v1/catalog-agent/catalogs/items",
        get(|headers: HeaderMap| async move {
            let correlation = headers
                .get("x-correlation-id")
                .and_then(|v| v.to_str().ok())
                .unwrap_or_default()
                .to_string();
            let request_id = headers
                .get("x-request-id")
                .and_then(|v| v.to_str().ok())
                .unwrap_or_default()
                .to_string();

            Json(json!({
                "correlation": correlation,
                "request_id": request_id,
                "status": "proxied_ok"
            }))
        }),
    );

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });

    let config = dummy_gateway_config(port);
    let proxy = HttpProxyDispatcher::new(config);

    let req = Request::builder()
        .uri("http://gateway/api/catalogs/items?page=1")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let resp = proxy
        .proxy_request("catalogs".to_string(), Some("items".to_string()), req)
        .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(resp.headers().contains_key("x-correlation-id"));
}

#[tokio::test]
async fn test_bff_module_service_trait_and_backward_compatibility() {
    let config = dummy_gateway_config(8080);
    let app_ctx = Arc::new(AppContext::new(config.clone(), None, None));
    let module = BffModule::new(app_ctx);

    assert_eq!(module.name(), "gateway");
    let http = module.http().expect("http routes present");
    assert_eq!(http.0, "");

    // Test legacy router helper
    let legacy_router = create_gateway_http_router(&config).await;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, legacy_router).await.unwrap();
    });

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{port}/admin/api/fe-config"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
