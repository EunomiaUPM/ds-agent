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

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use base64::Engine;
use http_body_util::BodyExt;
use sha2::{Digest, Sha256};
use tower::ServiceExt;

use crate::config::OAuthConfig;
use crate::data::factory::OAuthDataFactory;
use crate::data::in_memory::factory::InMemoryDataFactory;
use crate::entities::commands::{CreateClientCommand, CreateUserCommand};
use crate::entities::role::RbacRole;
use crate::http::clients_router::ClientsRouter;
use crate::http::errors::{OAuthError, OAuthErrorCode};
use crate::http::forms::{AuthorizeResponse, ClientView, IntrospectResponse, OpenIdConfiguration};
use crate::http::pats_router::PatsRouter;
use crate::http::token_router::TokenRouter;
use crate::http::users_router::UsersRouter;
use crate::services::client_service::ClientServiceTrait;
use crate::services::client_service::service::ClientService;
use crate::services::pat_service::PatServiceTrait;
use crate::services::pat_service::service::PatService;
use crate::services::pat_service::views::CreatePatResponse;
use crate::services::token_service::TokenServiceTrait;
use crate::services::token_service::jwt::JwtAssertionClaims;
use crate::services::token_service::service::TokenService;
use crate::services::token_service::views::TokenResponse;
use crate::services::user_service::UserServiceTrait;
use crate::services::user_service::service::UserService;

struct TestEnv {
    router: Router,
    token_svc: Arc<dyn TokenServiceTrait>,
    user_svc: Arc<dyn UserServiceTrait>,
    client_svc: Arc<dyn ClientServiceTrait>,
    pat_svc: Arc<dyn PatServiceTrait>,
    config: OAuthConfig,
}

impl TestEnv {
    async fn new() -> Self {
        let factory = InMemoryDataFactory::new();
        let config = OAuthConfig::new(
            "test-secret-key-that-is-sufficiently-long-for-hs256",
            "http://localhost:8080",
            "test-client",
        );
        let token_svc: Arc<dyn TokenServiceTrait> = Arc::new(TokenService::new(
            factory.user_repository(),
            factory.token_repository(),
            factory.client_repository(),
            factory.auth_code_repository(),
            factory.pat_repository(),
            config.clone(),
        ));
        let user_svc: Arc<dyn UserServiceTrait> =
            Arc::new(UserService::new(factory.user_repository()));
        let client_svc: Arc<dyn ClientServiceTrait> =
            Arc::new(ClientService::new(factory.client_repository()));
        let pat_svc: Arc<dyn PatServiceTrait> = Arc::new(PatService::new(factory.pat_repository()));

        let token_router =
            TokenRouter::new(token_svc.clone(), user_svc.clone(), config.issuer.clone()).router();
        let users_router = UsersRouter::new(token_svc.clone(), user_svc.clone()).router();
        let clients_router = ClientsRouter::new(token_svc.clone(), client_svc.clone()).router();
        let pats_router = PatsRouter::new(token_svc.clone(), pat_svc.clone()).router();

        let router = Router::new()
            .merge(token_router)
            .nest("/users", users_router)
            .nest("/clients", clients_router)
            .nest("/pats", pats_router);

        Self {
            router,
            token_svc,
            user_svc,
            client_svc,
            pat_svc,
            config,
        }
    }

    async fn seed_user(&self, tenant_id: &str, email: &str, password: &str, role: RbacRole) {
        self.user_svc
            .create_user(&CreateUserCommand {
                tenant_id: tenant_id.to_string(),
                email: email.to_string(),
                password: password.to_string(),
                role,
                extra_fields: serde_json::json!({}),
            })
            .await
            .unwrap();
    }

    async fn seed_client(
        &self,
        client_id: &str,
        client_secret: &str,
        role: RbacRole,
        scopes: Vec<String>,
    ) {
        self.client_svc
            .create_client(&CreateClientCommand {
                client_id: client_id.to_string(),
                client_secret: client_secret.to_string(),
                client_name: format!("Client {client_id}"),
                role,
                scopes,
            })
            .await
            .unwrap();
    }
}

