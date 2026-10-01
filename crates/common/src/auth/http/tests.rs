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

//! AuthHttpMiddleware and the AuthClaims and AccessScope extractors on an axum router, with a
//! stub token validator.

use std::sync::Arc;

use axum::body::Body;
use axum::extract::Request;
use axum::http::header::AUTHORIZATION;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{middleware, Json, Router};
use tower::ServiceExt;
use ymir::errors::{Errors, Outcome};

use crate::auth::http::{AuthClaims, AuthHttpMiddleware};
use crate::auth::{AccessScope, Claims, OauthTokenValidator, RbacRole};

/// Accepts one admin and one owner token; anything else is unauthorized.
struct StubValidator;

#[async_trait::async_trait]
impl OauthTokenValidator for StubValidator {
    async fn validate_token(&self, token: &str) -> Outcome<Claims> {
        match token {
            "valid_admin_token" => Ok(Claims {
                sub: "admin-tenant".to_string(),
                role: RbacRole::Admin,
                iat: 1000,
                exp: 9999999999,
            }),
            "valid_user_token" => Ok(Claims {
                sub: "tenant-42".to_string(),
                role: RbacRole::Owner,
                iat: 1000,
                exp: 9999999999,
            }),
            _ => Err(Errors::unauthorized("invalid token", None)),
        }
    }
}

/// The bearer is trimmed; a missing, non-bearer or blank header is rejected.
#[tokio::test]
async fn bearer_is_trimmed_and_non_bearer_rejected() {
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_static("Bearer secret_jwt_token"),
    );
    assert_eq!(
        AuthHttpMiddleware::bearer(&headers).unwrap(),
        "secret_jwt_token"
    );

    let mut headers_ws = HeaderMap::new();
    headers_ws.insert(
        AUTHORIZATION,
        HeaderValue::from_static("Bearer   spaced_jwt   "),
    );
    assert_eq!(
        AuthHttpMiddleware::bearer(&headers_ws).unwrap(),
        "spaced_jwt"
    );

    let empty_headers = HeaderMap::new();
    assert!(AuthHttpMiddleware::bearer(&empty_headers).is_err());

    let mut basic_headers = HeaderMap::new();
    basic_headers.insert(
        AUTHORIZATION,
        HeaderValue::from_static("Basic dXNlcjpwYXNz"),
    );
    assert!(AuthHttpMiddleware::bearer(&basic_headers).is_err());

    let mut blank_bearer = HeaderMap::new();
    blank_bearer.insert(AUTHORIZATION, HeaderValue::from_static("Bearer   "));
    assert!(AuthHttpMiddleware::bearer(&blank_bearer).is_err());
}

/// A valid token reaches the handler as AuthClaims; a bad or missing one is a 401.
#[tokio::test]
async fn middleware_puts_claims_in_reach_of_handlers() {
    let validator: Arc<dyn OauthTokenValidator> = Arc::new(StubValidator);

    async fn protected_handler(AuthClaims(claims): AuthClaims) -> impl IntoResponse {
        Json(serde_json::json!({
            "sub": claims.sub,
            "role": claims.role.as_str(),
            "is_admin": claims.is_admin()
        }))
    }

    let app = Router::new()
        .route("/protected", get(protected_handler))
        .route_layer(middleware::from_fn_with_state(
            validator,
            AuthHttpMiddleware::run,
        ));

    let req = Request::builder()
        .uri("/protected")
        .header(AUTHORIZATION, "Bearer valid_admin_token")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 64)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["sub"], "admin-tenant");
    assert_eq!(body["role"], "admin");
    assert_eq!(body["is_admin"], true);

    let req_bad = Request::builder()
        .uri("/protected")
        .header(AUTHORIZATION, "Bearer invalid_token")
        .body(Body::empty())
        .unwrap();
    let resp_bad = app.clone().oneshot(req_bad).await.unwrap();
    assert_eq!(resp_bad.status(), StatusCode::UNAUTHORIZED);

    let req_missing = Request::builder()
        .uri("/protected")
        .body(Body::empty())
        .unwrap();
    let resp_missing = app.oneshot(req_missing).await.unwrap();
    assert_eq!(resp_missing.status(), StatusCode::UNAUTHORIZED);
}

