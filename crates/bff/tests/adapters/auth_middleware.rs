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

//! The auth middleware as the bff mounts it, with a stub token validator.

use std::sync::Arc;

use axum::extract::Request;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::get;
use axum::Router;
use common::auth::claims::Claims;
use common::auth::http::AuthHttpMiddleware;
use common::auth::OauthTokenValidator;
use ymir::services::client::ClientTrait;
use ymir::utils::http_client;

use crate::support::fixtures::serve;
use crate::support::mocks::StubTokenValidator;

/// No token is a 401; a bearer header or a PAT in the query string, as browser WebSockets
/// send it, both pass.
#[tokio::test]
async fn accepts_bearer_header_or_query_token() {
    let validator: Arc<dyn OauthTokenValidator> = Arc::new(StubTokenValidator);
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
    let port = serve(app).await;

    let client = http_client();
    let base = format!("http://127.0.0.1:{port}");

    let unauth_resp = client
        .get(&format!("{base}/protected"), None)
        .await
        .unwrap();
    assert_eq!(unauth_resp.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        unauth_resp.headers().get("x-content-type-options").unwrap(),
        "nosniff"
    );

    let mut bearer = HeaderMap::new();
    bearer.insert("Authorization", "Bearer valid-jwt-token".parse().unwrap());
    let bearer_resp = client
        .get(&format!("{base}/protected"), Some(bearer))
        .await
        .unwrap();
    assert_eq!(bearer_resp.status(), StatusCode::OK);
    assert_eq!(bearer_resp.text().await.unwrap(), "user-admin-123");

    let query_resp = client
        .get(&format!("{base}/protected?token=pat_testsecret123"), None)
        .await
        .unwrap();
    assert_eq!(query_resp.status(), StatusCode::OK);
    assert_eq!(query_resp.text().await.unwrap(), "user-admin-123");
}
