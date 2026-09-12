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
use bff::auth::BffAuthMiddleware;
use bff::create_gateway_http_router;
use bff::events::feed_router::ListEventsQuery;
use bff::events::sse_handler::SseQuery;
use bff::events::{BffEventFeedRouter, SseStreamHandler};
use bff::proxy::HttpProxyDispatcher;
use bff::setup::context::AppContext;
use bff::setup::BffModule;
use bff::GatewayHttpRouter;
use common::auth::claims::{Claims, RbacRole};
use common::auth::middleware::OauthTokenValidator;
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
    let auth = BffAuthMiddleware::new(Some(validator), true);

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
async fn test_bff_events_feed_and_sse() {
    let events_ctx = events::setup::AppContext::in_memory(None);
    let bus = events_ctx.event_bus.clone();

    // Publish event
    let topic = Topic::new("transfer.process.started").unwrap();
    let payload = json!({ "process_id": "proc-456", "mode": "push" });
    let envelope = EventEnvelope::new(topic.clone(), "transfer-agent", 1, None, payload.clone());
    bus.publish(envelope.clone()).await.unwrap();

    // Query list
    let list_resp = BffEventFeedRouter::handle_list(
        bus.clone(),
        ListEventsQuery {
            topic: Some("transfer.process.started".to_string()),
            limit: None,
            offset: None,
        },
    )
    .await;
    assert_eq!(list_resp.status(), StatusCode::OK);

    // Query single
    let get_resp = BffEventFeedRouter::handle_get(bus.clone(), envelope.id.to_string()).await;
    assert_eq!(get_resp.status(), StatusCode::OK);

    // Test SSE Stream
    let sse = SseStreamHandler::stream_events(
        bus.clone(),
        SseQuery {
            topic: Some("transfer.**".to_string()),
        },
    );
    let mut stream = sse.into_response().into_body().into_data_stream();

    // Trigger next event
    let topic2 = Topic::new("transfer.process.completed").unwrap();
    let envelope2 = EventEnvelope::new(topic2, "transfer-agent", 1, None, json!({ "done": true }));
    bus.publish(envelope2).await.unwrap();

    // Next item from stream
    if let Some(Ok(bytes)) = stream.next().await {
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("transfer.process.completed"));
    }
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

#[tokio::test]
async fn test_bff_create_dataset_offering() {
    let mock_upstream = Router::new()
        .route(
            "/api/v1/catalog-agent/catalogs/main",
            get(|| async { Json(json!({ "id": "urn:uuid:mock-catalog-001" })) }),
        )
        .route(
            "/api/v1/catalog-agent/data-services/main",
            get(|| async { Json(json!({ "id": "urn:uuid:mock-dataservice-001" })) }),
        )
        .route(
            "/api/v1/catalog-agent/datasets",
            post(|Json(payload): Json<serde_json::Value>| async move {
                Json(json!({
                    "id": "urn:uuid:mock-dataset-001",
                    "dctTitle": payload["dctTitle"],
                    "catalogId": payload["catalogId"]
                }))
            }),
        )
        .route(
            "/api/v1/catalog-agent/distributions",
            post(|Json(payload): Json<serde_json::Value>| async move {
                Json(json!({
                    "id": "urn:uuid:mock-distribution-001",
                    "dctTitle": payload["dctTitle"],
                    "datasetId": payload["datasetId"]
                }))
            }),
        )
        .route(
            "/api/v1/connector/instances",
            post(|Json(payload): Json<serde_json::Value>| async move {
                Json(json!({
                    "id": payload["id"],
                    "name": payload["name"],
                    "distributionId": payload["distributionId"]
                }))
            }),
        )
        .route(
            "/api/v1/catalog-agent/odrl-policies",
            post(|Json(payload): Json<serde_json::Value>| async move {
                Json(json!({
                    "id": "urn:uuid:mock-policy-001",
                    "entityId": payload["entityId"]
                }))
            }),
        );

    let upstream_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let upstream_port = upstream_listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(upstream_listener, mock_upstream).await.unwrap();
    });

    let events_ctx = events::setup::AppContext::in_memory(None);
    let event_bus = events_ctx.event_bus.clone();

    let config = dummy_gateway_config(upstream_port);
    let app_ctx = Arc::new(AppContext::new(config, Some(event_bus.clone()), None));
    let router = GatewayHttpRouter::with_context(app_ctx).router();

    let bff_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let bff_port = bff_listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(bff_listener, router).await.unwrap();
    });

    let client = reqwest::Client::new();
    let payload = json!({
        "dataset": {
            "title": "Industrial Telemetry 2026",
            "description": "High-frequency machine telemetry data",
            "conformsTo": "https://w3id.org/dspace/v0.8/dcat",
            "creator": "urn:uuid:creator-1"
        },
        "distribution": {
            "title": "REST Streaming Feed",
            "description": "JSON format over HTTPS",
            "formats": "application/json"
        },
        "connector": {
            "name": "telemetry-connector",
            "endpoint": "https://data.example.org/telemetry",
            "protocol": "HTTP",
            "method": "GET",
            "auth": { "type": "NO_AUTH" }
        },
        "policy": {
            "description": "Permissive Access Policy",
            "action": "http://www.w3.org/ns/odrl/2/use",
            "profile": "http://www.w3.org/ns/odrl/2/"
        }
    });

    let resp = client
        .post(format!("http://127.0.0.1:{bff_port}/api/catalog-offerings"))
        .json(&payload)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["dataset"]["id"], "urn:uuid:mock-dataset-001");
    assert_eq!(body["distribution"]["id"], "urn:uuid:mock-distribution-001");
    assert_eq!(body["policy"]["id"], "urn:uuid:mock-policy-001");
    assert!(body["connector"].is_object());

    // Verify event was published
    let events = event_bus
        .event_repo()
        .list_events(Some("catalog.dataset.created"), 10, 0)
        .await
        .unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].topic.as_str(), "catalog.dataset.created");
}
