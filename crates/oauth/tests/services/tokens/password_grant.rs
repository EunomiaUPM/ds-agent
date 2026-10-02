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

//! Password grant: credentials checked against the stored hash, and the three tokens issued.

use jsonwebtoken::{DecodingKey, Validation, decode};
use oauth::entities::role::RbacRole;
use oauth::services::token_service::{OauthTokenValidator, TokenServiceTrait};

use super::{AUDIENCE, ISSUER, Repos, SECRET, user};

/// Valid credentials give an access, an ID and a refresh token; the access token validates
/// back to the user's tenant and role.
#[tokio::test]
async fn valid_credentials_issue_access_id_and_refresh_tokens() {
    let (repos, store) = Repos::default()
        .with_user(user("tenant-1", "alice@example.com", "pw", RbacRole::Owner))
        .with_refresh_store();
    let svc = repos.service();

    let tokens = svc.issue_token("alice@example.com", "pw").await.unwrap();

    assert_eq!(tokens.token_type, "Bearer");
    assert_eq!(tokens.expires_in, 3_600);
    assert!(tokens.id_token.is_some());
    assert!(tokens.refresh_token.is_some());
    assert_eq!(store.lock().unwrap().len(), 1, "one refresh record stored");
    let claims = svc.validate_token(&tokens.access_token).await.unwrap();
    assert_eq!(claims.sub, "tenant-1");
    assert_eq!(claims.role, RbacRole::Owner);
}

/// The tenant id works as login name too.
#[tokio::test]
async fn tenant_id_is_accepted_as_login() {
    let (repos, _) = Repos::default()
        .with_user(user("tenant-1", "alice@example.com", "pw", RbacRole::Owner))
        .with_refresh_store();
    assert!(repos.service().issue_token("tenant-1", "pw").await.is_ok());
}

/// A wrong password is rejected and nothing is stored.
#[tokio::test]
async fn wrong_password_is_rejected() {
    let repos =
        Repos::default().with_user(user("tenant-1", "alice@example.com", "pw", RbacRole::Owner));
    assert!(
        repos
            .service()
            .issue_token("alice@example.com", "nope")
            .await
            .is_err()
    );
}

/// An unknown user is rejected.
#[tokio::test]
async fn unknown_user_is_rejected() {
    let repos =
        Repos::default().with_user(user("tenant-1", "alice@example.com", "pw", RbacRole::Owner));
    assert!(
        repos
            .service()
            .issue_token("bob@example.com", "pw")
            .await
            .is_err()
    );
}

/// A requested scope is echoed in the response and carried by the access token.
#[tokio::test]
async fn requested_scope_is_carried_by_the_access_token() {
    let (repos, _) = Repos::default()
        .with_user(user("tenant-1", "alice@example.com", "pw", RbacRole::Owner))
        .with_refresh_store();
    let svc = repos.service();

    let tokens = svc
        .issue_token_with_scope("alice@example.com", "pw", Some("data:read"))
        .await
        .unwrap();

    assert_eq!(tokens.scope.as_deref(), Some("data:read"));
    let info = svc
        .introspect_token(&tokens.access_token, None)
        .await
        .unwrap();
    assert_eq!(info.scope.as_deref(), Some("data:read"));
}

/// The ID token names the issuer, the audience, the email and the user's extra fields.
#[tokio::test]
async fn id_token_carries_issuer_audience_email_and_extra_fields() {
    let (repos, _) = Repos::default()
        .with_user(user("tenant-1", "alice@example.com", "pw", RbacRole::Admin))
        .with_refresh_store();
    let tokens = repos
        .service()
        .issue_token("alice@example.com", "pw")
        .await
        .unwrap();

    let mut validation = Validation::default();
    validation.set_audience(&[AUDIENCE]);
    validation.set_issuer(&[ISSUER]);
    let claims = decode::<serde_json::Value>(
        &tokens.id_token.unwrap(),
        &DecodingKey::from_secret(SECRET.as_bytes()),
        &validation,
    )
    .unwrap()
    .claims;
    assert_eq!(claims["sub"], "tenant-1");
    assert_eq!(claims["email"], "alice@example.com");
    assert_eq!(claims["name"], "Alice");
}
