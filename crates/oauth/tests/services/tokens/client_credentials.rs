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

//! Client credentials grant: secret check and the scopes a client may obtain.

use oauth::services::token_service::TokenServiceTrait;

use super::{Repos, client};

/// Repos whose only client is `app` of `tenant-7`, secret `s3cret`, allowed `scopes`.
fn with_client(scopes: &[&str]) -> Repos {
    let mut repos = Repos::default();
    let app = client("app", "tenant-7", "s3cret", scopes);
    repos
        .clients
        .expect_get_by_client_id()
        .returning(move |id| Ok((id == "app").then(|| app.clone())));
    repos
}

/// A valid secret gives an access token for the client's tenant, with no refresh or ID token.
#[tokio::test]
async fn valid_secret_issues_an_access_token_only() {
    let svc = with_client(&["data:read"]).service();

    let tokens = svc
        .issue_client_credentials_token("app", "s3cret", None)
        .await
        .unwrap();

    assert!(tokens.refresh_token.is_none());
    assert!(tokens.id_token.is_none());
    let info = svc
        .introspect_token(&tokens.access_token, None)
        .await
        .unwrap();
    assert!(info.active);
    assert_eq!(info.sub.as_deref(), Some("tenant-7"));
    assert_eq!(info.client_id.as_deref(), Some("app"));
}

/// A wrong secret or an unknown client is rejected.
#[tokio::test]
async fn wrong_secret_or_unknown_client_is_rejected() {
    let svc = with_client(&[]).service();
    assert!(
        svc.issue_client_credentials_token("app", "nope", None)
            .await
            .is_err()
    );
    assert!(
        svc.issue_client_credentials_token("other", "s3cret", None)
            .await
            .is_err()
    );
}

/// Without a requested scope the client gets all its allowed scopes.
#[tokio::test]
async fn no_requested_scope_grants_every_allowed_scope() {
    let tokens = with_client(&["data:read", "data:write"])
        .service()
        .issue_client_credentials_token("app", "s3cret", None)
        .await
        .unwrap();
    assert_eq!(tokens.scope.as_deref(), Some("data:read data:write"));
}

/// A requested subset of the allowed scopes is granted as asked.
#[tokio::test]
async fn requested_subset_of_allowed_scopes_is_granted() {
    let tokens = with_client(&["data:read", "data:write"])
        .service()
        .issue_client_credentials_token("app", "s3cret", Some("data:read"))
        .await
        .unwrap();
    assert_eq!(tokens.scope.as_deref(), Some("data:read"));
}

/// Asking for a scope the client is not allowed is rejected.
#[tokio::test]
async fn scope_beyond_the_allowed_ones_is_rejected() {
    let result = with_client(&["data:read"])
        .service()
        .issue_client_credentials_token("app", "s3cret", Some("data:read admin"))
        .await;
    assert!(result.is_err());
}

/// A client with no scope list is not restricted: any requested scope is granted.
#[tokio::test]
async fn client_without_scope_list_gets_what_it_asks() {
    let tokens = with_client(&[])
        .service()
        .issue_client_credentials_token("app", "s3cret", Some("anything"))
        .await
        .unwrap();
    assert_eq!(tokens.scope.as_deref(), Some("anything"));
}

/// A JWT bearer assertion (RFC 7523) from a known client, which carries no `typ`, still gets
/// an access token with the client's scopes.
#[tokio::test]
async fn jwt_bearer_assertion_issues_a_token_for_the_client() {
    let svc = with_client(&["data:read"]).service();
    let now = chrono::Utc::now().timestamp();
    let assertion = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &serde_json::json!({ "iss": "app", "sub": "app", "exp": now + 60 }),
        &jsonwebtoken::EncodingKey::from_secret(super::SECRET.as_bytes()),
    )
    .unwrap();

    let tokens = svc.issue_jwt_bearer_token(&assertion, None).await.unwrap();

    assert_eq!(tokens.scope.as_deref(), Some("data:read"));
    let info = svc
        .introspect_token(&tokens.access_token, None)
        .await
        .unwrap();
    assert_eq!(info.client_id.as_deref(), Some("app"));
}
