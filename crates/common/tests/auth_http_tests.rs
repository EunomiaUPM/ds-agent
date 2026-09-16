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

use axum::body::Body;
use axum::extract::Request;
use axum::http::header::AUTHORIZATION;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router, middleware};
use common::auth::http::{AuthClaims, AuthHttpMiddleware};
use common::auth::{AccessScope, Claims, OauthTokenValidator, Rbac, RbacRole};
use tower::ServiceExt;
use ymir::errors::{Errors, Outcome};

struct MockValidator;

#[async_trait::async_trait]
impl OauthTokenValidator for MockValidator {
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

#[tokio::test]
async fn test_bearer_token_extraction() {
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_static("Bearer secret_jwt_token"),
    );
    assert_eq!(
        AuthHttpMiddleware::bearer(&headers).unwrap(),
        "secret_jwt_token"
    );

    // Whitespace trimming
    let mut headers_ws = HeaderMap::new();
    headers_ws.insert(
        AUTHORIZATION,
        HeaderValue::from_static("Bearer   spaced_jwt   "),
    );
    assert_eq!(
        AuthHttpMiddleware::bearer(&headers_ws).unwrap(),
        "spaced_jwt"
    );

    // Missing header
    let empty_headers = HeaderMap::new();
    assert!(AuthHttpMiddleware::bearer(&empty_headers).is_err());

    // Non-bearer header
    let mut basic_headers = HeaderMap::new();
    basic_headers.insert(
        AUTHORIZATION,
        HeaderValue::from_static("Basic dXNlcjpwYXNz"),
    );
    assert!(AuthHttpMiddleware::bearer(&basic_headers).is_err());

    // Empty bearer token
    let mut blank_bearer = HeaderMap::new();
    blank_bearer.insert(AUTHORIZATION, HeaderValue::from_static("Bearer   "));
    assert!(AuthHttpMiddleware::bearer(&blank_bearer).is_err());
}

#[tokio::test]
async fn test_auth_http_middleware_and_extractor_integration() {
    let validator: Arc<dyn OauthTokenValidator> = Arc::new(MockValidator);

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

    // 1. Successful request with admin token
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

    // 2. Rejected request with invalid token
    let req_bad = Request::builder()
        .uri("/protected")
        .header(AUTHORIZATION, "Bearer invalid_token")
        .body(Body::empty())
        .unwrap();
    let resp_bad = app.clone().oneshot(req_bad).await.unwrap();
    assert_eq!(resp_bad.status(), StatusCode::UNAUTHORIZED);

    // 3. Rejected request without authorization header
    let req_missing = Request::builder()
        .uri("/protected")
        .body(Body::empty())
        .unwrap();
    let resp_missing = app.oneshot(req_missing).await.unwrap();
    assert_eq!(resp_missing.status(), StatusCode::UNAUTHORIZED);
}

#[test]
fn test_access_scope_and_rbac_guards() {
    let admin_claims = Claims {
        sub: "admin-system".to_string(),
        role: RbacRole::Admin,
        iat: 100,
        exp: 200,
    };
    let user_claims = Claims {
        sub: "tenant-alpha".to_string(),
        role: RbacRole::Owner,
        iat: 100,
        exp: 200,
    };
    let reader_claims = Claims {
        sub: "tenant-alpha".to_string(),
        role: RbacRole::Reader,
        iat: 100,
        exp: 200,
    };

    // Rbac checks
    assert!(Rbac::require_admin(&admin_claims).is_ok());
    assert!(Rbac::require_admin(&user_claims).is_err());

    assert!(Rbac::require_read(&admin_claims, "any-tenant").is_ok());
    assert!(Rbac::require_read(&user_claims, "tenant-alpha").is_ok());
    assert!(Rbac::require_read(&user_claims, "tenant-beta").is_err());

    assert!(Rbac::require_write(&admin_claims, "any-tenant").is_ok());
    assert!(Rbac::require_write(&user_claims, "tenant-alpha").is_ok());
    assert!(Rbac::require_write(&user_claims, "tenant-beta").is_err());
    assert!(Rbac::require_write(&reader_claims, "tenant-alpha").is_err());

    // AccessScope with &str
    let admin_scope = AccessScope::for_read(&admin_claims, "foreign-tenant").unwrap();
    assert_eq!(admin_scope.tenant_filter(), None);
    assert!(admin_scope.permits("any-other-tenant"));

    let user_scope = AccessScope::for_write(&user_claims, "tenant-alpha").unwrap();
    assert_eq!(user_scope.tenant_filter(), Some("tenant-alpha".to_string()));
    assert!(user_scope.permits("tenant-alpha"));
    assert!(!user_scope.permits("tenant-beta"));
}

