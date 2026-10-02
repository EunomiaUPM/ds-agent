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

//! The HTTP proxy's auth headers and query params for each kind of resolved credentials.

use axum::http::header::AUTHORIZATION;
use axum::http::HeaderMap;
use connector::ApiKeyLocation;
use dataplane::engine::dataplane_manager::dataplane_runtime::ResolvedAuthCredentials;

use dataplane::engine::dataplane_drivers::proxy::http::HttpProxyDriver;

fn auth_headers(creds: &ResolvedAuthCredentials) -> HeaderMap {
    HttpProxyDriver::build_auth_artifacts(creds).unwrap().0
}

fn auth_query(creds: &ResolvedAuthCredentials) -> Vec<(String, String)> {
    HttpProxyDriver::build_auth_artifacts(creds).unwrap().1
}

/// No auth adds no headers.
#[test]
fn no_auth_produces_no_headers() {
    let headers = auth_headers(&ResolvedAuthCredentials::NoAuth);
    assert!(headers.is_empty());
}

/// A bearer token becomes `Authorization: Bearer <token>`.
#[test]
fn bearer_token_produces_authorization_header() {
    let creds = ResolvedAuthCredentials::BearerToken {
        token: "my-secret-token".to_string(),
    };
    let headers = auth_headers(&creds);
    let auth = headers.get(AUTHORIZATION).unwrap().to_str().unwrap();
    assert_eq!(auth, "Bearer my-secret-token");
}

/// Basic credentials become a base64 `Authorization: Basic` header.
#[test]
fn basic_auth_produces_base64_authorization_header() {
    let creds = ResolvedAuthCredentials::BasicAuth {
        username: "alice".to_string(),
        password: "wonderland".to_string(),
    };
    let headers = auth_headers(&creds);
    let auth = headers.get(AUTHORIZATION).unwrap().to_str().unwrap();
    // base64("alice:wonderland") = "YWxpY2U6d29uZGVybGFuZA=="
    assert_eq!(auth, "Basic YWxpY2U6d29uZGVybGFuZA==");
}

/// A header API key becomes a header named after the key.
#[test]
fn api_key_header_location_adds_header() {
    let creds = ResolvedAuthCredentials::ApiKey {
        key: "x-api-key".to_string(),
        value: "key-value-123".to_string(),
        location: ApiKeyLocation::Header,
    };
    let headers = auth_headers(&creds);
    let val = headers.get("x-api-key").unwrap().to_str().unwrap();
    assert_eq!(val, "key-value-123");
}

/// A query API key becomes a query param and adds no header.
#[test]
fn api_key_query_location_adds_query_param() {
    let creds = ResolvedAuthCredentials::ApiKey {
        key: "api_key".to_string(),
        value: "secret".to_string(),
        location: ApiKeyLocation::Query,
    };
    let headers = auth_headers(&creds);
    assert!(headers.is_empty(), "no headers for query param");

    let params = auth_query(&creds);
    assert_eq!(params, vec![("api_key".to_string(), "secret".to_string())]);
}

/// An OAuth2 token is sent with its token type as prefix.
#[test]
fn oauth2_uses_token_type_prefix() {
    let creds = ResolvedAuthCredentials::OAuth2 {
        access_token: "abc123".to_string(),
        token_type: "Bearer".to_string(),
        expires_at: None,
        refresh_token: None,
    };
    let headers = auth_headers(&creds);
    let auth = headers.get(AUTHORIZATION).unwrap().to_str().unwrap();
    assert_eq!(auth, "Bearer abc123");
}

/// A non-Bearer token type such as MAC is kept as the prefix.
#[test]
fn oauth2_with_non_bearer_token_type() {
    let creds = ResolvedAuthCredentials::OAuth2 {
        access_token: "xyz".to_string(),
        token_type: "MAC".to_string(),
        expires_at: None,
        refresh_token: None,
    };
    let headers = auth_headers(&creds);
    let auth = headers.get(AUTHORIZATION).unwrap().to_str().unwrap();
    assert_eq!(auth, "MAC xyz");
}
