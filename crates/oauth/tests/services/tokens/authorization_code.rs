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

//! Authorization code grant with PKCE: issuing a code bound to a challenge, and exchanging it.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{Duration, Utc};
use oauth::entities::auth_code::AuthCode;
use oauth::entities::role::RbacRole;
use oauth::services::token_service::TokenServiceTrait;
use sha2::{Digest, Sha256};

use super::{Repos, client, user};

const VERIFIER: &str = "dBjftJeZ4CVP-mJ92K9qwqpWyQ1VzDJ6pAgSnByg1e0";

fn s256(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

/// Repos with client `app` and user `alice` of `tenant-1`.
fn issuing() -> Repos {
    let mut repos =
        Repos::default().with_user(user("tenant-1", "alice@example.com", "pw", RbacRole::Owner));
    let app = client("app", "tenant-1", "s3cret", &[]);
    repos
        .clients
        .expect_get_by_client_id()
        .returning(move |id| Ok((id == "app").then(|| app.clone())));
    repos
}

/// An unused S256 code for `app` and `tenant-1`, valid for ten minutes.
fn code() -> AuthCode {
    AuthCode {
        code: "code-1".to_string(),
        client_id: "app".to_string(),
        redirect_uri: Some("https://app/cb".to_string()),
        tenant_id: "tenant-1".to_string(),
        role: RbacRole::Owner,
        scopes: vec!["data:read".to_string()],
        code_challenge: s256(VERIFIER),
        code_challenge_method: "S256".to_string(),
        expires_at: Utc::now() + Duration::minutes(10),
        used: false,
    }
}

/// Repos where `stored` is the only code.
fn exchanging(stored: AuthCode) -> Repos {
    let mut repos =
        Repos::default().with_user(user("tenant-1", "alice@example.com", "pw", RbacRole::Owner));
    repos
        .codes
        .expect_get_by_code()
        .returning(move |c| Ok((c == stored.code).then(|| stored.clone())));
    repos
}

/// The code is saved bound to the user, the challenge and the requested scopes; S256 is the
/// default method.
#[tokio::test]
async fn code_is_saved_bound_to_user_challenge_and_scopes() {
    let mut repos = issuing();
    repos
        .codes
        .expect_save()
        .withf(|c| {
            c.tenant_id == "tenant-1"
                && c.code_challenge == "challenge"
                && c.code_challenge_method == "S256"
                && c.scopes == ["data:read", "data:write"]
                && !c.used
                && c.expires_at > Utc::now()
        })
        .times(1)
        .returning(|c| Ok(c.clone()));

    let code = repos
        .service()
        .issue_authorization_code(
            "app",
            Some("https://app/cb"),
            Some("data:read data:write"),
            "challenge",
            None,
            Some("alice@example.com"),
        )
        .await
        .unwrap();
    assert!(!code.is_empty());
}

/// Issuing needs a code challenge, a supported method, a known client and a user.
#[tokio::test]
async fn issuing_rejects_missing_challenge_bad_method_unknown_client_or_no_user() {
    let svc = issuing().service();
    let user = Some("tenant-1");
    assert!(
        svc.issue_authorization_code("app", None, None, " ", None, user)
            .await
            .is_err()
    );
    assert!(
        svc.issue_authorization_code("app", None, None, "c", Some("S512"), user)
            .await
            .is_err()
    );
    assert!(
        svc.issue_authorization_code("other", None, None, "c", None, user)
            .await
            .is_err()
    );
    assert!(
        svc.issue_authorization_code("app", None, None, "c", None, None)
            .await
            .is_err()
    );
}

/// The right verifier exchanges the code for tokens and burns it.
#[tokio::test]
async fn valid_verifier_exchanges_the_code_and_burns_it() {
    let (mut repos, _) = exchanging(code()).with_refresh_store();
    repos
        .codes
        .expect_mark_used()
        .withf(|c| c == "code-1")
        .times(1)
        .returning(|_| Ok(()));

    let tokens = repos
        .service()
        .exchange_authorization_code("code-1", VERIFIER, Some("https://app/cb"), Some("app"))
        .await
        .unwrap();

    assert_eq!(tokens.scope.as_deref(), Some("data:read"));
    assert!(tokens.refresh_token.is_some());
    assert!(tokens.id_token.is_some());
}

/// With the plain method the verifier is compared as is.
#[tokio::test]
async fn plain_method_compares_the_verifier_directly() {
    let stored = AuthCode {
        code_challenge: "plain-secret".to_string(),
        code_challenge_method: "plain".to_string(),
        ..code()
    };
    let (mut repos, _) = exchanging(stored).with_refresh_store();
    repos.codes.expect_mark_used().returning(|_| Ok(()));

    let result = repos
        .service()
        .exchange_authorization_code("code-1", "plain-secret", Some("https://app/cb"), None)
        .await;
    assert!(result.is_ok());
}

/// A wrong verifier is rejected and the code is not burnt.
#[tokio::test]
async fn wrong_verifier_is_rejected_without_burning_the_code() {
    let svc = exchanging(code()).service();
    let result = svc
        .exchange_authorization_code("code-1", "not-the-verifier", Some("https://app/cb"), None)
        .await;
    assert!(result.is_err());
}

/// A used, expired or unknown code is rejected.
#[tokio::test]
async fn used_expired_or_unknown_codes_are_rejected() {
    let used = AuthCode {
        used: true,
        ..code()
    };
    let expired = AuthCode {
        expires_at: Utc::now() - Duration::seconds(1),
        ..code()
    };
    for stored in [used, expired] {
        let result = exchanging(stored)
            .service()
            .exchange_authorization_code("code-1", VERIFIER, Some("https://app/cb"), None)
            .await;
        assert!(result.is_err());
    }
    let result = exchanging(code())
        .service()
        .exchange_authorization_code("other", VERIFIER, Some("https://app/cb"), None)
        .await;
    assert!(result.is_err());
}

/// The code only exchanges for the client and redirect URI it was issued to.
#[tokio::test]
async fn client_or_redirect_mismatch_is_rejected() {
    let svc = exchanging(code()).service();
    assert!(
        svc.exchange_authorization_code("code-1", VERIFIER, Some("https://app/cb"), Some("evil"))
            .await
            .is_err()
    );
    assert!(
        svc.exchange_authorization_code("code-1", VERIFIER, Some("https://evil/cb"), None)
            .await
            .is_err()
    );
    assert!(
        svc.exchange_authorization_code("code-1", VERIFIER, None, None)
            .await
            .is_err()
    );
}
