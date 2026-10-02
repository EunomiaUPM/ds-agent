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

//! Refresh grant and revocation: rotation, reuse after rotation, and revoking tokens.

use oauth::entities::role::RbacRole;
use oauth::services::token_service::TokenServiceTrait;

use super::{Repos, user};

/// A service with user `alice` and an in-memory refresh store, plus her first token set.
async fn logged_in() -> (
    oauth::services::token_service::service::TokenService,
    std::sync::Arc<std::sync::Mutex<Vec<oauth::entities::refresh_token::RefreshToken>>>,
    oauth::services::token_service::views::TokenResponse,
) {
    let (repos, store) = Repos::default()
        .with_user(user("tenant-1", "alice@example.com", "pw", RbacRole::Owner))
        .with_refresh_store();
    let svc = repos.service();
    let tokens = svc.issue_token("alice@example.com", "pw").await.unwrap();
    (svc, store, tokens)
}

/// Refreshing revokes the old refresh token and issues a new one.
#[tokio::test]
async fn refresh_rotates_the_refresh_token() {
    let (svc, store, tokens) = logged_in().await;
    let old = tokens.refresh_token.unwrap();

    let renewed = svc.refresh_token(&old).await.unwrap();

    let new = renewed.refresh_token.unwrap();
    assert_ne!(new, old);
    let records = store.lock().unwrap();
    assert_eq!(records.len(), 2);
    assert!(records[0].revoked, "the old record is revoked");
    assert!(!records[1].revoked);
}

/// A refresh token that was already rotated cannot be used again.
#[tokio::test]
async fn reusing_a_rotated_refresh_token_is_rejected() {
    let (svc, _, tokens) = logged_in().await;
    let old = tokens.refresh_token.unwrap();
    svc.refresh_token(&old).await.unwrap();

    assert!(svc.refresh_token(&old).await.is_err());
}

/// A refresh token whose record is gone, an access token and garbage are all rejected.
#[tokio::test]
async fn unknown_access_or_garbage_tokens_cannot_refresh() {
    let (svc, store, tokens) = logged_in().await;
    assert!(svc.refresh_token(&tokens.access_token).await.is_err());
    assert!(svc.refresh_token("not-a-jwt").await.is_err());

    store.lock().unwrap().clear();
    assert!(
        svc.refresh_token(&tokens.refresh_token.unwrap())
            .await
            .is_err()
    );
}

/// Revoking a refresh token stops it from refreshing.
#[tokio::test]
async fn revoked_refresh_token_cannot_refresh() {
    let (svc, _, tokens) = logged_in().await;
    let refresh = tokens.refresh_token.unwrap();

    svc.revoke_refresh_token(&refresh).await.unwrap();

    assert!(svc.refresh_token(&refresh).await.is_err());
}

/// RFC 7009 revocation revokes a refresh token, but not with an access_token hint.
#[tokio::test]
async fn revoke_token_honours_the_type_hint() {
    let (svc, store, tokens) = logged_in().await;
    let refresh = tokens.refresh_token.unwrap();

    svc.revoke_token(&refresh, Some("access_token"))
        .await
        .unwrap();
    assert!(!store.lock().unwrap()[0].revoked);

    svc.revoke_token(&refresh, None).await.unwrap();
    assert!(store.lock().unwrap()[0].revoked);
}

/// Revoking an unknown token is not an error (RFC 7009).
#[tokio::test]
async fn revoking_an_unknown_token_succeeds() {
    let (svc, _, _) = logged_in().await;
    assert!(svc.revoke_token("not-a-jwt", None).await.is_ok());
}
