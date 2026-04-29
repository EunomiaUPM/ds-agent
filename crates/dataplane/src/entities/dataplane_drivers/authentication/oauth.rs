use crate::entities::dataplane_drivers::DriverAuthenticatorTrait;
use crate::entities::dataplane_manager::dataplane_context::DataplaneContext;
use crate::entities::dataplane_manager::dataplane_runtime::{
    DataplaneRuntime, ResolvedAuthCredentials,
};
use connector::{AuthenticationConfig, TemplateVecString};
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};
use ymir::errors::{Errors, Outcome};

/// Response body returned by an OAuth 2.0 token endpoint.
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: Option<u64>,
}

#[derive(Debug)]
pub struct OauthAuthenticator;

impl OauthAuthenticator {
    /// Exchange client credentials for an access token at `token_url`.
    async fn fetch_token(
        token_url: &str,
        client_id: &str,
        client_secret: &str,
        scopes: &[String],
    ) -> Outcome<TokenResponse> {
        let client = reqwest::Client::new();
        let scope_str = scopes.join(" ");
        let mut params = vec![
            ("grant_type", "client_credentials"),
            ("client_id", client_id),
            ("client_secret", client_secret),
        ];
        if !scope_str.is_empty() {
            params.push(("scope", &scope_str));
        }

        let resp = client
            .post(token_url)
            .form(&params)
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

    /// Resolve a `TemplateVecString` to a `Vec<String>`.
    /// Returns an empty Vec if it is still a template placeholder.
    fn resolve_scopes(scopes: TemplateVecString) -> Vec<String> {
        match scopes {
            TemplateVecString::Value(v) => v,
            TemplateVecString::Template(_) => vec![],
        }
    }
}

#[async_trait::async_trait]
impl DriverAuthenticatorTrait for OauthAuthenticator {
    async fn authenticate(&self, context: &DataplaneContext) -> Outcome<DataplaneContext> {
        let connector = context
            .connector_instance()
            .ok_or_else(|| Errors::crazy("Connector not available", None))?;

        let AuthenticationConfig::OAuth2 {
            token_url,
            client_id,
            client_secret,
            scopes,
            ..
        } = connector.authentication_config.clone()
        else {
            return Err(Errors::crazy(
                "Connector auth config should be type OAUTH2",
                None,
            ));
        };

        let secret = client_secret.resolve().await?;
        let scopes_vec = Self::resolve_scopes(scopes);
        let response = Self::fetch_token(&token_url, &client_id, &secret, &scopes_vec).await?;

        // Compute absolute expiry timestamp if the server provided expires_in.
        let expires_at = response.expires_in.and_then(|secs| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .ok()
                .map(|now| now.as_secs() + secs)
        });

        let mut ctx = context.clone();
        ctx.set_runtime(DataplaneRuntime {
            auth: ResolvedAuthCredentials::OAuth2 {
                access_token: response.access_token,
                token_type: response.token_type,
                expires_at,
            },
            ..Default::default()
        });
        Ok(ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{bearer_context, consumer_context, oauth2_context};

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

    /// Validates graceful failure when the token URL is unreachable.
    #[tokio::test]
    async fn returns_error_when_token_url_unreachable() {
        // Port 1 is system-reserved and will always refuse connections.
        let ctx = oauth2_context("http://127.0.0.1:1", "client-id", "secret").await;
        let result = OauthAuthenticator.authenticate(&ctx).await;
        assert!(result.is_err());
    }
}