/// The AccessScope extractor gives 403 for a foreign tenant unless Admin, and 400 for a
/// malformed one.
#[tokio::test]
async fn scope_extractor_enforces_the_tenant_header() {
    let validator: Arc<dyn OauthTokenValidator> = Arc::new(StubValidator);

    async fn scope_handler(scope: AccessScope) -> impl IntoResponse {
        Json(serde_json::json!({
            "tenant": scope.acting_tenant(),
            "is_admin": scope.is_admin()
        }))
    }

    let app = Router::new()
        .route("/tenant-data", get(scope_handler))
        .route_layer(middleware::from_fn_with_state(
            validator,
            AuthHttpMiddleware::run,
        ));

    let req = Request::builder()
        .uri("/tenant-data")
        .header(AUTHORIZATION, "Bearer valid_user_token")
        .header("x-tenant-id", "tenant-42")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let req_mismatch = Request::builder()
        .uri("/tenant-data")
        .header(AUTHORIZATION, "Bearer valid_user_token")
        .header("x-tenant-id", "other-tenant")
        .body(Body::empty())
        .unwrap();
    let resp_mismatch = app.clone().oneshot(req_mismatch).await.unwrap();
    assert_eq!(resp_mismatch.status(), StatusCode::FORBIDDEN);

    let req_admin = Request::builder()
        .uri("/tenant-data")
        .header(AUTHORIZATION, "Bearer valid_admin_token")
        .header("x-tenant-id", "other-tenant")
        .body(Body::empty())
        .unwrap();
    let resp_admin = app.clone().oneshot(req_admin).await.unwrap();
    assert_eq!(resp_admin.status(), StatusCode::OK);

    let req_bad_tenant = Request::builder()
        .uri("/tenant-data")
        .header(AUTHORIZATION, "Bearer valid_admin_token")
        .header("x-tenant-id", "bad!tenant@id")
        .body(Body::empty())
        .unwrap();
    let resp_bad_tenant = app.oneshot(req_bad_tenant).await.unwrap();
    assert_eq!(resp_bad_tenant.status(), StatusCode::BAD_REQUEST);
}

/// Tokens are also read from `token` and `access_token` query params, and permissive mode
/// lets anonymous requests through.
#[tokio::test]
async fn query_tokens_and_permissive_mode_are_accepted() {
    let validator: Arc<dyn OauthTokenValidator> = Arc::new(StubValidator);

    let req_query = Request::builder()
        .uri("/data?token=valid_user_token")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        AuthHttpMiddleware::extract_token(&req_query),
        Some("valid_user_token".to_string())
    );

    let req_access_token = Request::builder()
        .uri("/data?access_token=my_custom_token")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        AuthHttpMiddleware::extract_token(&req_access_token),
        Some("my_custom_token".to_string())
    );

    let permissive = AuthHttpMiddleware::permissive(Some(validator.clone()));
    let app_permissive = Router::new()
        .route(
            "/check",
            get(|req: Request| async move {
                let has_claims = req.extensions().get::<Claims>().is_some();
                Json(serde_json::json!({ "authenticated": has_claims }))
            }),
        )
        .layer(middleware::from_fn(move |req, next| {
            let mw = permissive.clone();
            async move { mw.handle(req, next).await }
        }));

    let req_anon = Request::builder()
        .uri("/check")
        .body(Body::empty())
        .unwrap();
    let resp_anon = app_permissive.clone().oneshot(req_anon).await.unwrap();
    assert_eq!(resp_anon.status(), StatusCode::OK);
    assert_eq!(
        resp_anon.headers().get("x-content-type-options").unwrap(),
        "nosniff"
    );

    let req_auth = Request::builder()
        .uri("/check?token=valid_admin_token")
        .body(Body::empty())
        .unwrap();
    let resp_auth = app_permissive.oneshot(req_auth).await.unwrap();
    assert_eq!(resp_auth.status(), StatusCode::OK);
}
