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

//! OauthAuthenticator: rejects wrong configs and fails cleanly when the token URL is down.

use dataplane::engine::dataplane_drivers::DriverAuthenticatorTrait;

use dataplane::engine::dataplane_drivers::authentication::oauth::*;

use crate::support::fixtures::{
    bearer_context, consumer_context, oauth2_context, oauth2_password_context,
};

/// A connector configured with another auth type is rejected.
#[tokio::test]
async fn returns_error_for_wrong_auth_type() {
    let ctx = bearer_context("token").await;
    let result = OauthAuthenticator.authenticate(&ctx).await;
    assert!(result.is_err());
}

/// A context without connector instance is rejected.
#[tokio::test]
async fn returns_error_without_connector() {
    let ctx = consumer_context().await;
    let result = OauthAuthenticator.authenticate(&ctx).await;
    assert!(result.is_err());
}

/// Validates graceful failure when the token URL is unreachable (client_credentials).
#[tokio::test]
async fn returns_error_when_token_url_unreachable_client_credentials() {
    let ctx = oauth2_context("http://127.0.0.1:1", "client-id", "secret").await;
    let result = OauthAuthenticator.authenticate(&ctx).await;
    assert!(result.is_err());
}

/// Validates graceful failure when the token URL is unreachable (password grant).
#[tokio::test]
async fn returns_error_when_token_url_unreachable_password_grant() {
    let ctx =
        oauth2_password_context("http://127.0.0.1:1", "client-id", "secret", "user", "pass").await;
    let result = OauthAuthenticator.authenticate(&ctx).await;
    assert!(result.is_err());
}
