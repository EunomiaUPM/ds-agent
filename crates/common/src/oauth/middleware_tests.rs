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

//! OauthHttpMiddleware and the UserInfo extractor on an axum router, with a stub token
//! validator.

use std::sync::Arc;

use axum::body::Body;
use axum::extract::Request;
use axum::http::header::AUTHORIZATION;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{middleware, Json, Router};
use serde_json::Map;
use tower::ServiceExt;
use ymir::errors::{Errors, Outcome};

use ymir::http::OauthHttpMiddleware;

use crate::oauth::{RolePath, TokenValidatorTrait, UserInfo};

/// Accepts one root and one regular user token; anything else is unauthorized.
struct StubValidator;

#[async_trait::async_trait]
impl TokenValidatorTrait for StubValidator {
    async fn validate_token<'a>(&self, token: Option<&'a str>) -> Outcome<UserInfo> {
        match token.unwrap_or_default() {
            "valid_root_token" => Ok(UserInfo::new("root", None, RolePath::root(), Map::new())),
            "valid_user_token" => Ok(UserInfo::new(
                "ana",
                Some("ana@upm.es".to_string()),
                "/admin/upm/dit".parse()?,
                Map::new(),
            )),
            _ => Err(Errors::unauthorized("invalid token", None)),
        }
    }
}

fn app() -> Router {
    let validator: Arc<dyn TokenValidatorTrait> = Arc::new(StubValidator);

    async fn scope_handler(user: UserInfo) -> impl IntoResponse {
        Json(serde_json::json!({
            "user_id": user.user_id(),
            "role": user.role().as_str(),
            "is_root": user.is_root()
        }))
    }

    Router::new()
        .route("/protected", get(scope_handler))
        .route_layer(middleware::from_fn_with_state(
            validator,
            OauthHttpMiddleware::run,
        ))
}

async fn json(resp: axum::response::Response) -> serde_json::Value {
    let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 64)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
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
        OauthHttpMiddleware::bearer(&headers).unwrap(),
        "secret_jwt_token"
    );

    let mut headers_ws = HeaderMap::new();
    headers_ws.insert(
        AUTHORIZATION,
        HeaderValue::from_static("Bearer   spaced_jwt   "),
    );
    assert_eq!(
        OauthHttpMiddleware::bearer(&headers_ws).unwrap(),
        "spaced_jwt"
    );

    let empty_headers = HeaderMap::new();
    assert!(OauthHttpMiddleware::bearer(&empty_headers).is_err());

    let mut basic_headers = HeaderMap::new();
    basic_headers.insert(
        AUTHORIZATION,
        HeaderValue::from_static("Basic dXNlcjpwYXNz"),
    );
    assert!(OauthHttpMiddleware::bearer(&basic_headers).is_err());

    let mut blank_bearer = HeaderMap::new();
    blank_bearer.insert(AUTHORIZATION, HeaderValue::from_static("Bearer   "));
    assert!(OauthHttpMiddleware::bearer(&blank_bearer).is_err());
}

/// A valid token reaches the handler as the caller's scope; a bad or missing one is a 401.
#[tokio::test]
async fn middleware_puts_the_user_in_reach_of_handlers() {
    let req = Request::builder()
        .uri("/protected")
        .header(AUTHORIZATION, "Bearer valid_user_token")
        .body(Body::empty())
        .unwrap();
    let resp = app().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = json(resp).await;
    assert_eq!(body["user_id"], "ana");
    assert_eq!(body["role"], "/admin/upm/dit");
    assert_eq!(body["is_root"], false);

    let req_root = Request::builder()
        .uri("/protected")
        .header(AUTHORIZATION, "Bearer valid_root_token")
        .body(Body::empty())
        .unwrap();
    let body_root = json(app().oneshot(req_root).await.unwrap()).await;
    assert_eq!(body_root["is_root"], true);

    let req_bad = Request::builder()
        .uri("/protected")
        .header(AUTHORIZATION, "Bearer invalid_token")
        .body(Body::empty())
        .unwrap();
    let resp_bad = app().oneshot(req_bad).await.unwrap();
    assert_eq!(resp_bad.status(), StatusCode::UNAUTHORIZED);

    let req_missing = Request::builder()
        .uri("/protected")
        .body(Body::empty())
        .unwrap();
    let resp_missing = app().oneshot(req_missing).await.unwrap();
    assert_eq!(resp_missing.status(), StatusCode::UNAUTHORIZED);
}

/// Without the middleware there is no user, so the extractor rejects with a 401.
#[tokio::test]
async fn user_without_middleware_is_unauthorized() {
    async fn scope_handler(_user: UserInfo) -> impl IntoResponse {
        StatusCode::OK
    }
    let unprotected = Router::new().route("/open", get(scope_handler));
    let req = Request::builder().uri("/open").body(Body::empty()).unwrap();
    let resp = unprotected.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

/// Tokens are also read from `token` and `access_token` query params, and permissive mode
/// lets anonymous requests through.
#[tokio::test]
async fn query_tokens_and_permissive_mode_are_accepted() {
    let validator: Arc<dyn TokenValidatorTrait> = Arc::new(StubValidator);

    let req_query = Request::builder()
        .uri("/data?token=valid_user_token")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        OauthHttpMiddleware::extract_token(&req_query),
        Some("valid_user_token".to_string())
    );

    let req_access_token = Request::builder()
        .uri("/data?access_token=my_custom_token")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        OauthHttpMiddleware::extract_token(&req_access_token),
        Some("my_custom_token".to_string())
    );

    let permissive = OauthHttpMiddleware::permissive(Some(validator.clone()));
    let app_permissive = Router::new()
        .route(
            "/check",
            get(|req: Request| async move {
                let has_user = req.extensions().get::<UserInfo>().is_some();
                Json(serde_json::json!({ "authenticated": has_user }))
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
    assert_eq!(json(resp_anon).await["authenticated"], false);

    let req_auth = Request::builder()
        .uri("/check?token=valid_root_token")
        .body(Body::empty())
        .unwrap();
    let resp_auth = app_permissive.oneshot(req_auth).await.unwrap();
    assert_eq!(resp_auth.status(), StatusCode::OK);
    assert_eq!(json(resp_auth).await["authenticated"], true);
}
