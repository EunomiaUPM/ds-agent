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

//! BasicConfigAuthenticator: resolves Basic credentials of the connector into the runtime.

use dataplane::engine::dataplane_drivers::DriverAuthenticatorTrait;
use dataplane::engine::dataplane_manager::dataplane_runtime::ResolvedAuthCredentials;

use dataplane::engine::dataplane_drivers::authentication::basic_config::*;

use crate::support::fixtures::{basic_auth_context, bearer_context, consumer_context};

/// Plain username and password land in the runtime.
#[tokio::test]
async fn resolves_plain_basic_auth() {
    let ctx = basic_auth_context("alice", "s3cr3t").await;
    let result = BasicConfigAuthenticator.authenticate(&ctx).await;
    assert!(result.is_ok());
    let updated = result.unwrap();
    let runtime = updated.runtime().expect("runtime must be set");
    match &runtime.auth {
        ResolvedAuthCredentials::BasicAuth { username, password } => {
            assert_eq!(username, "alice");
            assert_eq!(password, "s3cr3t");
        }
        other => panic!("Expected BasicAuth, got {other:?}"),
    }
}

/// A connector configured with another auth type is rejected.
#[tokio::test]
async fn returns_error_for_wrong_auth_type() {
    let ctx = bearer_context("token").await;
    let result = BasicConfigAuthenticator.authenticate(&ctx).await;
    assert!(result.is_err());
}

/// A context without connector instance is rejected.
#[tokio::test]
async fn returns_error_without_connector() {
    let ctx = consumer_context().await;
    let result = BasicConfigAuthenticator.authenticate(&ctx).await;
    assert!(result.is_err());
}
