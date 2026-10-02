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

//! Token validation and RFC 7662 introspection for JWTs and personal access tokens.

use chrono::{Duration, Utc};
use oauth::entities::pat::PersonalAccessToken;
use oauth::entities::role::RbacRole;
use oauth::services::token_service::{OauthTokenValidator, TokenServiceTrait};
use uuid::Uuid;

use super::{Repos, config, user};

const RAW_PAT: &str = "pat_0123456789abcdef";

fn pat(revoked: bool, expires_in: Option<Duration>) -> PersonalAccessToken {
    PersonalAccessToken {
        id: Uuid::new_v4(),
        tenant_id: "tenant-1".to_string(),
        name: "ci".to_string(),
        token_prefix: "pat_0123".to_string(),
        token_hash: PersonalAccessToken::hash_token(RAW_PAT),
        role: RbacRole::Reader,
        scopes: vec!["data:read".to_string()],
        expires_at: expires_in.map(|d| Utc::now() + d),
        created_at: Utc::now(),
        last_used_at: None,
        revoked,
    }
}

/// Repos whose only PAT is `stored`.
fn with_pat(stored: PersonalAccessToken) -> Repos {
    let mut repos = Repos::default();
    repos
        .pats
        .expect_get_by_hash()
        .returning(move |h| Ok((h == stored.token_hash).then(|| stored.clone())));
    repos.pats.expect_update_last_used().returning(|_| Ok(()));
    repos
}

/// A service with user `alice` and her first token set.
async fn logged_in(
    config: oauth::config::OAuthConfig,
) -> (
    oauth::services::token_service::service::TokenService,
    oauth::services::token_service::views::TokenResponse,
) {
    let (repos, _) = Repos::default()
        .with_user(user("tenant-1", "alice@example.com", "pw", RbacRole::Owner))
        .with_refresh_store();
    let svc = repos.service_with(config);
    let tokens = svc.issue_token("alice@example.com", "pw").await.unwrap();
    (svc, tokens)
}

/// A live access token introspects as active, with its subject and role.
#[tokio::test]
async fn access_token_introspects_as_active() {
    let (svc, tokens) = logged_in(config()).await;
    let info = svc
        .introspect_token(&tokens.access_token, None)
        .await
        .unwrap();
    assert!(info.active);
    assert_eq!(info.sub.as_deref(), Some("tenant-1"));
    assert_eq!(info.token_type.as_deref(), Some("Bearer"));
    assert_eq!(info.role.as_deref(), Some("owner"));
}

/// An expired access token neither validates nor introspects as active.
#[tokio::test]
async fn expired_access_token_is_rejected() {
    let (svc, tokens) = logged_in(config().with_access_ttl(-120)).await;
    assert!(svc.validate_token(&tokens.access_token).await.is_err());
    let info = svc
        .introspect_token(&tokens.access_token, None)
        .await
        .unwrap();
    assert!(!info.active);
}

/// A token signed with another secret is rejected.
#[tokio::test]
async fn token_signed_with_another_secret_is_rejected() {
    let (_, foreign) = logged_in(oauth::config::OAuthConfig::new("other", "iss", "aud")).await;
    let (svc, _) = logged_in(config()).await;
    assert!(svc.validate_token(&foreign.access_token).await.is_err());
    assert!(
        !svc.introspect_token(&foreign.access_token, None)
            .await
            .unwrap()
            .active
    );
}

/// Garbage introspects as inactive instead of failing.
#[tokio::test]
async fn garbage_introspects_as_inactive() {
    let (svc, _) = logged_in(config()).await;
    assert!(!svc.introspect_token("garbage", None).await.unwrap().active);
}

