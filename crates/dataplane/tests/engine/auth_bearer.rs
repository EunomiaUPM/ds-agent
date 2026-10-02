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

//! BearerTokenAuthenticator: resolves the bearer token of the connector into the runtime.

use dataplane::engine::dataplane_drivers::DriverAuthenticatorTrait;
use dataplane::engine::dataplane_manager::dataplane_runtime::ResolvedAuthCredentials;

use dataplane::engine::dataplane_drivers::authentication::bearer_token::*;

use crate::support::fixtures::{bearer_context, consumer_context, no_auth_context};

/// A plain bearer token lands in the runtime.
#[tokio::test]
async fn resolves_plain_bearer_token() {
    let ctx = bearer_context("secret-token").await;
    let result = BearerTokenAuthenticator.authenticate(&ctx).await;
    assert!(result.is_ok());
    let updated = result.unwrap();
    let runtime = updated.runtime().expect("runtime must be set after auth");
    match &runtime.auth {
        ResolvedAuthCredentials::BearerToken { token } => {
            assert_eq!(token, "secret-token");
        }
        other => panic!("Expected BearerToken, got {other:?}"),
    }
}

/// A connector configured with another auth type is rejected.
#[tokio::test]
async fn returns_error_for_wrong_auth_type() {
    let ctx = no_auth_context().await;
    let result = BearerTokenAuthenticator.authenticate(&ctx).await;
    assert!(result.is_err());
}

/// A context without connector instance is rejected.
#[tokio::test]
async fn returns_error_without_connector() {
    let ctx = consumer_context().await;
    let result = BearerTokenAuthenticator.authenticate(&ctx).await;
    assert!(result.is_err());
}