async fn response_json<T: serde::de::DeserializeOwned>(resp: axum::response::Response) -> T {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn test_password_grant_form_and_json() {
    let env = TestEnv::new().await;
    env.seed_user(
        "tenant-1",
        "user@example.com",
        "Secret123!",
        RbacRole::Owner,
    )
    .await;

    // 1. Form urlencoded
    let req = Request::builder()
        .method("POST")
        .uri("/token")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(
            "grant_type=password&username=user@example.com&password=Secret123!",
        ))
        .unwrap();

    let resp = env.router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let token: TokenResponse = response_json(resp).await;
    assert_eq!(token.token_type, "Bearer");
    assert!(token.refresh_token.is_some());
    assert!(token.id_token.is_some());

    let claims = env
        .token_svc
        .validate_token(&token.access_token)
        .await
        .unwrap();
    assert_eq!(claims.sub, "tenant-1");
    assert_eq!(claims.role, RbacRole::Owner);

    // 2. JSON
    let req_json = Request::builder()
        .method("POST")
        .uri("/token")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_vec(&serde_json::json!({
                "grant_type": "password",
                "username": "tenant-1",
                "password": "Secret123!"
            }))
            .unwrap(),
        ))
        .unwrap();

    let resp_json = env.router.clone().oneshot(req_json).await.unwrap();
    assert_eq!(resp_json.status(), StatusCode::OK);

    // 3. Backward compatible (omitted grant_type)
    let req_legacy = Request::builder()
        .method("POST")
        .uri("/token")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_vec(&serde_json::json!({
                "username": "user@example.com",
                "password": "Secret123!"
            }))
            .unwrap(),
        ))
        .unwrap();

    let resp_legacy = env.router.clone().oneshot(req_legacy).await.unwrap();
    assert_eq!(resp_legacy.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_password_grant_invalid_credentials() {
    let env = TestEnv::new().await;
    env.seed_user(
        "tenant-1",
        "user@example.com",
        "Secret123!",
        RbacRole::Owner,
    )
    .await;

    let req = Request::builder()
        .method("POST")
        .uri("/token")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(
            "grant_type=password&username=user@example.com&password=WrongPassword",
        ))
        .unwrap();

    let resp = env.router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let err: OAuthError = response_json(resp).await;
    assert_eq!(err.error, OAuthErrorCode::InvalidGrant);
}

#[tokio::test]
async fn test_client_credentials_basic_auth_and_body() {
    let env = TestEnv::new().await;
    env.seed_client(
        "agent-connector-1",
        "client-secret-xyz",
        RbacRole::Admin,
        vec!["data:read".into(), "data:write".into()],
    )
    .await;

    // 1. Basic Auth
    let credentials =
        base64::engine::general_purpose::STANDARD.encode("agent-connector-1:client-secret-xyz");
    let req = Request::builder()
        .method("POST")
        .uri("/token")
        .header(header::AUTHORIZATION, format!("Basic {credentials}"))
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from("grant_type=client_credentials&scope=data:read"))
        .unwrap();

    let resp = env.router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let token: TokenResponse = response_json(resp).await;
    assert_eq!(token.token_type, "Bearer");
    assert_eq!(token.scope, Some("data:read".to_string()));
    assert!(token.refresh_token.is_none());
    assert!(token.id_token.is_none());

    let claims = env
        .token_svc
        .validate_token(&token.access_token)
        .await
        .unwrap();
    assert_eq!(claims.sub, "agent-connector-1");
    assert_eq!(claims.role, RbacRole::Admin);

    // 2. Body parameters
    let req_body = Request::builder()
        .method("POST")
        .uri("/token")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(
            "grant_type=client_credentials&client_id=agent-connector-1&client_secret=client-secret-xyz",
        ))
        .unwrap();

    let resp_body = env.router.clone().oneshot(req_body).await.unwrap();
    assert_eq!(resp_body.status(), StatusCode::OK);

    // 3. Unauthorized scope
    let req_bad_scope = Request::builder()
        .method("POST")
        .uri("/token")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(
            "grant_type=client_credentials&client_id=agent-connector-1&client_secret=client-secret-xyz&scope=admin:all",
        ))
        .unwrap();

    let resp_bad_scope = env.router.clone().oneshot(req_bad_scope).await.unwrap();
    assert_eq!(resp_bad_scope.status(), StatusCode::BAD_REQUEST);
    let err: OAuthError = response_json(resp_bad_scope).await;
    assert_eq!(err.error, OAuthErrorCode::InvalidScope);

    // 4. Invalid client secret
    let req_bad_secret = Request::builder()
        .method("POST")
        .uri("/token")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(
            "grant_type=client_credentials&client_id=agent-connector-1&client_secret=wrong-secret",
        ))
        .unwrap();

    let resp_bad_secret = env.router.clone().oneshot(req_bad_secret).await.unwrap();
    assert_eq!(resp_bad_secret.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        resp_bad_secret
            .headers()
            .get(header::WWW_AUTHENTICATE)
            .unwrap(),
        "Basic realm=\"oauth\""
    );
    let err: OAuthError = response_json(resp_bad_secret).await;
    assert_eq!(err.error, OAuthErrorCode::InvalidClient);
}

