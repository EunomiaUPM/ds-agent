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

//! HttpProxyDispatcher against a local upstream standing in for the catalog agent.

use axum::body::Body;
use axum::extract::Request;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::get;
use axum::{Json, Router};
use bff::proxy::HttpProxyDispatcher;
use serde_json::json;

use crate::support::fixtures::{gateway_config, serve};

/// A gateway path is forwarded to the agent's API and the response carries a correlation id.
#[tokio::test]
async fn forwards_gateway_path_to_the_agent_api() {
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
    let port = serve(upstream).await;

    let proxy = HttpProxyDispatcher::new(gateway_config(port));

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
