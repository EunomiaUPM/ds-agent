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

//! Tests for the monolith static HTML landing page at `GET /`.

use axum::http::header::CONTENT_TYPE;
use axum::http::StatusCode;
use axum::Router;
use monolith::http::LandingRouter;
use tokio::net::TcpListener;
use ymir::services::client::ClientTrait;
use ymir::utils::http_client;

struct TestServer;

impl TestServer {
    async fn spawn(router: Router) -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        port
    }
}

/// Renders HTML at `GET /` linking to `/admin/` and default v1 OpenAPI docs.
#[tokio::test]
async fn landing_renders_links_to_admin_and_openapi() {
    let port = TestServer::spawn(LandingRouter::for_api_base("/api/v1").router()).await;

    let resp = http_client()
        .get(&format!("http://127.0.0.1:{port}/"), None)
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let content_type = resp
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    assert!(content_type.contains("text/html"));

    let body = resp.text().await.unwrap();
    assert!(body.contains("Eunomia DS-Agent"));
    assert!(body.contains("href=\"/admin/\""));
    assert!(body.contains("href=\"/api/v1/docs/openapi\""));
    assert!(body.contains("href=\"/api/v1/docs/openapi.json\""));
}

/// Normalizes trailing slashes in custom API prefixes when building OpenAPI links.
#[tokio::test]
async fn landing_normalizes_custom_api_base() {
    let port = TestServer::spawn(LandingRouter::for_api_base("/api/v2/").router()).await;

    let resp = http_client()
        .get(&format!("http://127.0.0.1:{port}/"), None)
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.text().await.unwrap();
    assert!(body.contains("href=\"/api/v2/docs/openapi\""));
    assert!(body.contains("href=\"/api/v2/docs/openapi.json\""));
    assert!(!body.contains("//docs/openapi"));
}
