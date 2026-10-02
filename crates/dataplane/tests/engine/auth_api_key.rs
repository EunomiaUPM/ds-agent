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

//! ApiKeyAuthenticator: resolves the API key credentials of the connector into the runtime.

use dataplane::engine::dataplane_drivers::DriverAuthenticatorTrait;
use dataplane::engine::dataplane_manager::dataplane_runtime::ResolvedAuthCredentials;

use connector::ApiKeyLocation;
use dataplane::engine::dataplane_drivers::authentication::api_key::*;

use crate::support::fixtures::{api_key_context, bearer_context, consumer_context};

/// A plain header API key lands in the runtime with its name, value and location.
#[tokio::test]
async fn resolves_plain_api_key() {
    let ctx = api_key_context("X-Api-Key", "my-key-value", ApiKeyLocation::Header).await;
    let result = ApiKeyAuthenticator.authenticate(&ctx).await;
    assert!(result.is_ok());
    let updated = result.unwrap();
    let runtime = updated.runtime().expect("runtime must be set");
    match &runtime.auth {
        ResolvedAuthCredentials::ApiKey {
            key,
            value,
            location,
        } => {
            assert_eq!(key, "X-Api-Key");
            assert_eq!(value, "my-key-value");
            assert!(matches!(location, ApiKeyLocation::Header));
        }
        other => panic!("Expected ApiKey, got {other:?}"),
    }
}

/// A query-param API key keeps its Query location.
#[tokio::test]
async fn resolves_query_param_api_key() {
    let ctx = api_key_context("api_key", "query-val", ApiKeyLocation::Query).await;
    let result = ApiKeyAuthenticator.authenticate(&ctx).await;
    assert!(result.is_ok());
    let updated = result.unwrap();
    match &updated.runtime().unwrap().auth {
        ResolvedAuthCredentials::ApiKey { location, .. } => {
            assert!(matches!(location, ApiKeyLocation::Query));
        }
        other => panic!("Expected ApiKey, got {other:?}"),
    }
}

/// A connector configured with another auth type is rejected.
#[tokio::test]
async fn returns_error_for_wrong_auth_type() {
    let ctx = bearer_context("token").await;
    let result = ApiKeyAuthenticator.authenticate(&ctx).await;
    assert!(result.is_err());
}

/// A context without connector instance is rejected.
#[tokio::test]
async fn returns_error_without_connector() {
    let ctx = consumer_context().await;
    let result = ApiKeyAuthenticator.authenticate(&ctx).await;
    assert!(result.is_err());
}