#[test]
fn test_claims_and_roles_helpers() {
    let role = RbacRole::Owner;
    assert_eq!(role.as_str(), "owner");
    assert!(!role.is_admin());
    assert!(role.can_write());

    let claims = Claims {
        sub: "tenant-123".to_string(),
        role: RbacRole::Admin,
        iat: 1000,
        exp: 2000,
    };
    assert_eq!(claims.tenant_id(), "tenant-123");
    assert!(claims.is_admin());
    assert!(!claims.is_expired(1500));
    assert!(claims.is_expired(2500));
}

#[test]
fn test_auth_validators_level1() {
    let now = 1000;
    let claims_val = common::auth::validators::AuthValidators::claims_validator_at(now);

    let valid_claims = Claims {
        sub: "tenant-42".to_string(),
        role: RbacRole::Owner,
        iat: 500,
        exp: 1500,
    };
    assert!(claims_val.validate(&valid_claims).is_ok());

    let empty_sub = Claims {
        sub: "  ".to_string(),
        role: RbacRole::Owner,
        iat: 500,
        exp: 1500,
    };
    let vs = claims_val.validate(&empty_sub).unwrap_err();
    assert_eq!(vs.code(), Some(common::validation::codes::MISSING));

    let expired = Claims {
        sub: "tenant-42".to_string(),
        role: RbacRole::Owner,
        iat: 100,
        exp: 900,
    };
    let vs = claims_val.validate(&expired).unwrap_err();
    assert_eq!(vs.code(), Some(common::validation::codes::NOT_ALLOWED));

    let tenant_val = common::auth::validators::AuthValidators::tenant_id_validator();
    assert!(tenant_val.validate(&"valid-tenant.1".to_string()).is_ok());
    assert!(tenant_val.validate(&"".to_string()).is_err());
    assert!(tenant_val.validate(&"invalid/tenant#".to_string()).is_err());
}

#[test]
fn test_service_level_authorization() {
    let admin_scope = AccessScope::from_role(RbacRole::Admin, "system");
    let owner_scope = AccessScope::from_role(RbacRole::Owner, "tenant-a");
    let reader_scope = AccessScope::from_role(RbacRole::Reader, "tenant-a");

    // Reads permitted for all valid authenticated scopes
    assert!(admin_scope.require_read().is_ok());
    assert!(owner_scope.require_read().is_ok());
    assert!(reader_scope.require_read().is_ok());

    // Writes rejected for reader role at the service layer
    assert!(admin_scope.require_write().is_ok());
    assert!(owner_scope.require_write().is_ok());
    assert!(reader_scope.require_write().is_err());

    // Tenant access boundary check
    assert!(admin_scope.ensure_tenant_access("tenant-b").is_ok());
    assert!(owner_scope.ensure_tenant_access("tenant-a").is_ok());
    assert!(owner_scope.ensure_tenant_access("tenant-b").is_err());
}

#[tokio::test]
async fn test_access_scope_extractor_with_validator() {
    let validator: Arc<dyn OauthTokenValidator> = Arc::new(MockValidator);

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

    // 1. User token with matching tenant header succeeds
    let req = Request::builder()
        .uri("/tenant-data")
        .header(AUTHORIZATION, "Bearer valid_user_token")
        .header("x-tenant-id", "tenant-42")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 2. User token with mismatching tenant header fails (403 Forbidden)
    let req_mismatch = Request::builder()
        .uri("/tenant-data")
        .header(AUTHORIZATION, "Bearer valid_user_token")
        .header("x-tenant-id", "other-tenant")
        .body(Body::empty())
        .unwrap();
    let resp_mismatch = app.clone().oneshot(req_mismatch).await.unwrap();
    assert_eq!(resp_mismatch.status(), StatusCode::FORBIDDEN);

    // 3. Admin token can access foreign tenant
    let req_admin = Request::builder()
        .uri("/tenant-data")
        .header(AUTHORIZATION, "Bearer valid_admin_token")
        .header("x-tenant-id", "other-tenant")
        .body(Body::empty())
        .unwrap();
    let resp_admin = app.clone().oneshot(req_admin).await.unwrap();
    assert_eq!(resp_admin.status(), StatusCode::OK);

    // 4. Invalid tenant format fails validation (400 Bad Request)
    let req_bad_tenant = Request::builder()
        .uri("/tenant-data")
        .header(AUTHORIZATION, "Bearer valid_admin_token")
        .header("x-tenant-id", "bad!tenant@id")
        .body(Body::empty())
        .unwrap();
    let resp_bad_tenant = app.oneshot(req_bad_tenant).await.unwrap();
    assert_eq!(resp_bad_tenant.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_auth_http_middleware_dual_token_and_modes() {
    let validator: Arc<dyn OauthTokenValidator> = Arc::new(MockValidator);

    // 1. Dual extraction: query param token
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

    // 2. Permissive mode: unauthenticated request proceeds without claims
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