#[tokio::test]
async fn test_refresh_token_rotation_and_revocation() {
    let env = TestEnv::new().await;
    env.seed_user(
        "tenant-2",
        "user2@example.com",
        "Secret123!",
        RbacRole::Owner,
    )
    .await;

    // Issue token
    let req = Request::builder()
        .method("POST")
        .uri("/token")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(
            "grant_type=password&username=user2@example.com&password=Secret123!",
        ))
        .unwrap();

    let resp = env.router.clone().oneshot(req).await.unwrap();
    let initial_token: TokenResponse = response_json(resp).await;
    let refresh1 = initial_token.refresh_token.unwrap();

    // Refresh at /token with grant_type=refresh_token
    let req_refresh = Request::builder()
        .method("POST")
        .uri("/token")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(format!(
            "grant_type=refresh_token&refresh_token={refresh1}"
        )))
        .unwrap();

    let resp_refresh = env.router.clone().oneshot(req_refresh).await.unwrap();
    assert_eq!(resp_refresh.status(), StatusCode::OK);
    let new_token: TokenResponse = response_json(resp_refresh).await;
    let refresh2 = new_token.refresh_token.unwrap();
    assert_ne!(refresh1, refresh2);

    // Old refresh token must now be revoked
    let req_reuse = Request::builder()
        .method("POST")
        .uri("/token")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(format!(
            "grant_type=refresh_token&refresh_token={refresh1}"
        )))
        .unwrap();

    let resp_reuse = env.router.clone().oneshot(req_reuse).await.unwrap();
    assert_eq!(resp_reuse.status(), StatusCode::BAD_REQUEST);

    // Legacy /refresh endpoint
    let req_legacy = Request::builder()
        .method("POST")
        .uri("/refresh")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(format!("refresh_token={refresh2}")))
        .unwrap();

    let resp_legacy = env.router.clone().oneshot(req_legacy).await.unwrap();
    assert_eq!(resp_legacy.status(), StatusCode::OK);
    let token3: TokenResponse = response_json(resp_legacy).await;
    let refresh3 = token3.refresh_token.unwrap();

    // Revoke at /revoke per RFC 7009
    let req_revoke = Request::builder()
        .method("POST")
        .uri("/revoke")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(format!("token={refresh3}")))
        .unwrap();

    let resp_revoke = env.router.clone().oneshot(req_revoke).await.unwrap();
    assert_eq!(resp_revoke.status(), StatusCode::OK);

    // Revoking already revoked or unknown token returns 200 OK
    let req_revoke_again = Request::builder()
        .method("POST")
        .uri("/revoke")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from("token=nonexistent-token"))
        .unwrap();

    let resp_revoke_again = env.router.clone().oneshot(req_revoke_again).await.unwrap();
    assert_eq!(resp_revoke_again.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_introspection_and_discovery() {
    let env = TestEnv::new().await;
    env.seed_user(
        "tenant-3",
        "user3@example.com",
        "Secret123!",
        RbacRole::Reader,
    )
    .await;

    // Issue token
    let req = Request::builder()
        .method("POST")
        .uri("/token")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(
            "grant_type=password&username=user3@example.com&password=Secret123!&scope=profile",
        ))
        .unwrap();

    let resp = env.router.clone().oneshot(req).await.unwrap();
    let token: TokenResponse = response_json(resp).await;

    // Introspect active access token
    let req_introspect = Request::builder()
        .method("POST")
        .uri("/introspect")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(format!("token={}", token.access_token)))
        .unwrap();

    let resp_introspect = env.router.clone().oneshot(req_introspect).await.unwrap();
    assert_eq!(resp_introspect.status(), StatusCode::OK);
    let intro: IntrospectResponse = response_json(resp_introspect).await;
    assert!(intro.active);
    assert_eq!(intro.sub, Some("tenant-3".to_string()));
    assert_eq!(intro.token_type, Some("Bearer".to_string()));
    assert_eq!(intro.role, Some("reader".to_string()));

    // Introspect invalid token
    let req_bad = Request::builder()
        .method("POST")
        .uri("/introspect")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from("token=garbage.token.here"))
        .unwrap();

    let resp_bad = env.router.clone().oneshot(req_bad).await.unwrap();
    let intro_bad: IntrospectResponse = response_json(resp_bad).await;
    assert!(!intro_bad.active);

    // Discovery document
    let req_disco = Request::builder()
        .method("GET")
        .uri("/.well-known/openid-configuration")
        .body(Body::empty())
        .unwrap();

    let resp_disco = env.router.clone().oneshot(req_disco).await.unwrap();
    assert_eq!(resp_disco.status(), StatusCode::OK);
    let config: OpenIdConfiguration = response_json(resp_disco).await;
    assert!(
        config
            .grant_types_supported
            .contains(&"client_credentials".to_string())
    );
    assert!(
        config
            .grant_types_supported
            .contains(&"refresh_token".to_string())
    );
    assert!(config.revocation_endpoint.ends_with("/revoke"));
    assert!(config.introspection_endpoint.ends_with("/introspect"));
}

