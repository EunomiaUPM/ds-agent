use crate::entities::dataplane_drivers::DriverAuthenticatorTrait;
use crate::entities::dataplane_manager::dataplane_context::DataplaneContext;
use crate::entities::dataplane_manager::dataplane_runtime::{
    DataplaneRuntime, ResolvedAuthCredentials,
};
use connector::{AuthenticationConfig, OAuthGrantType, TemplateVecString, TokenExpireAction};
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};
use ymir::errors::{Errors, Outcome};

/// Response body returned by an OAuth 2.0 token endpoint.
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: Option<u64>,
    refresh_token: Option<String>,
}

#[derive(Debug)]
pub struct OauthAuthenticator;

fn http_client() -> &'static reqwest::Client {
    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    CLIENT.get_or_init(reqwest::Client::new)
}

impl OauthAuthenticator {
    /// POST to `token_url` with the given form fields and deserialize the response.
    async fn post_token(token_url: &str, params: &[(&str, &str)]) -> Outcome<TokenResponse> {
        let resp = http_client()
            .post(token_url)
            .form(params)
            .send()
            .await
            .map_err(|e| {
                Errors::crazy(
                    &format!("OAuth2 token request failed: {}", e),
                    Some(Box::new(e)),
                )
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(Errors::crazy(
                &format!("OAuth2 token endpoint returned {}: {}", status, body),
                None,
            ));
        }

        resp.json::<TokenResponse>().await.map_err(|e| {
            Errors::crazy(
                &format!("Failed to parse OAuth2 token response: {}", e),
                Some(Box::new(e)),
            )
        })
    }

    /// Full grant flow: dispatches on `grant_type` to build the right form body.
    async fn fetch_grant(
        grant_type: &OAuthGrantType,
        token_url: &str,
        client_id: &str,
        client_secret: &str,
        scopes: &[String],
    ) -> Outcome<TokenResponse> {
        let scope_str = scopes.join(" ");
        match grant_type {
            OAuthGrantType::ClientCredentials => {
                let mut params = vec![
                    ("grant_type", "client_credentials"),
                    ("client_id", client_id),
                    ("client_secret", client_secret),
                ];
                if !scope_str.is_empty() {
                    params.push(("scope", &scope_str));
                }
                Self::post_token(token_url, &params).await
            }
            OAuthGrantType::Password { username, password } => {
                let resolved_password = password.resolve().await?;
                let mut params = vec![
                    ("grant_type", "password"),
                    ("client_id", client_id),
                    ("client_secret", client_secret),
                    ("username", username.as_str()),
                    ("password", resolved_password.as_str()),
                ];
                if !scope_str.is_empty() {
                    params.push(("scope", &scope_str));
                }
                Self::post_token(token_url, &params).await
            }
            OAuthGrantType::AuthorizationCode => Err(Errors::crazy(
                "AuthorizationCode grant requires browser interaction and cannot be used in a connector driver",
                None,
            )),
        }
    }

    /// Refresh-token flow. Returns `Err` if the server rejects the refresh token
    /// so the caller can fall back to a full grant.
    async fn fetch_refresh(
        token_url: &str,
        client_id: &str,
        client_secret: &str,
        refresh_token: &str,
    ) -> Outcome<TokenResponse> {
        let params = vec![
            ("grant_type", "refresh_token"),
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("refresh_token", refresh_token),
        ];
        Self::post_token(token_url, &params).await
    }

    /// Resolve a `TemplateVecString` to a `Vec<String>`.
    fn resolve_scopes(scopes: TemplateVecString) -> Vec<String> {
        match scopes {
            TemplateVecString::Value(v) => v,
            TemplateVecString::Template(t) => {
                // Scopes arrived as an unresolved template — parameter resolution did not run
                // before authentication. OAuth proceeds with no scopes; the resulting token may
                // have incorrect permissions.
                tracing::warn!(template = %t, "OAuth2 scopes contain an unresolved template; proceeding with empty scopes");
                vec![]
            }
        }
    }

    fn expires_at_from(expires_in: Option<u64>) -> Option<u64> {
        expires_in.and_then(|secs| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .ok()
                .map(|now| now.as_secs() + secs)
        })
    }
}

#[async_trait::async_trait]
impl DriverAuthenticatorTrait for OauthAuthenticator {
    async fn authenticate(&self, context: &DataplaneContext) -> Outcome<DataplaneContext> {
        // Skip re-auth when the current token is still valid within a 30-second buffer.
        if let Some(runtime) = context.runtime() {
            if let ResolvedAuthCredentials::OAuth2 { expires_at, .. } = &runtime.auth {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .ok()
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                const EXPIRY_BUFFER_SECS: u64 = 30;
                if expires_at.map_or(false, |exp| exp > now + EXPIRY_BUFFER_SECS) {
                    return Ok(context.clone());
                }
            }
        }

        let connector = context
            .connector_instance()
            .ok_or_else(|| Errors::crazy("Connector not available", None))?;

        let AuthenticationConfig::OAuth2 {
            grant_type,
            token_url,
            client_id,
            client_secret,
            scopes,
            on_token_expire,
        } = connector.authentication_config.clone()
        else {
            return Err(Errors::crazy(
                "Connector auth config should be type OAUTH2",
                None,
            ));
        };

        let secret = client_secret.resolve().await?;
        let scopes_vec = Self::resolve_scopes(scopes);

        // If the policy is RefreshOrRefetch and we have a previous refresh_token, try it first.
        let existing_refresh = context.runtime().and_then(|r| match &r.auth {
            ResolvedAuthCredentials::OAuth2 { refresh_token, .. } => {
                refresh_token.as_deref().map(str::to_string)
            }
            _ => None,
        });

        let response = match (&on_token_expire, existing_refresh) {
            (TokenExpireAction::RefreshOrRefetch, Some(rt)) => {
                match Self::fetch_refresh(&token_url, &client_id, &secret, &rt).await {
                    Ok(r) => Ok(r),
                    Err(_) => {
                        Self::fetch_grant(&grant_type, &token_url, &client_id, &secret, &scopes_vec)
                            .await
                    }
                }
            }
            _ => Self::fetch_grant(&grant_type, &token_url, &client_id, &secret, &scopes_vec).await,
        }?;

        let mut ctx = context.clone();
        ctx.set_runtime(DataplaneRuntime {
            auth: ResolvedAuthCredentials::OAuth2 {
                access_token: response.access_token,
                token_type: response.token_type,
                expires_at: Self::expires_at_from(response.expires_in),
                refresh_token: response.refresh_token,
            },
            ..context.runtime().cloned().unwrap_or_default()
        });
        Ok(ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{
        bearer_context, consumer_context, oauth2_context, oauth2_password_context,
    };

    #[tokio::test]
    async fn returns_error_for_wrong_auth_type() {
        let ctx = bearer_context("token").await;
        let result = OauthAuthenticator.authenticate(&ctx).await;
        assert!(result.is_err());
    }

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
            oauth2_password_context("http://127.0.0.1:1", "client-id", "secret", "user", "pass")
                .await;
        let result = OauthAuthenticator.authenticate(&ctx).await;
        assert!(result.is_err());
    }
}