/// A refresh token introspects as active with the refresh_token hint, and inactive once revoked.
#[tokio::test]
async fn refresh_token_introspects_until_revoked() {
    let (svc, tokens) = logged_in(config()).await;
    let refresh = tokens.refresh_token.unwrap();

    let info = svc
        .introspect_token(&refresh, Some("refresh_token"))
        .await
        .unwrap();
    assert!(info.active);
    assert_eq!(info.token_type.as_deref(), Some("refresh_token"));

    svc.revoke_refresh_token(&refresh).await.unwrap();
    assert!(
        !svc.introspect_token(&refresh, Some("refresh_token"))
            .await
            .unwrap()
            .active
    );
}

/// An active PAT validates as its tenant and role.
#[tokio::test]
async fn active_pat_validates_as_its_tenant() {
    let svc = with_pat(pat(false, Some(Duration::days(1)))).service();
    let claims = svc.validate_token(RAW_PAT).await.unwrap();
    assert_eq!(claims.sub, "tenant-1");
    assert_eq!(claims.role, RbacRole::Reader);
}

/// A revoked, expired or unknown PAT is rejected.
#[tokio::test]
async fn revoked_expired_or_unknown_pats_are_rejected() {
    for stored in [pat(true, None), pat(false, Some(Duration::seconds(-1)))] {
        assert!(
            with_pat(stored)
                .service()
                .validate_token(RAW_PAT)
                .await
                .is_err()
        );
    }
    let svc = with_pat(pat(false, None)).service();
    assert!(svc.validate_token("pat_unknown").await.is_err());
}

/// An active PAT introspects with its scopes and type; a revoked one is inactive.
#[tokio::test]
async fn pat_introspection_reflects_its_state() {
    let info = with_pat(pat(false, None))
        .service()
        .introspect_token(RAW_PAT, None)
        .await
        .unwrap();
    assert!(info.active);
    assert_eq!(info.token_type.as_deref(), Some("pat"));
    assert_eq!(info.scope.as_deref(), Some("data:read"));

    let revoked = with_pat(pat(true, None))
        .service()
        .introspect_token(RAW_PAT, None)
        .await
        .unwrap();
    assert!(!revoked.active);
}

/// Revoking a PAT revokes it in its own tenant.
#[tokio::test]
async fn revoking_a_pat_revokes_it_in_its_tenant() {
    let stored = pat(false, None);
    let id = stored.id;
    let mut repos = with_pat(stored);
    repos
        .pats
        .expect_revoke()
        .withf(move |tenant, pat_id| tenant.as_deref() == Some("tenant-1") && *pat_id == id)
        .times(1)
        .returning(|_, _| Ok("revoked".to_string()));
    repos.service().revoke_token(RAW_PAT, None).await.unwrap();
}

/// A refresh token must not pass as an access token.
#[tokio::test]
async fn refresh_token_is_not_an_access_token() {
    let (svc, tokens) = logged_in(config()).await;
    assert!(
        svc.validate_token(&tokens.refresh_token.unwrap())
            .await
            .is_err()
    );
}

/// An ID token must not pass as an access token.
#[tokio::test]
async fn id_token_is_not_an_access_token() {
    let (svc, tokens) = logged_in(config()).await;
    assert!(svc.validate_token(&tokens.id_token.unwrap()).await.is_err());
}

/// Without a hint, a refresh token introspects as a refresh token, not as a bearer.
#[tokio::test]
async fn refresh_token_without_hint_introspects_as_refresh() {
    let (svc, tokens) = logged_in(config()).await;
    let info = svc
        .introspect_token(&tokens.refresh_token.unwrap(), None)
        .await
        .unwrap();
    assert!(info.active);
    assert_eq!(info.token_type.as_deref(), Some("refresh_token"));
}

/// An access token cannot be revoked as if it were a refresh token, nor introspect as one.
#[tokio::test]
async fn access_token_is_not_a_refresh_token() {
    let (svc, tokens) = logged_in(config()).await;
    let info = svc
        .introspect_token(&tokens.access_token, Some("refresh_token"))
        .await
        .unwrap();
    assert!(!info.active);
}
