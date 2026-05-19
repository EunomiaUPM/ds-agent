use crate::entities::dataplane_manager::dataplane_runtime::ResolvedAuthCredentials;
use axum::body::Bytes;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use connector::ApiKeyLocation;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION};
use reqwest::{Client, Method, Response};
use std::str::FromStr;
use ymir::errors::{Errors, Outcome};

/// Builds auth headers and (for query-param API keys) extra query parameters
/// from already-resolved credentials.
///
/// Returns `(extra_headers, query_params)`.
pub fn build_auth_artifacts(
    credentials: &ResolvedAuthCredentials,
) -> Outcome<(HeaderMap, Vec<(String, String)>)> {
    let mut headers = HeaderMap::new();
    let mut query_params: Vec<(String, String)> = Vec::new();

    match credentials {
        ResolvedAuthCredentials::NoAuth => {}

        ResolvedAuthCredentials::BearerToken { token } => {
            let value = HeaderValue::from_str(&format!("Bearer {}", token)).map_err(|e| {
                Errors::crazy(&format!("Invalid bearer token header value: {}", e), None)
            })?;
            headers.insert(AUTHORIZATION, value);
        }

        ResolvedAuthCredentials::BasicAuth { username, password } => {
            let encoded = BASE64.encode(format!("{}:{}", username, password));
            let value = HeaderValue::from_str(&format!("Basic {}", encoded)).map_err(|e| {
                Errors::crazy(&format!("Invalid basic auth header value: {}", e), None)
            })?;
            headers.insert(AUTHORIZATION, value);
        }

        ResolvedAuthCredentials::ApiKey {
            key,
            value,
            location,
        } => match location {
            ApiKeyLocation::Header => {
                let name = HeaderName::from_str(key).map_err(|e| {
                    Errors::crazy(
                        &format!("Invalid API key header name '{}': {}", key, e),
                        None,
                    )
                })?;
                let hval = HeaderValue::from_str(value).map_err(|e| {
                    Errors::crazy(&format!("Invalid API key header value: {}", e), None)
                })?;
                headers.insert(name, hval);
            }
            ApiKeyLocation::Query => {
                query_params.push((key.clone(), value.clone()));
            }
        },

        ResolvedAuthCredentials::OAuth2 {
            access_token,
            token_type,
            ..
        } => {
            let value = HeaderValue::from_str(&format!("{} {}", token_type, access_token))
                .map_err(|e| {
                    Errors::crazy(&format!("Invalid OAuth2 auth header value: {}", e), None)
                })?;
            headers.insert(AUTHORIZATION, value);
        }
    }

    Ok((headers, query_params))
}

/// Forward an HTTP request to `target_url` (with optional sub-path appended),
/// injecting auth artifacts derived from `credentials`.
///
/// Incoming client headers are forwarded as-is; auth headers from `credentials`
/// override any existing `Authorization` header from the client.
pub async fn forward(
    client: &Client,
    method: Method,
    target_url: &str,
    extra_path: Option<&str>,
    incoming_headers: &HeaderMap,
    body: Bytes,
    credentials: &ResolvedAuthCredentials,
) -> Outcome<Response> {
    let mut url = target_url.to_string();
    if let Some(p) = extra_path {
        if !url.ends_with('/') {
            url.push('/');
        }
        url.push_str(p);
    }

    let (auth_headers, query_params) = build_auth_artifacts(credentials)?;

    // Append query-param API key to the URL if needed.
    if !query_params.is_empty() {
        let separator = if url.contains('?') { '&' } else { '?' };
        let qs: String = query_params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");
        url.push(separator);
        url.push_str(&qs);
    }

    // Start building the request, copying incoming headers first.
    let mut builder = client.request(method, &url);
    for (name, value) in incoming_headers.iter() {
        // Skip hop-by-hop headers that must not be forwarded.
        let key = name.as_str().to_ascii_lowercase();
        if matches!(
            key.as_str(),
            "host" | "connection" | "transfer-encoding" | "te" | "trailer" | "upgrade"
        ) {
            continue;
        }
        builder = builder.header(name.clone(), value.clone());
    }

    // Auth headers override whatever the client sent.
    for (name, value) in auth_headers.iter() {
        builder = builder.header(name.clone(), value.clone());
    }

    builder.body(body).send().await.map_err(|e| {
        Errors::crazy(
            &format!("Upstream request failed: {}", e),
            Some(Box::new(e)),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::dataplane_manager::dataplane_runtime::ResolvedAuthCredentials;
    use connector::ApiKeyLocation;
    use reqwest::header::AUTHORIZATION;

    fn auth_headers(creds: &ResolvedAuthCredentials) -> HeaderMap {
        build_auth_artifacts(creds).unwrap().0
    }

    fn auth_query(creds: &ResolvedAuthCredentials) -> Vec<(String, String)> {
        build_auth_artifacts(creds).unwrap().1
    }

    #[test]
    fn no_auth_produces_no_headers() {
        let headers = auth_headers(&ResolvedAuthCredentials::NoAuth);
        assert!(headers.is_empty());
    }

    #[test]
    fn bearer_token_produces_authorization_header() {
        let creds = ResolvedAuthCredentials::BearerToken {
            token: "my-secret-token".to_string(),
        };
        let headers = auth_headers(&creds);
        let auth = headers.get(AUTHORIZATION).unwrap().to_str().unwrap();
        assert_eq!(auth, "Bearer my-secret-token");
    }

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
}