#[tokio::test]
async fn test_client_crud_admin_endpoints() {
    let env = TestEnv::new().await;
    env.seed_user(
        "admin-user",
        "admin@example.com",
        "AdminPass123!",
        RbacRole::Admin,
    )
    .await;

    // Issue admin token
    let req = Request::builder()
        .method("POST")
        .uri("/token")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(
            "grant_type=password&username=admin@example.com&password=AdminPass123!",
        ))
        .unwrap();
    let resp = env.router.clone().oneshot(req).await.unwrap();
    let token: TokenResponse = response_json(resp).await;

    // Create client
    let req_create = Request::builder()
        .method("POST")
        .uri("/clients")
        .header(
            header::AUTHORIZATION,
            format!("Bearer {}", token.access_token),
        )
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_vec(&serde_json::json!({
                "clientId": "client-test-1",
                "clientSecret": "very-secret-key-123",
                "clientName": "Test Client",
                "role": "owner",
                "scopes": ["data:read"]
            }))
            .unwrap(),
        ))
        .unwrap();

    let resp_create = env.router.clone().oneshot(req_create).await.unwrap();
    assert_eq!(resp_create.status(), StatusCode::CREATED);
    let created: ClientView = response_json(resp_create).await;
    assert_eq!(created.client_id, "client-test-1");

    // List clients
    let req_list = Request::builder()
        .method("GET")
        .uri("/clients")
        .header(
            header::AUTHORIZATION,
            format!("Bearer {}", token.access_token),
        )
        .body(Body::empty())
        .unwrap();

    let resp_list = env.router.clone().oneshot(req_list).await.unwrap();
    assert_eq!(resp_list.status(), StatusCode::OK);
    let list: Vec<ClientView> = response_json(resp_list).await;
    assert_eq!(list.len(), 1);

    // Get client by id
    let req_get = Request::builder()
        .method("GET")
        .uri("/clients/client-test-1")
        .header(
            header::AUTHORIZATION,
            format!("Bearer {}", token.access_token),
        )
        .body(Body::empty())
        .unwrap();

    let resp_get = env.router.clone().oneshot(req_get).await.unwrap();
    assert_eq!(resp_get.status(), StatusCode::OK);

    // Delete client
    let req_del = Request::builder()
        .method("DELETE")
        .uri("/clients/client-test-1")
        .header(
            header::AUTHORIZATION,
            format!("Bearer {}", token.access_token),
        )
        .body(Body::empty())
        .unwrap();

    let resp_del = env.router.clone().oneshot(req_del).await.unwrap();
    assert_eq!(resp_del.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_authorization_code_pkce_flow() {
    let env = TestEnv::new().await;
    env.seed_user(
        "user-pkce",
        "pkce@example.com",
        "PkcePass123!",
        RbacRole::Owner,
    )
    .await;
    env.seed_client(
        "pkce-client",
        "client-secret",
        RbacRole::Owner,
        vec!["read".into()],
    )
    .await;

    // Code verifier and S256 code challenge (RFC 7636)
    let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(Sha256::digest(verifier.as_bytes()));

    // 1. Request authorization code via GET /authorize
    let auth_uri = format!(
        "/authorize?response_type=code&client_id=pkce-client&code_challenge={challenge}&code_challenge_method=S256&user_id=user-pkce&state=xyz123"
    );
    let req = Request::builder()
        .method("GET")
        .uri(&auth_uri)
        .body(Body::empty())
        .unwrap();
    let resp = env.router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let auth_res: AuthorizeResponse = response_json(resp).await;
    assert_eq!(auth_res.state.as_deref(), Some("xyz123"));
    assert!(!auth_res.code.is_empty());

    // 2. Exchange code with wrong verifier -> must fail
    let req_fail = Request::builder()
        .method("POST")
        .uri("/token")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(format!(
            "grant_type=authorization_code&code={}&code_verifier=wrong_verifier_here&client_id=pkce-client",
            auth_res.code
        )))
        .unwrap();
    let resp_fail = env.router.clone().oneshot(req_fail).await.unwrap();
    assert_eq!(resp_fail.status(), StatusCode::BAD_REQUEST);

    // 3. Exchange code with correct verifier -> succeeds
    let req_ok = Request::builder()
        .method("POST")
        .uri("/token")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(format!(
            "grant_type=authorization_code&code={}&code_verifier={verifier}&client_id=pkce-client",
            auth_res.code
        )))
        .unwrap();
    let resp_ok = env.router.clone().oneshot(req_ok).await.unwrap();
    assert_eq!(resp_ok.status(), StatusCode::OK);
    let token_res: TokenResponse = response_json(resp_ok).await;
    assert!(!token_res.access_token.is_empty());
    assert!(token_res.refresh_token.is_some());

    // 4. Reusing the authorization code -> must fail
    let req_reuse = Request::builder()
        .method("POST")
        .uri("/token")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(format!(
            "grant_type=authorization_code&code={}&code_verifier={verifier}&client_id=pkce-client",
            auth_res.code
        )))
        .unwrap();
    let resp_reuse = env.router.clone().oneshot(req_reuse).await.unwrap();
    assert_eq!(resp_reuse.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_personal_access_tokens() {
    let env = TestEnv::new().await;
    env.seed_user(
        "dev-tenant",
        "dev@example.com",
        "DevPass123!",
        RbacRole::Owner,
    )
    .await;

    // Obtain user JWT
    let req_login = Request::builder()
        .method("POST")
        .uri("/token")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(
            "grant_type=password&username=dev@example.com&password=DevPass123!",
        ))
        .unwrap();
    let resp_login = env.router.clone().oneshot(req_login).await.unwrap();
    let user_token: TokenResponse = response_json(resp_login).await;

    // Create PAT
    let req_create = Request::builder()
        .method("POST")
        .uri("/pats")
        .header(
            header::AUTHORIZATION,
            format!("Bearer {}", user_token.access_token),
        )
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_vec(&serde_json::json!({
                "name": "cli-access",
                "scopes": ["repo:read", "repo:write"]
            }))
            .unwrap(),
        ))
        .unwrap();
    let resp_create = env.router.clone().oneshot(req_create).await.unwrap();
    assert_eq!(resp_create.status(), StatusCode::CREATED);
    let pat_created: CreatePatResponse = response_json(resp_create).await;
    assert!(pat_created.token.starts_with("pat_"));

    // Authenticate with PAT as Bearer token against /userinfo
    let req_userinfo = Request::builder()
        .method("GET")
        .uri("/userinfo")
        .header(
            header::AUTHORIZATION,
            format!("Bearer {}", pat_created.token),
        )
        .body(Body::empty())
        .unwrap();
    let resp_userinfo = env.router.clone().oneshot(req_userinfo).await.unwrap();
    assert_eq!(resp_userinfo.status(), StatusCode::OK);

    // Introspect PAT via RFC 7662
    let req_intro = Request::builder()
        .method("POST")
        .uri("/introspect")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(format!("token={}", pat_created.token)))
        .unwrap();
    let resp_intro = env.router.clone().oneshot(req_intro).await.unwrap();
    assert_eq!(resp_intro.status(), StatusCode::OK);
    let intro: IntrospectResponse = response_json(resp_intro).await;
    assert!(intro.active);
    assert_eq!(intro.token_type.as_deref(), Some("pat"));
    assert_eq!(intro.sub.as_deref(), Some("dev-tenant"));

    // Revoke PAT via RFC 7009
    let req_revoke = Request::builder()
        .method("POST")
        .uri("/revoke")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(format!("token={}", pat_created.token)))
        .unwrap();
    let resp_revoke = env.router.clone().oneshot(req_revoke).await.unwrap();
    assert_eq!(resp_revoke.status(), StatusCode::OK);

    // PAT is now invalid
    let req_after = Request::builder()
        .method("GET")
        .uri("/userinfo")
        .header(
            header::AUTHORIZATION,
            format!("Bearer {}", pat_created.token),
        )
        .body(Body::empty())
        .unwrap();
    let resp_after = env.router.clone().oneshot(req_after).await.unwrap();
    assert_eq!(resp_after.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_rfc7523_jwt_bearer_m2m() {
    let env = TestEnv::new().await;
    env.seed_client(
        "m2m-service",
        "m2m-secret",
        RbacRole::Owner,
        vec!["data:read".into(), "data:write".into()],
    )
    .await;

    // Mint a signed JWT assertion for M2M grant
    let now = chrono::Utc::now().timestamp();
    let assertion_claims = JwtAssertionClaims {
        iss: "m2m-service".to_string(),
        sub: "m2m-service".to_string(),
        aud: Some(serde_json::Value::String(
            "http://localhost:8080".to_string(),
        )),
        exp: now + 3600,
        iat: Some(now),
        jti: Some("jwt-assertion-123".to_string()),
        scope: Some("data:read".to_string()),
    };
    let assertion = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &assertion_claims,
        &jsonwebtoken::EncodingKey::from_secret(env.config.jwt_secret.as_bytes()),
    )
    .unwrap();

    // 1. Exchange RFC 7523 JWT Bearer grant
    let req = Request::builder()
        .method("POST")
        .uri("/token")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(format!(
            "grant_type=urn:ietf:params:oauth:grant-type:jwt-bearer&assertion={assertion}&scope=data:read"
        )))
        .unwrap();
    let resp = env.router.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let token_res: TokenResponse = response_json(resp).await;
    assert!(!token_res.access_token.is_empty());
    assert_eq!(token_res.scope.as_deref(), Some("data:read"));

    // 2. RFC 7523 Client Authentication (private_key_jwt profile) with client_credentials
    let req_client_jwt = Request::builder()
        .method("POST")
        .uri("/token")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(format!(
            "grant_type=client_credentials&client_assertion_type=urn:ietf:params:oauth:client-assertion-type:jwt-bearer&client_assertion={assertion}"
        )))
        .unwrap();
    let resp_client_jwt = env.router.clone().oneshot(req_client_jwt).await.unwrap();
    assert_eq!(resp_client_jwt.status(), StatusCode::OK);
    let token_res_jwt: TokenResponse = response_json(resp_client_jwt).await;
    assert!(!token_res_jwt.access_token.is_empty());
}
